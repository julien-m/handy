//! N26: the panel steps against an independent model of an `NSPanel` (on screen or not, opaque
//! or not), from the state a drag leaves it in.
use super::*;
use crate::recovery::types::DragResult;

/// The panel as the user sees it.
#[derive(Clone, Copy, Debug)]
struct Panel {
    on_screen: bool,
    opaque: bool,
}

impl Panel {
    /// During a drag: still ordered in, but transparent.
    const DRAGGING: Self = Self {
        on_screen: true,
        opaque: false,
    };
    /// A bubble shown and not dragged.
    const SHOWN: Self = Self {
        on_screen: true,
        opaque: true,
    };

    fn visible(self) -> bool {
        self.on_screen && self.opaque
    }

    fn apply(self, step: PanelStep) -> Self {
        match step {
            PanelStep::OrderOut => Self {
                on_screen: false,
                ..self
            },
            PanelStep::Reveal => Self {
                opaque: true,
                ..self
            },
        }
    }

    /// Every state the panel goes through while `steps` run, the starting one excluded.
    fn through(self, steps: &[PanelStep]) -> Vec<Self> {
        steps
            .iter()
            .scan(self, |panel, step| {
                *panel = panel.apply(*step);
                Some(*panel)
            })
            .collect()
    }
}

/// What the service does with the panel for `result` (`hide` or `show_again`).
fn steps_after(result: DragResult) -> &'static [PanelStep] {
    match result {
        DragResult::Released => &RESTORE_STEPS,
        DragResult::Dropped | DragResult::CancelledByEscape => &HIDE_STEPS,
    }
}

// @spec AC-004: Escape never shows the bubble again — .specs/features/010-recovery-bubble/spec.md#ac-004
#[test]
fn a_bubble_retired_during_its_drag_is_never_visible_again_and_a_release_shows_it() {
    for accepted in [false, true] {
        for escape in [false, true] {
            let result = DragResult::of_session(accepted, escape);
            let states = Panel::DRAGGING.through(steps_after(result));
            let comes_back = result == DragResult::Released;
            let last = states.last().copied().unwrap_or(Panel::DRAGGING);
            assert_eq!(last.visible(), comes_back, "{result:?} ends visible");
            if !comes_back {
                assert!(
                    states.iter().all(|panel| !panel.visible()),
                    "{result:?} showed the panel: {states:?}"
                );
            }
        }
    }
}

#[test]
fn hiding_takes_the_panel_off_the_screen_before_revealing_it() {
    let order_out = HIDE_STEPS
        .iter()
        .position(|step| *step == PanelStep::OrderOut);
    let reveal = HIDE_STEPS
        .iter()
        .position(|step| *step == PanelStep::Reveal);
    assert!(matches!((order_out, reveal), (Some(out), Some(shown)) if out < shown));
}

#[test]
fn hiding_a_bubble_shown_without_a_drag_leaves_it_off_screen_and_ready() {
    let states = Panel::SHOWN.through(&HIDE_STEPS);
    let last = states.last().copied().unwrap_or(Panel::SHOWN);
    assert!(!last.on_screen);
    // Ready for the next presentation: ordering it in is enough to show it.
    assert!(last.opaque);
}
