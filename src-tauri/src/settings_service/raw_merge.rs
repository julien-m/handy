//! Raw-preserving merges keep unknown stored fields while typed settings change.
use serde_json::Value;

/// Normalized payload: every typed field merged into the raw payload, keeping unknown fields.
///
/// Used by startup and by every committed mutation so the stored document always carries the
/// complete normalization (including migration markers such as `settings_schema_version`,
/// `overlay_style` or `chinese_script`); otherwise the next read would replay migrations over it.
pub(super) fn normalized_payload(previous: &Value, normalized: &Value) -> Value {
    let mut preserved = if previous.is_object() {
        previous.clone()
    } else {
        Value::Object(serde_json::Map::new())
    };
    for entry in super::descriptor::descriptors() {
        // Recovery preserves existing raw secrets even when their typed decoding was invalid.
        if !(entry.secret && previous.get(entry.key).is_some()) {
            merge_preserved(&mut preserved[entry.key], &normalized[entry.key]);
        }
    }
    preserved
}
/// Mutation delta: apply to `payload` only the typed fields whose value changed from `before` to
/// `after`, so unrelated raw secrets and future fields stay untouched.
pub(super) fn apply_changed_fields(payload: &mut Value, before: &Value, after: &Value) {
    for entry in super::descriptor::descriptors() {
        merge_change(
            payload,
            entry.key,
            before.get(entry.key),
            after.get(entry.key),
        );
    }
}
/// Diff strategy: a key the typed value lost (map entry deleted, field no longer serialized) is
/// removed from the raw object; a changed object recurses key by key so unknown keys and keys
/// that only exist raw survive; any other changed value is merged by `merge_preserved`.
fn merge_change(raw: &mut Value, key: &str, before: Option<&Value>, after: Option<&Value>) {
    let Some(object) = raw.as_object_mut() else {
        return;
    };
    match (before, after) {
        (Some(_), None) => {
            object.remove(key);
        }
        (Some(Value::Object(old)), Some(Value::Object(new))) if old != new => {
            let target = object.entry(key).or_insert(Value::Null);
            if !target.is_object() {
                // A non-object raw value was never decodable; start from the typed previous value.
                *target = Value::Object(old.clone());
            }
            let keys = old.keys().chain(new.keys()).cloned().collect::<Vec<_>>();
            for child in keys {
                merge_change(target, &child, old.get(&child), new.get(&child));
            }
        }
        (old, Some(new)) if old != Some(new) => {
            merge_preserved(object.entry(key).or_insert(Value::Null), new);
        }
        _ => {}
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
#[cfg(test)]
mod tests {
    use super::apply_changed_fields;
    use serde_json::json;

    #[test]
    fn deleted_map_entry_is_removed_while_raw_only_entries_and_unknown_fields_survive() {
        let mut payload = json!({
            "bindings": {
                "kept": {"id": "kept", "current_binding": "a", "unknown": "CANARY_FIELD"},
                "deleted": {"id": "deleted", "current_binding": "b"},
                "raw_only": {"unknown": "CANARY_RAW"}
            },
            "post_process_models": {"openai": "old", "groq": "removed"}
        });
        let before = json!({
            "bindings": {
                "kept": {"id": "kept", "current_binding": "a"},
                "deleted": {"id": "deleted", "current_binding": "b"}
            },
            "post_process_models": {"openai": "old", "groq": "removed"}
        });
        let after = json!({
            "bindings": {"kept": {"id": "kept", "current_binding": "c"}},
            "post_process_models": {"openai": "new"}
        });
        apply_changed_fields(&mut payload, &before, &after);
        assert_eq!(
            payload,
            json!({
                "bindings": {
                    "kept": {"id": "kept", "current_binding": "c", "unknown": "CANARY_FIELD"},
                    "raw_only": {"unknown": "CANARY_RAW"}
                },
                "post_process_models": {"openai": "new"}
            })
        );
    }
    #[test]
    fn unchanged_fields_keep_their_raw_bytes() {
        let mut payload = json!({"audio_feedback": true, "post_process_api_keys": {"x": "RAW"}});
        let before = json!({"audio_feedback": true, "post_process_api_keys": {"x": "typed"}});
        let after = json!({"audio_feedback": false, "post_process_api_keys": {"x": "typed"}});
        apply_changed_fields(&mut payload, &before, &after);
        assert_eq!(
            payload,
            json!({"audio_feedback": false, "post_process_api_keys": {"x": "RAW"}})
        );
    }
}
