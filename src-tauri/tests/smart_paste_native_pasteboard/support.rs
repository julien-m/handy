//! Main-loop driver, external reader process and named-pasteboard fixtures.

use std::ffi::c_void;
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use handy_app_lib::smart_paste::clock::{Clock, SystemClock};
use handy_app_lib::smart_paste::macos_pasteboard::MacPasteboard;
use handy_app_lib::smart_paste::main_thread::run_on_main;
use handy_app_lib::smart_paste::pasteboard_port::ReadEvent;
use handy_app_lib::smart_paste::snapshot_helper::HelperCommand;
use objc2::rc::Retained;
use objc2::runtime::{NSObjectProtocol, ProtocolObject};
use objc2::{define_class, msg_send, AnyThread};
use objc2_app_kit::{
    NSPasteboard, NSPasteboardItem, NSPasteboardItemDataProvider, NSPasteboardType,
    NSPasteboardWriting,
};
use objc2_foundation::{NSArray, NSData, NSObject, NSString};

/// Argument turning this binary into an external pasteboard reader.
const READ_FLAG: &str = "--read-named-pasteboard";
/// Safety bound for the whole scenario; real runs take a few seconds.
const SCENARIO_TIMEOUT: Duration = Duration::from_secs(120);
/// Run-loop slice: short enough to serve requests promptly.
const RUN_LOOP_SLICE_SECONDS: f64 = 0.005;
/// Main-thread fixture operations are a few AppKit calls.
const FIXTURE_MAIN_TIMEOUT: Duration = Duration::from_secs(5);
/// A reader process reads one string; five seconds fails closed.
const READER_TIMEOUT: Duration = Duration::from_secs(5);
/// The blocking promise used to exercise the helper deadline.
pub const SLOW_PROMISE_DELAY: Duration = Duration::from_millis(1000);

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFRunLoopDefaultMode: *const c_void;
    fn CFRunLoopRunInMode(mode: *const c_void, seconds: f64, return_after_source: u8) -> i32;
}

/// One saved or observed pasteboard type: identifier and raw bytes.
pub type TypeBytes = (String, Vec<u8>);

/// Dispatches between reader mode, opt-out and the scenario.
pub fn main_with_run_loop(scenario: fn() -> Result<Vec<serde_json::Value>, String>) -> ExitCode {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.get(1).map(String::as_str) == Some(READ_FLAG) {
        return read_named_pasteboard(arguments.get(2).map(String::as_str).unwrap_or_default());
    }
    if !arguments.iter().any(|argument| argument == "--ignored") {
        println!("smart_paste_native_pasteboard: 1 ignored (opt-in: -- --ignored)");
        return ExitCode::SUCCESS;
    }
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(scenario());
    });
    report(pump_until(&receiver))
}

fn pump_until(
    receiver: &Receiver<Result<Vec<serde_json::Value>, String>>,
) -> Result<Vec<serde_json::Value>, String> {
    let deadline = Instant::now() + SCENARIO_TIMEOUT;
    // Serves on-demand items and main-queue work until the scenario ends.
    while Instant::now() < deadline {
        // SAFETY: called on the process main thread with a CF constant.
        unsafe { CFRunLoopRunInMode(kCFRunLoopDefaultMode, RUN_LOOP_SLICE_SECONDS, 0) };
        if let Ok(result) = receiver.try_recv() {
            return result;
        }
    }
    Err("native pasteboard scenario exceeded its safety deadline".into())
}

