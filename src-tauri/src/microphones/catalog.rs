//! One observation of the present input devices, grouped by exact name: availability of each
//! configured choice and the lookup used to open one. Enumeration indices and device handles are
//! valid only inside one observation and are never persisted.
use super::types::Availability;
use serde::Serialize;

/// One present device of an observation.
#[derive(Clone, Debug)]
pub struct CatalogEntry<D> {
    /// Exact device name, as the backend reports it.
    pub name: String,
    /// Backend handle used to open this device.
    pub device: D,
}

/// The present devices of one enumeration.
#[derive(Clone, Debug)]
pub struct Catalog<D> {
    entries: Vec<CatalogEntry<D>>,
    /// False when some device names could not be read: absence is then not proven.
    complete: bool,
}

/// Result of looking a configured name up in a catalog.
#[derive(Debug, PartialEq)]
pub enum Lookup<'a, D> {
    /// No present device has this name.
    Missing,
    /// Several present devices have this name.
    Ambiguous,
    /// Exactly one device has this name.
    Unique(&'a D),
}

/// Availability of one configured choice.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, specta::Type)]
pub struct ChoiceStatus {
    /// Configured exact name.
    pub name: String,
    /// Observed status.
    pub status: Availability,
}

impl<D> Catalog<D> {
    /// Catalog of a complete enumeration.
    pub fn new(entries: Vec<CatalogEntry<D>>) -> Self {
        Self {
            entries,
            complete: true,
        }
    }
    /// Catalog of an enumeration in which some device names could not be read.
    pub fn partial(entries: Vec<CatalogEntry<D>>) -> Self {
        Self {
            entries,
            complete: false,
        }
    }
    /// The device `name` designates, without any arbitrary choice among same-named devices.
    pub fn lookup(&self, name: &str) -> Lookup<'_, D> {
        let mut matching = self.entries.iter().filter(|entry| entry.name == name);
        match (matching.next(), matching.next()) {
            (None, _) => Lookup::Missing,
            (Some(entry), None) => Lookup::Unique(&entry.device),
            (Some(_), Some(_)) => Lookup::Ambiguous,
        }
    }
    /// Present device names in enumeration order, each name once.
    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        for entry in &self.entries {
            if !names.contains(&entry.name) {
                names.push(entry.name.clone());
            }
        }
        names
    }
}

// @spec FR-002: Availability separate from preferences — .specs/features/001-microphone-priority/spec.md#fr-002
/// Status of every configured choice, in saved order. Without a catalog (failed or timed-out
/// enumeration) every status is unknown, and so is a name missing from a partial catalog: a
/// failed observation never proves absence.
pub fn availability<D>(configured: &[String], catalog: Option<&Catalog<D>>) -> Vec<ChoiceStatus> {
    configured
        .iter()
        .map(|name| ChoiceStatus {
            name: name.clone(),
            status: catalog.map_or(Availability::Unknown, |catalog| status(catalog, name)),
        })
        .collect()
}
fn status<D>(catalog: &Catalog<D>, name: &str) -> Availability {
    match catalog.lookup(name) {
        Lookup::Missing if catalog.complete => Availability::Unavailable,
        Lookup::Missing => Availability::Unknown,
        Lookup::Ambiguous => Availability::Ambiguous,
        Lookup::Unique(_) => Availability::Available,
    }
}

#[cfg(test)]
#[path = "catalog_tests.rs"]
mod tests;
