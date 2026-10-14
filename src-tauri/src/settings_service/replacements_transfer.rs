//! CLI-process side of replacement operations larger than one channel message (1 MiB). A list,
//! an export or a `get` of the rules reads them in pages and refuses pages of different revisions
//! (it starts again, so a concurrent change never mixes two rule sets); a `get` of every setting
//! reads the other settings in one request and keeps them only when the pages have the same
//! revision. An import whose input does not fit one request is staged in ordered chunks, then
//! imported by token in one atomic commit. Every request of one operation shares the operation's
//! single deadline, and the caller still gets exactly one reply.
use super::{
    domains::replacements::{MAX_PAGE_BYTES, RULES_KEY},
    errors::{SettingsError, SettingsErrorCode},
    ipc,
    operations::Reply,
    protocol::{Revision, SettingsOperation},
    route::route_until,
};
use crate::word_replacements::{
    exchange::serialize_exchange, model::ReplacementRule, transfer::sha256_hex,
};
use serde_json::{json, Value};
use std::time::Instant;

/// Attempts at a consistent paged read before reporting the settings as busy: each concurrent
/// change restarts the read, and three changes within one five-second operation mean the rules
/// are being rewritten continuously.
const READ_ATTEMPTS: usize = 3;

/// Route `operation`; `get` of every setting or of the rules, replacement lists, exports and
/// oversized imports use the bounded transfer.
pub fn route_operation(operation: &SettingsOperation) -> Reply {
    let deadline = Instant::now() + ipc::DEADLINE;
    match operation {
        SettingsOperation::Get { key: None } => get_all(deadline),
        SettingsOperation::Get { key: Some(key) } if key == RULES_KEY => get_rules(deadline),
        SettingsOperation::Domain {
            domain,
            action,
            arguments,
            ..
        } if domain == "replacements" => match (action.as_str(), arguments.len()) {
            ("list", 0) => list(deadline, false),
            ("export", 0) => list(deadline, true),
            ("import", 1) if !ipc::fits_frame(operation) => staged_import(operation, deadline),
            _ => route_until(operation, deadline),
        },
        _ => route_until(operation, deadline),
    }
}

/// The whole list (`{rules}`) or export (`{content, count}`) of one revision.
fn list(deadline: Instant, export: bool) -> Reply {
    let (revision, rules, warning) = match consistent_pages(deadline) {
        Ok(read) => read,
        Err(failure) => return failure,
    };
    let data = if export {
        match serde_json::from_value::<Vec<ReplacementRule>>(Value::Array(rules)) {
            Ok(rules) => json!({ "content": serialize_exchange(&rules), "count": rules.len() }),
            Err(_) => return Reply::failure(SettingsError::new(SettingsErrorCode::InternalError)),
        }
    } else {
        json!({ "rules": rules })
    };
    Reply::success(revision, data, warning)
}

/// `get word_replacements`: the rules of one revision, in the shape of any one-key `get`.
fn get_rules(deadline: Instant) -> Reply {
    match consistent_pages(deadline) {
        Ok((revision, rules, warning)) => {
            Reply::success(revision, json!({ RULES_KEY: rules }), warning)
        }
        Err(failure) => failure,
    }
}

/// `get` of every setting: the other settings and the rule pages, kept only when they come from
/// one revision; a change between them starts again (see `READ_ATTEMPTS`).
fn get_all(deadline: Instant) -> Reply {
    for _ in 0..READ_ATTEMPTS {
        let head = route_until(&private_step("read-settings", Vec::new(), None), deadline);
        let Some(revision) = head.envelope()["revision"].as_u64() else {
            return head;
        };
        let (pages_revision, rules, warning) = match consistent_pages(deadline) {
            Ok(read) => read,
            Err(failure) => return failure,
        };
        // Loop exit: both reads saw the same revision, so they describe one settings state.
        if pages_revision == Revision(revision) {
            let mut data = head.envelope()["data"].clone();
            data[RULES_KEY] = Value::Array(rules);
            return Reply::success(pages_revision, data, warning || head.warning());
        }
    }
    Reply::failure(SettingsError::new(SettingsErrorCode::Unavailable))
}

/// All rule pages of one revision, after at most `READ_ATTEMPTS` reads.
fn consistent_pages(deadline: Instant) -> Result<Pages, Reply> {
    match (0..READ_ATTEMPTS).find_map(|_| read_pages(deadline).transpose()) {
        Some(read) => read,
        None => Err(Reply::failure(SettingsError::new(
            SettingsErrorCode::Unavailable,
        ))),
    }
}

type Pages = (Revision, Vec<Value>, bool);

