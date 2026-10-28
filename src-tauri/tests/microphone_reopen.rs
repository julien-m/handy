//! Feature 001, AC-004 non-regression: at an always-on recording start, the open stream is
//! compared with what the priority resolves to now. A first choice that is present but cannot be
//! opened must never hide a lower choice that reconnected: with [Dock, AirPods], Dock busy and
//! AirPods away, the stream opens on the system default; once AirPods are back, the next start
//! reopens and lands on AirPods.
use handy_app_lib::microphones::catalog::{Catalog, CatalogEntry};
use handy_app_lib::microphones::effective::{EffectiveInput, EffectiveState};
use handy_app_lib::microphones::policy::{
    preferred, resolve, OpenFailure, OpenTarget, Opened, Preferences, ResolutionPorts,
};
use std::time::Duration;

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Present devices, of which `busy` ones refuse to open; the system default always opens.
struct Fake {
    present: Vec<&'static str>,
    busy: Vec<&'static str>,
}
impl ResolutionPorts for Fake {
    type Device = String;
    type Stream = String;
    fn elapsed(&self) -> Duration {
        Duration::ZERO
    }
    fn lid_closed(&mut self, _budget: Duration) -> Option<bool> {
        None
    }
    fn enumerate(&mut self, _budget: Duration) -> Option<Catalog<String>> {
        let entries = self.present.iter().map(|name| CatalogEntry {
            name: (*name).to_owned(),
            device: (*name).to_owned(),
        });
        Some(Catalog::new(entries.collect()))
    }
    fn open(&mut self, target: OpenTarget<'_, String>, _: Duration) -> Result<String, OpenFailure> {
        match target {
            OpenTarget::Device(name) if self.busy.contains(&name.as_str()) => {
                Err(OpenFailure::Failed("busy".into()))
            }
            OpenTarget::Device(name) => Ok(name.clone()),
            OpenTarget::SystemDefault => Ok("<default>".into()),
        }
    }
}

fn effective(opened: Opened<String>) -> EffectiveInput {
    EffectiveInput {
        name: Some(opened.stream),
        choice: opened.choice,
        reason: opened.reason,
        capture_generation: 0,
    }
}

// @spec AC-004: A reconnected higher priority device is preferred on the next recording — .specs/features/001-microphone-priority/spec.md#ac-004
#[test]
fn a_reconnected_second_choice_reopens_a_default_stream_left_by_a_busy_first_choice() -> TestResult
{
    let priority = vec!["Dock".to_owned(), "AirPods".to_owned()];
    let preferences = Preferences {
        priority: &priority,
        clamshell: None,
    };
    let mut away = Fake {
        present: vec!["Dock"],
        busy: vec!["Dock"],
    };
    let opened = resolve(&preferences, &mut away)
        .opened
        .ok_or("nothing opened")?;
    assert_eq!(
        opened.choice, None,
        "Dock busy, AirPods away: the default opens"
    );
    let mut state = EffectiveState::default();
    state.stream_opened(effective(opened));

    let mut back = Fake {
        present: vec!["Dock", "AirPods"],
        busy: vec!["Dock"],
    };
    let now = preferred(&preferences, &mut back).ok_or("catalog unobservable")?;
    assert!(
        !state.is_on(now.as_deref()),
        "AirPods are back: the next start must reopen the default stream"
    );
    let reopened = resolve(&preferences, &mut back)
        .opened
        .ok_or("nothing opened")?;
    assert_eq!(reopened.choice.as_deref(), Some("AirPods"));
    Ok(())
}
