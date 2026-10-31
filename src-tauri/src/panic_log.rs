//! Writes every panic to `handy.log` before the default hook runs. A release build is stripped
//! and its stderr is lost when launched from Finder, so a panic on the main thread (which
//! aborts the app: the event loop cannot unwind) left no message at all (finding N32).
use std::panic::{self, PanicHookInfo};

/// Chains a hook that logs the panic message, its location and its thread.
pub fn install() {
    let default = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        log::error!("{}", describe(info));
        default(info);
    }));
}

fn describe(info: &PanicHookInfo<'_>) -> String {
    let thread = std::thread::current();
    let name = thread.name().unwrap_or("unnamed");
    let message = info
        .payload()
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| info.payload().downcast_ref::<String>().map(String::as_str))
        .unwrap_or("non-string payload");
    let location = info
        .location()
        .map_or_else(|| "unknown location".to_owned(), ToString::to_string);
    format!("Panic on thread '{name}' at {location}: {message}")
}
