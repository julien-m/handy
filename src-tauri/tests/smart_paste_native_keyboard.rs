//! Opt-in, read-only native check of the production Text Input Sources and
//! key adapters: reads the current input source, applies the AC-011 decision
//! and resolves the Cmd+V key for the active layout. It never selects a
//! source, enables a layout or posts a key event.
//!
//! Text Input Sources require the main thread, which libtest does not run
//! tests on, so this target has its own `main` (harness = false). Run with
//! `cargo test --test smart_paste_native_keyboard -- --ignored`.

#[cfg(target_os = "macos")]
fn main() -> std::process::ExitCode {
    use handy_app_lib::smart_paste::key_events::KeyEventPort;
    use handy_app_lib::smart_paste::keyboard::{decide_layout, InputSourcePort};
    use handy_app_lib::smart_paste::macos_keyboard::{MacInputSources, MacKeyEvents};

    if !std::env::args().any(|argument| argument == "--ignored") {
        println!("smart_paste_native_keyboard: 1 ignored (opt-in: -- --ignored)");
        return std::process::ExitCode::SUCCESS;
    }
    // On the main thread the adapters run their Text Input Sources calls inline.
    let source = MacInputSources.current_source();
    let key = MacKeyEvents.resolve_paste_key();
    let (Ok(source), Ok(key)) = (source, key) else {
        eprintln!("native keyboard check failed: input source or key unavailable");
        return std::process::ExitCode::FAILURE;
    };
    let observation = serde_json::json!({
        "check": "keyboard_read_only",
        "current_source_id": source.id,
        "current_source_kind": format!("{:?}", source.kind),
        "decision": format!("{:?}", decide_layout(&source)),
        "command_v_key_code": key.0,
        "source_selection": "pending release evidence: ABC/US/Dvorak are not enabled here; enabling them changes system settings",
        "key_posting": "pending release evidence: posting keys would type into the user's focused application",
    });
    println!("{observation}");
    println!("test result: ok. native keyboard read-only check passed");
    std::process::ExitCode::SUCCESS
}

#[cfg(not(target_os = "macos"))]
fn main() {}