/// All pages of one revision; `Ok(None)` when the revision moved between pages.
fn read_pages(deadline: Instant) -> Result<Option<Pages>, Reply> {
    let (mut rules, mut start, mut pinned, mut warning) = (Vec::new(), 0, None, false);
    loop {
        let reply = route_until(&page_request(start), deadline);
        let envelope = reply.envelope();
        let revision = envelope["revision"].as_u64().ok_or_else(|| reply.clone())?;
        if pinned.is_some_and(|pinned| pinned != revision) {
            return Ok(None);
        }
        pinned = Some(revision);
        warning |= reply.warning();
        let page = envelope["data"]["rules"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        rules.extend(page);
        match envelope["data"]["next"].as_u64() {
            // A page always holds at least one rule, so `next` only moves forward.
            Some(next) if next as usize > start => start = next as usize,
            Some(_) => {
                return Err(Reply::failure(SettingsError::new(
                    SettingsErrorCode::InternalError,
                )))
            }
            None => return Ok(Some((Revision(revision), rules, warning))),
        }
    }
}

fn page_request(start: usize) -> SettingsOperation {
    private_step("read-page", vec![start.to_string()], None)
}

/// A private transfer step of the replacements domain (never a dry run or a revision check).
fn private_step(action: &str, arguments: Vec<String>, input: Option<String>) -> SettingsOperation {
    SettingsOperation::Domain {
        domain: "replacements".to_owned(),
        action: action.to_owned(),
        arguments,
        input,
        dry_run: false,
        expected_revision: None,
    }
}

/// Stage the input of `operation` in chunks, then import it by token with the operation's mode,
/// dry run and expected revision. A refused step is the reply; nothing is committed before the
/// final import.
fn staged_import(operation: &SettingsOperation, deadline: Instant) -> Reply {
    let SettingsOperation::Domain {
        arguments,
        input: Some(input),
        dry_run,
        expected_revision,
        ..
    } = operation
    else {
        return Reply::failure(SettingsError::new(SettingsErrorCode::UsageError));
    };
    let token = match stage(input, deadline) {
        Ok(token) => token,
        Err(refused) => return refused,
    };
    let import = SettingsOperation::Domain {
        domain: "replacements".to_owned(),
        action: "import".to_owned(),
        arguments: vec![arguments[0].clone(), token],
        input: None,
        dry_run: *dry_run,
        expected_revision: *expected_revision,
    };
    route_until(&import, deadline)
}

/// Declare `input`, send its chunks in order, and return the staging token.
fn stage(input: &str, deadline: Instant) -> Result<String, Reply> {
    let declare = vec![input.len().to_string(), sha256_hex(input)];
    let begun = route_until(&private_step("stage-begin", declare, None), deadline);
    let token = begun.envelope()["data"]["token"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| begun.clone())?;
    for (index, chunk) in chunks(input).into_iter().enumerate() {
        let arguments = vec![token.clone(), index.to_string()];
        let step = private_step("stage-append", arguments, Some(chunk.to_owned()));
        let appended = route_until(&step, deadline);
        if appended.exit_code() != 0 {
            return Err(appended);
        }
    }
    Ok(token)
}

/// Pieces of `input` whose JSON-escaped size stays within `MAX_PAGE_BYTES`, split on character
/// boundaries.
fn chunks(input: &str) -> Vec<&str> {
    let (mut pieces, mut start, mut escaped) = (Vec::new(), 0, 0);
    for (offset, character) in input.char_indices() {
        let size = escaped_size(character);
        if escaped + size > MAX_PAGE_BYTES {
            pieces.push(&input[start..offset]);
            (start, escaped) = (offset, 0);
        }
        escaped += size;
    }
    pieces.push(&input[start..]);
    pieces
}

/// Bytes `character` takes in a JSON string (serde_json escaping).
fn escaped_size(character: char) -> usize {
    match character {
        '"' | '\\' | '\n' | '\r' | '\t' | '\u{8}' | '\u{c}' => 2,
        control if control < ' ' => 6,
        other => other.len_utf8(),
    }
}

#[cfg(test)]
mod tests {
    use super::{chunks, escaped_size};

    #[test]
    fn chunks_cover_the_input_in_order_within_the_escaped_bound() {
        let input = format!("{}é\"{}", "\u{1}".repeat(100_000), "a;b\n".repeat(200_000));
        let pieces = chunks(&input);
        assert!(pieces.len() > 1);
        assert_eq!(pieces.concat(), input);
        for piece in pieces {
            let escaped = serde_json::to_string(piece).map_or(usize::MAX, |text| text.len() - 2);
            assert_eq!(escaped, piece.chars().map(escaped_size).sum::<usize>());
            assert!(escaped <= super::MAX_PAGE_BYTES);
        }
    }
}
