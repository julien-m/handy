//! Pure detection of a modifier key pressed alone (014): the key goes down, then up, and nothing
//! else happens in between: no other key pressed or released, no other modifier, and no shortcut
//! of Handy fired (a backend may hide the keys of a shortcut it owns, such as Option+Space, from
//! this observer). The auto-repeat of a key held since before the press (Space of a push-to-talk
//! Option+Space) is not a new key. Modifiers held before the press do not matter, and the
//! duration of the press does not matter either.
use handy_keys::{Key, Modifiers};
use std::collections::HashSet;

/// One keyboard event as the observer reports it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyInput {
    /// One modifier key (a single side bit) went down or up.
    Modifier { key: Modifiers, down: bool },
    /// Any other key (or a mouse button with a modifier held) went down or up.
    Other { key: Key, down: bool },
}

/// A lone press: the modifier key (one side bit) and the count of Handy shortcut events when it
/// went down. The observer confirms it once that count is known to be unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Tap {
    pub key: Modifiers,
    pub shortcuts_at_press: u64,
}

/// A modifier key that went down and is still clean.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Pending {
    key: Modifiers,
    shortcuts_at_press: u64,
    clean: bool,
}

// @spec FR-002: Tap only: press then release, nothing between — .specs/features/014-in-dictation-keys/spec.md#fr-002
/// Detector of lone modifier presses.
#[derive(Debug, Default)]
pub struct TapDetector {
    pending: Option<Pending>,
    keys_down: HashSet<Key>,
}

impl TapDetector {
    /// Feed one event; `shortcuts_seen` is the count of Handy shortcut events so far. Returns the
    /// lone press ended by this event, if any.
    pub fn feed(&mut self, input: KeyInput, shortcuts_seen: u64) -> Option<Tap> {
        match input {
            KeyInput::Modifier { key, down: true } => {
                self.on_modifier_down(key, shortcuts_seen);
                None
            }
            KeyInput::Modifier { key, down: false } => self.on_modifier_up(key, shortcuts_seen),
            KeyInput::Other { key, down: true } => {
                // A repeat of a key already down is not a new key.
                if self.keys_down.insert(key) {
                    self.dirty();
                }
                None
            }
            KeyInput::Other { key, down: false } => {
                self.keys_down.remove(&key);
                self.dirty();
                None
            }
        }
    }

    fn on_modifier_down(&mut self, key: Modifiers, shortcuts_seen: u64) {
        match self.pending {
            // A second modifier during the press: a combination, never a lone press.
            Some(pending) if pending.key != key => self.dirty(),
            // A new press of the pending key (its release was never seen) or a first press.
            _ => {
                self.pending = Some(Pending {
                    key,
                    shortcuts_at_press: shortcuts_seen,
                    clean: true,
                });
            }
        }
    }

    fn on_modifier_up(&mut self, key: Modifiers, shortcuts_seen: u64) -> Option<Tap> {
        let pending = self.pending?;
        if pending.key != key {
            // Another modifier released during the press.
            self.dirty();
            return None;
        }
        self.pending = None;
        let alone = pending.clean && pending.shortcuts_at_press == shortcuts_seen;
        alone.then_some(Tap {
            key,
            shortcuts_at_press: pending.shortcuts_at_press,
        })
    }

    fn dirty(&mut self) {
        if let Some(pending) = self.pending.as_mut() {
            pending.clean = false;
        }
    }
}

/// The modifier keys held, as the observer knows them. On macOS the listener reports a modifier
/// change as a toggle of its own state, which starts empty: a key held when it starts would be
/// reported the wrong way round. The observer then starts from the system's real state and
/// derives the direction itself; elsewhere the reported direction is right.
#[derive(Clone, Copy, Debug)]
pub struct ModifierState {
    held: Modifiers,
    derive_direction: bool,
}

impl ModifierState {
    /// State starting from `held`; `derive_direction` ignores the reported direction.
    pub fn new(held: Modifiers, derive_direction: bool) -> Self {
        Self {
            held,
            derive_direction,
        }
    }
    /// Whether the change of `key` reported as `reported_down` is a press.
    pub fn is_press(&mut self, key: Modifiers, reported_down: bool) -> bool {
        let down = if self.derive_direction {
            !self.held.contains(key)
        } else {
            reported_down
        };
        if down {
            self.held |= key;
        } else {
            self.held &= !key;
        }
        down
    }
}

/// Whether the configured `configured` key (one modifier family, one or both sides) is the
/// tapped `key` (one side bit).
pub fn is_configured_key(configured: Modifiers, key: Modifiers) -> bool {
    configured.contains(key)
}

#[cfg(test)]
#[path = "tap_tests.rs"]
mod tests;
