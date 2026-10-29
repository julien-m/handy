//! Scripted elements of the 011 accessibility world (`fake.rs`): the facts of 010's rule, a
//! selection range, scripted write answers, and how a range write lands (late, normalized).
use handy_app_lib::recovery::classifier::AxQueryError;
use std::time::Duration;

/// How a scripted element answers a focus or range write.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Write {
    /// Answered and applied.
    Apply,
    /// Refused at once (element gone, not focusable).
    Refuse,
    /// No answer in time (`kAXErrorCannotComplete`), not applied.
    Unanswered,
    /// No answer in time, but the target applies it anyway.
    UnansweredApplied,
}

/// One scripted element.
#[derive(Clone, Debug)]
pub struct Element {
    pub pid: u32,
    pub role: &'static str,
    pub subrole: Option<&'static str>,
    pub settable: bool,
    pub range: Result<(usize, usize), AxQueryError>,
    pub window: Option<u32>,
    pub focus: Write,
    pub select: Write,
    pub raise_works: bool,
    /// Where an applied focus write really lands (a background tab: another element).
    pub focus_lands_on: Option<u32>,
    /// An applied range write shows in reads only after this delay (Chromium applies it later).
    pub select_lag: Duration,
    /// The range an applied range write really sets (an application that normalizes it).
    pub select_lands_at: Option<(usize, usize)>,
}
impl Element {
    /// An editable text area of `pid` in window `window`, caret at `range`.
    pub fn input(pid: u32, window: u32, range: (usize, usize)) -> Self {
        Self {
            pid,
            role: "AXTextArea",
            subrole: None,
            settable: true,
            range: Ok(range),
            window: Some(window),
            focus: Write::Apply,
            select: Write::Apply,
            raise_works: true,
            focus_lands_on: None,
            select_lag: Duration::ZERO,
            select_lands_at: None,
        }
    }
    /// A non-editable element of `role` (a list, a button…) of `pid`.
    pub fn control(pid: u32, role: &'static str) -> Self {
        Self {
            settable: false,
            role,
            ..Self::input(pid, 0, (0, 0))
        }
    }
}

/// A write sent while the application answers late gets no answer in time; the target applies
/// it once it answers, unless it would have refused it.
pub(super) fn late(answered: bool, scripted: Write) -> Write {
    match (answered, scripted) {
        (true, _) => scripted,
        (false, Write::Apply | Write::UnansweredApplied) => Write::UnansweredApplied,
        (false, Write::Refuse | Write::Unanswered) => Write::Unanswered,
    }
}
