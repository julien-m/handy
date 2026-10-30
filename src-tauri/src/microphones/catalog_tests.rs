//! Availability projection of one observation (pure).
use super::{availability, Catalog, CatalogEntry};
use crate::microphones::types::Availability;

fn catalog(names: &[&str], complete: bool) -> Catalog<()> {
    let entries = names
        .iter()
        .map(|name| CatalogEntry {
            name: (*name).to_owned(),
            device: (),
        })
        .collect();
    if complete {
        Catalog::new(entries)
    } else {
        Catalog::partial(entries)
    }
}
fn statuses(configured: &[&str], catalog: Option<&Catalog<()>>) -> Vec<Availability> {
    let configured = configured
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<Vec<_>>();
    availability(&configured, catalog)
        .into_iter()
        .map(|status| status.status)
        .collect()
}

// @spec AC-002: Available, unavailable, ambiguous or unknown — .specs/features/001-microphone-priority/spec.md#ac-002
#[test]
fn each_configured_choice_gets_one_of_four_statuses_in_saved_order() {
    let present = catalog(&["USB", "Dock", "Dock"], true);
    let observed = statuses(&["Dock", "USB", "Gone"], Some(&present));
    let expected = [
        Availability::Ambiguous,
        Availability::Available,
        Availability::Unavailable,
    ];
    assert_eq!(observed, expected);
}
#[test]
fn a_failed_or_partial_observation_never_proves_absence() {
    assert_eq!(statuses(&["USB"], None), [Availability::Unknown]);
    let partial = catalog(&["Dock"], false);
    let observed = statuses(&["USB", "Dock"], Some(&partial));
    assert_eq!(observed, [Availability::Unknown, Availability::Available]);
}
#[test]
fn names_lists_each_present_name_once_in_enumeration_order() {
    assert_eq!(catalog(&["B", "A", "B"], true).names(), ["B", "A"]);
}
