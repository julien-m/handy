//! Channel boundary tests: frame bounds, request binding, unknown outcomes and client-side peer
//! authentication before any application byte.
use super::*;
use serde_json::json;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn failure_code(reply: &Reply) -> Value {
    reply.envelope()["error"]["code"].clone()
}

#[test]
fn frames_round_trip_and_oversized_lengths_are_refused_before_allocation() -> TestResult {
    let mut buffer = Vec::new();
    write_frame(&mut buffer, b"{}")?;
    assert_eq!(read_frame(&mut buffer.as_slice())?, b"{}");
    let oversized = ((MAX_FRAME_BYTES + 1) as u32).to_be_bytes();
    let error = read_frame(&mut oversized.as_slice())
        .err()
        .ok_or("accepted")?;
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(write_frame(&mut Vec::new(), &vec![0; MAX_FRAME_BYTES + 1]).is_err());
    Ok(())
}

#[cfg(unix)]
#[test]
fn a_request_for_another_store_or_a_malformed_frame_gets_a_fixed_refusal() -> TestResult {
    let directory = tempfile::tempdir()?;
    let store = directory.path().join("settings_store.json");
    let live = super::super::live::LiveSettings::start_offline(
        super::super::store::SettingsRepository::new(store),
        Box::new(super::super::ownership::LegacyPathProbe::new(
            directory.path().join("x"),
        )),
    )?;
    let request = request("not-this-store", SettingsOperation::Keys);
    let reply = answer(&serde_json::to_vec(&request)?, "this-store", &live);
    assert_eq!(
        (reply.request_id, &reply.envelope["error"]["code"]),
        (request.request_id, &json!("unavailable"))
    );
    let malformed = answer(b"{\"secret\":\"sk-123\"}", "this-store", &live);
    assert_eq!(
        (
            malformed.request_id.as_str(),
            &malformed.envelope["error"]["code"]
        ),
        ("", &json!("usage_error"))
    );
    assert!(!malformed.envelope.to_string().contains("sk-123"));
    Ok(())
}

#[cfg(unix)]
#[test]
fn silence_or_a_reply_to_another_request_is_an_unknown_outcome() -> TestResult {
    use std::os::unix::net::UnixStream;
    let (client, _silent_server) = UnixStream::pair()?;
    let request = request("store", SettingsOperation::Keys);
    let reply = exchange(
        Box::new(client),
        &request,
        Instant::now() + Duration::from_millis(100),
    );
    assert_eq!(
        (
            failure_code(&reply),
            &reply.envelope()["error"]["details"]["outcome"]
        ),
        (json!("unavailable"), &json!("unknown"))
    );
    let (client, mut server) = UnixStream::pair()?;
    let other = serde_json::to_vec(&json!({"request_id": "other", "warning": false,
        "envelope": {"ok": true, "revision": 1, "data": {}}}))?;
    let responder = std::thread::spawn(move || {
        read_frame(&mut server).and_then(|_| write_frame(&mut server, &other))
    });
    let reply = exchange(Box::new(client), &request, Instant::now() + DEADLINE);
    responder.join().map_err(|_| "responder panicked")??;
    assert_eq!(
        reply.envelope()["error"]["details"]["outcome"],
        json!("unknown")
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn the_client_refuses_an_unexpected_server_owner_before_sending_any_byte() -> TestResult {
    let directory = tempfile::tempdir()?;
    let identity = store_identity(&directory.path().join("settings_store.json"))?;
    let endpoint = endpoint(&identity).map_err(|error| error.message())?;
    let listener = endpoint.listen()?;
    let refused = endpoint.connect_as(super::super::platform_unix::current_uid().wrapping_add(1));
    assert!(matches!(refused, Connection::Refused));
    drop(refused);
    let mut accepted = listener.accept()?;
    let mut received = Vec::new();
    accepted.read_to_end(&mut received)?;
    assert!(
        received.is_empty(),
        "no application byte before authentication"
    );
    listener.close()?;
    assert!(!endpoint.path().exists(), "closing removes the name");
    Ok(())
}