fn report(result: Result<Vec<serde_json::Value>, String>) -> ExitCode {
    match result {
        Ok(observations) => {
            observations.iter().for_each(|line| println!("{line}"));
            println!("test result: ok. native pasteboard scenario passed");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("native pasteboard scenario failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn read_named_pasteboard(name: &str) -> ExitCode {
    let pasteboard = NSPasteboard::pasteboardWithName(&NSString::from_str(name));
    let text = pasteboard.stringForType(&NSString::from_str("public.utf8-plain-text"));
    // Only the public synthetic fixture is ever printed.
    print!("{}", text.map(|text| text.to_string()).unwrap_or_default());
    ExitCode::SUCCESS
}

/// Runs the helper binary once and returns the wall time in ms.
///
/// Cargo rewrites the binary before tests; macOS assesses the first
/// execution of a newly written binary (about two seconds observed). In
/// production the helper is the running app's own, already launched
/// executable, so the scenario measures it warm and reports this cold time.
pub fn helper_first_execution_ms() -> f64 {
    let started = Instant::now();
    let _ = Command::new(env!("CARGO_BIN_EXE_handy"))
        .arg("--smart-paste-snapshot")
        .stdout(Stdio::null())
        .status();
    started.elapsed().as_secs_f64() * 1000.0
}

/// Launches an external reader of the named pasteboard and returns its text.
pub fn external_read(name: &str) -> Result<String, String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let mut child = Command::new(executable)
        .args([READ_FLAG, name])
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;
    let deadline = Instant::now() + READER_TIMEOUT;
    while child
        .try_wait()
        .map_err(|error| error.to_string())?
        .is_none()
    {
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err("reader exceeded its deadline".into());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let output = child
        .wait_with_output()
        .map_err(|error| error.to_string())?;
    String::from_utf8(output.stdout).map_err(|error| error.to_string())
}

/// Adapter under test plus the reads its providers reported.
pub struct Harness {
    pub name: String,
    pub pasteboard: MacPasteboard,
    pub reads: Receiver<ReadEvent>,
    pub clock: Arc<SystemClock>,
}

/// Creates the production adapter on a fresh uniquely named pasteboard,
/// with the real Handy binary as snapshot helper.
pub fn harness() -> Harness {
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    let name = format!(
        "com.handy.smart-paste.native-test.{}.{nanos}",
        std::process::id()
    );
    let (sender, reads) = mpsc::channel();
    let clock = Arc::new(SystemClock);
    let helper = HelperCommand {
        executable: PathBuf::from(env!("CARGO_BIN_EXE_handy")),
        pasteboard_name: Some(name.clone()),
    };
    let sink = Arc::new(move |event: ReadEvent| {
        let _ = sender.send(event);
    });
    let pasteboard = MacPasteboard::new(Some(name.clone()), helper, sink, clock.clone());
    Harness {
        name,
        pasteboard,
        reads,
        clock,
    }
}

impl Harness {
    /// Current monotonic instant of the adapter clock.
    pub fn now(&self) -> Instant {
        self.clock.now()
    }
}

fn named(name: &str) -> Retained<NSPasteboard> {
    NSPasteboard::pasteboardWithName(&NSString::from_str(name))
}

/// Replaces the named pasteboard content with eager items.
pub fn seed(name: &str, items: Vec<Vec<TypeBytes>>) -> Result<(), String> {
    let name = name.to_string();
    run_on_main(
        move || {
            let pasteboard = named(&name);
            pasteboard.clearContents();
            let objects: Vec<_> = items.iter().map(|types| eager_item(types)).collect();
            objects.is_empty() || pasteboard.writeObjects(&NSArray::from_retained_slice(&objects))
        },
        FIXTURE_MAIN_TIMEOUT,
    )
    .map_err(|error| format!("{error:?}"))?
    .then_some(())
    .ok_or_else(|| "seeding the named pasteboard failed".to_string())
}

fn eager_item(types: &[TypeBytes]) -> Retained<ProtocolObject<dyn NSPasteboardWriting>> {
    let item = NSPasteboardItem::new();
    for (uti, bytes) in types {
        item.setData_forType(&NSData::with_bytes(bytes), &NSString::from_str(uti));
    }
    ProtocolObject::from_retained(item)
}

/// Reads every item and type of the named pasteboard (eager content only).
pub fn read_all(name: &str) -> Result<Vec<Vec<TypeBytes>>, String> {
    let name = name.to_string();
    run_on_main(
        move || {
            let items = named(&name).pasteboardItems().map(|items| items.to_vec());
            items
                .unwrap_or_default()
                .iter()
                .map(|item| {
                    let types = item.types().to_vec();
                    types
                        .iter()
                        .map(|uti| {
                            let data = item.dataForType(uti).map(|data| data.to_vec());
                            (uti.to_string(), data.unwrap_or_default())
                        })
                        .collect()
                })
                .collect()
        },
        FIXTURE_MAIN_TIMEOUT,
    )
    .map_err(|error| format!("{error:?}"))
}

/// Writes plain text as another application would (a user copy).
pub fn user_copy(name: &str, text: &str) -> Result<(), String> {
    seed(
        name,
        vec![vec![(
            "public.utf8-plain-text".into(),
            text.as_bytes().to_vec(),
        )]],
    )
}

define_class!(
    // SAFETY: NSObject has no subclassing requirements and there are no ivars.
    #[unsafe(super(NSObject))]
    #[name = "HandySmartPasteSlowPromiseFixture"]
    struct SlowPromise;

    unsafe impl NSObjectProtocol for SlowPromise {}

    unsafe impl NSPasteboardItemDataProvider for SlowPromise {
        #[unsafe(method(pasteboard:item:provideDataForType:))]
        fn provide(
            &self,
            _pasteboard: Option<&NSPasteboard>,
            item: &NSPasteboardItem,
            data_type: &NSPasteboardType,
        ) {
            // Simulates an application whose promise blocks for one second.
            std::thread::sleep(SLOW_PROMISE_DELAY);
            item.setData_forType(&NSData::with_bytes(b"late"), data_type);
        }
    }
);

thread_local! {
    static SLOW_PROMISES: std::cell::RefCell<Vec<Retained<SlowPromise>>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Seeds one item with an eager type and a type promised by a provider that
/// blocks the main thread for [`SLOW_PROMISE_DELAY`].
pub fn seed_slow_promise(name: &str, eager: TypeBytes, slow_uti: &str) -> Result<(), String> {
    let (name, slow_uti) = (name.to_string(), slow_uti.to_string());
    run_on_main(
        move || {
            let pasteboard = named(&name);
            pasteboard.clearContents();
            let item = NSPasteboardItem::new();
            item.setData_forType(&NSData::with_bytes(&eager.1), &NSString::from_str(&eager.0));
            // SAFETY: NSObject's designated initializer on a fresh instance.
            let provider: Retained<SlowPromise> = unsafe { msg_send![SlowPromise::alloc(), init] };
            let promised = NSArray::from_retained_slice(&[NSString::from_str(&slow_uti)]);
            item.setDataProvider_forTypes(ProtocolObject::from_ref(&*provider), &promised);
            SLOW_PROMISES.with(|providers| providers.borrow_mut().push(provider));
            let item = ProtocolObject::<dyn NSPasteboardWriting>::from_retained(item);
            pasteboard.writeObjects(&NSArray::from_retained_slice(&[item]))
        },
        FIXTURE_MAIN_TIMEOUT,
    )
    .map_err(|error| format!("{error:?}"))?
    .then_some(())
    .ok_or_else(|| "seeding the slow promise failed".to_string())
}

/// Removes the named pasteboard's content at the end of the scenario.
pub fn release(name: &str) {
    let name = name.to_string();
    let _ = run_on_main(
        move || {
            let pasteboard = named(&name);
            pasteboard.clearContents();
            // SAFETY: `releaseGlobally` takes no arguments and returns void;
            // it frees the named pasteboard created for this run.
            let _: () = unsafe { msg_send![&*pasteboard, releaseGlobally] };
        },
        FIXTURE_MAIN_TIMEOUT,
    );
}
