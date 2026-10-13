//! Raw-preserving merges keep unknown stored fields while typed settings change.
use serde_json::Value;

/// Startup payload: every typed field merged into the raw payload, keeping unknown fields.
pub(super) fn normalized_payload(previous: &Value, normalized: &Value) -> Value {
    let mut preserved = previous.clone();
    for entry in super::descriptor::descriptors() {
        // Startup recovery preserves existing raw secrets even when their typed decoding was invalid.
        if !(entry.secret && previous.get(entry.key).is_some()) {
            merge_preserved(&mut preserved[entry.key], &normalized[entry.key]);
        }
    }
    preserved
}
/// Mutation payload: merge only typed fields whose value changed; a missing store takes the typed payload.
pub(super) fn apply_changed_fields(raw: &mut Value, before: &Value, after: &Value, missing: bool) {
    if missing {
        raw["settings"] = after.clone();
        return;
    }
    if !raw["settings"].is_object() {
        raw["settings"] = serde_json::json!({});
    }
    // Apply only semantic changes to the preserved payload; unrelated raw secrets/future fields remain untouched.
    for entry in super::descriptor::descriptors() {
        if before[entry.key] != after[entry.key] {
            merge_preserved(&mut raw["settings"][entry.key], &after[entry.key]);
        }
    }
}
/// Merge strategy: typed values always win; objects recurse key by key so unknown keys survive.
/// Arrays keep unknown item fields only when typed and stored items are matched by a unique,
/// non-null `id`; any other array (primitives, missing or duplicate ids) is replaced by its typed
/// items, which drops unknown item fields rather than copying them onto an unrelated item.
fn merge_preserved(raw: &mut Value, typed: &Value) {
    match (raw, typed) {
        (Value::Object(raw), Value::Object(typed)) => {
            for (key, value) in typed {
                merge_preserved(raw.entry(key).or_insert(Value::Null), value);
            }
        }
        (Value::Array(raw), Value::Array(typed)) if unique_ids(raw) && unique_ids(typed) => {
            let previous = std::mem::take(raw);
            *raw = typed
                .iter()
                .map(|item| {
                    let mut preserved = previous
                        .iter()
                        .find(|old| old.get("id") == item.get("id"))
                        .cloned()
                        .unwrap_or(Value::Null);
                    merge_preserved(&mut preserved, item);
                    preserved
                })
                .collect();
        }
        (raw, typed) => *raw = typed.clone(),
    }
}
/// True when every item is an object whose non-null `id` occurs exactly once in the array.
fn unique_ids(items: &[Value]) -> bool {
    let mut seen = std::collections::HashSet::new();
    items.iter().all(|item| match item.get("id") {
        Some(id) if !id.is_null() => seen.insert(id.to_string()),
        _ => false,
    })
}
