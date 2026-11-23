//! One dictation streamed to ChatGPT over a blocking WebSocket owned by the
//! stream worker thread. Reads use a short socket timeout so the worker can
//! interleave sending audio and draining server events on one thread.

use std::io::ErrorKind;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

use log::{debug, info, warn};
use tungstenite::client::IntoClientRequest;
use tungstenite::handshake::client::Request;
use tungstenite::http::HeaderValue;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::HandshakeError;
use tungstenite::{Message, WebSocket};

use super::auth::SignIn;
use super::protocol::{self, ServerEvent};
use super::transcript::Transcript;
use crate::transcription_engine::{LiveSession, LiveSessionError, LiveText};

/// Read timeout while streaming: short, so polling never delays audio sends.
const POLL_READ_TIMEOUT: Duration = Duration::from_millis(5);
/// Read timeout while waiting for the final text after `session.close`.
const FINISH_READ_TIMEOUT: Duration = Duration::from_millis(50);

/// Bounds of every network step, so an unresponsive network makes the caller
/// fall back to the local model instead of hanging the end of a dictation.
#[derive(Debug, Clone, Copy)]
struct NetworkLimits {
    /// TCP connection to one resolved address.
    connect: Duration,
    /// TLS and WebSocket upgrade, per read or write.
    handshake: Duration,
    /// One audio or control message while streaming.
    write: Duration,
    /// From the start of `connect` to `session.started` (the desktop app also
    /// gives up after 10 s).
    start: Duration,
    /// From `session.close` to the final text (the desktop app waits 8 s).
    finish: Duration,
}

const NETWORK_LIMITS: NetworkLimits = NetworkLimits {
    connect: Duration::from_secs(5),
    handshake: Duration::from_secs(10),
    write: Duration::from_secs(2),
    start: Duration::from_secs(10),
    finish: Duration::from_secs(8),
};

type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

/// Live ChatGPT dictation session (see [`LiveSession`]).
pub(super) struct ChatGptSession {
    socket: Socket,
    transcript: Transcript,
    session_id: String,
    limits: NetworkLimits,
    is_finishing: bool,
    is_closed: bool,
}

impl ChatGptSession {
    /// Connect with `sign_in`, start the session and wait for the server to
    /// accept it; every step is bounded by [`NETWORK_LIMITS`].
    pub(super) fn connect(
        sign_in: &SignIn,
        language: Option<&str>,
    ) -> Result<Self, LiveSessionError> {
        Self::connect_to(protocol::STREAM_URL, sign_in, language, NETWORK_LIMITS)
    }

    fn connect_to(
        url: &str,
        sign_in: &SignIn,
        language: Option<&str>,
        limits: NetworkLimits,
    ) -> Result<Self, LiveSessionError> {
        let started = Instant::now();
        let socket = open_socket(build_request(url, sign_in)?, limits)?;
        let mut session = Self {
            socket,
            transcript: Transcript::default(),
            session_id: String::new(),
            limits,
            is_finishing: false,
            is_closed: false,
        };
        let client_id = uuid::Uuid::new_v4().to_string();
        let attempt_id = uuid::Uuid::new_v4().to_string();
        session.send(protocol::session_start(&client_id, &attempt_id, language))?;
        while session.session_id.is_empty() {
            if started.elapsed() > session.limits.start {
                return Err(LiveSessionError::Timeout("starting the session"));
            }
            session.drain_events()?;
        }
        info!(
            "ChatGPT engine: session {} started in {} ms",
            session.session_id,
            started.elapsed().as_millis()
        );
        Ok(session)
    }

    fn send(&mut self, text: String) -> Result<(), LiveSessionError> {
        self.socket
            .send(Message::text(text))
            .map_err(|e| LiveSessionError::Stream(e.to_string()))
    }

    /// Apply every pending server event; returns whether the transcript changed.
    fn drain_events(&mut self) -> Result<bool, LiveSessionError> {
        let mut has_changed = false;
        // Exits when no frame is pending (read timeout) or once the server
        // reports the session closed: nothing useful follows that event.
        loop {
            let message = match self.socket.read() {
                Ok(message) => message,
                Err(tungstenite::Error::Io(e))
                    if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) =>
                {
                    break;
                }
                Err(e) => return Err(LiveSessionError::Stream(e.to_string())),
            };
            let Message::Text(text) = message else {
                continue;
            };
            match protocol::parse_event(text.as_str()) {
                Some(event) => has_changed |= self.apply(event)?,
                None => warn!("ChatGPT engine: unreadable server event ignored"),
            }
            if self.is_closed {
                break;
            }
        }
        Ok(has_changed)
    }

    /// Apply one event; returns whether the transcript changed.
    fn apply(&mut self, event: ServerEvent) -> Result<bool, LiveSessionError> {
        match event {
            ServerEvent::SessionStarted { session } => self.session_id = session.session_id,
            ServerEvent::SessionUpdated { session } => {
                self.is_closed = session.status == protocol::SESSION_CLOSED;
                // A close Handy did not request (session lifetime, server
                // shutdown) may have cut the text: report it so the local
                // model transcribes the whole recording instead.
                if self.is_closed && !self.is_finishing {
                    return Err(LiveSessionError::Stream(
                        "the server closed the session before the end of the dictation".into(),
                    ));
                }
            }
            ServerEvent::Segment {
                utterance_id,
                revision,
                text,
            } => {
                self.transcript.segment(&utterance_id, revision, &text);
                return Ok(true);
            }
            ServerEvent::Final {
                utterance_id,
                revision,
                text,
            } => {
                self.transcript.finalize(&utterance_id, revision, &text);
                return Ok(true);
            }
            ServerEvent::Failed { error } | ServerEvent::Error { fatal: true, error } => {
                return Err(LiveSessionError::Stream(format!(
                    "{}: {}",
                    error.code, error.message
                )));
            }
            ServerEvent::Error { error, .. } => {
                warn!(
                    "ChatGPT engine: non-fatal {}: {}",
                    error.code, error.message
                );
            }
            ServerEvent::Other => {}
        }
        Ok(false)
    }
}

impl LiveSession for ChatGptSession {
    fn send_audio(&mut self, samples: &[f32]) -> Result<(), LiveSessionError> {
        self.send(protocol::audio_append(samples))
    }

    fn poll(&mut self) -> Result<Option<LiveText>, LiveSessionError> {
        Ok(self.drain_events()?.then(|| self.transcript.live()))
    }

    fn finish(mut self: Box<Self>) -> Result<String, LiveSessionError> {
        let started = Instant::now();
        self.is_finishing = true;
        self.send(protocol::session_close())?;
        set_read_timeout(&mut self.socket, FINISH_READ_TIMEOUT)?;
        while !self.is_closed {
            if started.elapsed() > self.limits.finish {
                return Err(LiveSessionError::Timeout("waiting for the final text"));
            }
            self.drain_events()?;
        }
        // Best effort: the server already closed the session; a failed close
        // handshake loses nothing.
        let _ = self.socket.close(None);
        let text = self.transcript.text();
        info!(
            "ChatGPT engine: session {} finalized in {} ms ({} chars)",
            self.session_id,
            started.elapsed().as_millis(),
            text.chars().count()
        );
        Ok(text)
    }
}

/// Upgrade request carrying the sign-in as the desktop app does: bearer token,
/// ChatGPT account and dictation subprotocols.
fn build_request(url: &str, sign_in: &SignIn) -> Result<Request, LiveSessionError> {
    let mut request = url
        .into_client_request()
        .map_err(|e| LiveSessionError::Stream(e.to_string()))?;
    let header = |value: &str| {
        HeaderValue::from_str(value).map_err(|_| {
            LiveSessionError::Unavailable("sign-in contains invalid characters".into())
        })
    };
    let headers = request.headers_mut();
    headers.insert(
        "Authorization",
        header(&format!("Bearer {}", sign_in.access_token()))?,
    );
    if let Some(account) = sign_in.account_id() {
        headers.insert("ChatGPT-Account-Id", header(account)?);
    }
    headers.insert("Sec-WebSocket-Protocol", header(protocol::SUBPROTOCOLS)?);
    Ok(request)
}

/// Open the WebSocket with bounded DNS-resolved TCP connection, TLS and
/// upgrade; leaves the socket with the streaming read and write timeouts.
fn open_socket(request: Request, limits: NetworkLimits) -> Result<Socket, LiveSessionError> {
    let uri = request.uri();
    let host = uri
        .host()
        .ok_or_else(|| LiveSessionError::Stream("dictation URL has no host".into()))?
        .to_owned();
    let default_port = if uri.scheme_str() == Some("wss") {
        443
    } else {
        80
    };
    let port = uri.port_u16().unwrap_or(default_port);
    // Name resolution uses the system resolver and its own timeout; the
    // connection, TLS and upgrade steps below are bounded by `limits`.
    let addresses = (host.as_str(), port)
        .to_socket_addrs()
        .map_err(|e| LiveSessionError::Stream(format!("cannot resolve {host}: {}", e.kind())))?;
    let stream = connect_first(addresses, limits.connect)?;
    for timeout in [
        stream.set_read_timeout(Some(limits.handshake)),
        stream.set_write_timeout(Some(limits.handshake)),
    ] {
        timeout.map_err(|e| LiveSessionError::Stream(e.to_string()))?;
    }
    let (mut socket, response) = tungstenite::client_tls(request, stream).map_err(|e| match e {
        HandshakeError::Failure(error) => handshake_failure(error),
        HandshakeError::Interrupted(_) => LiveSessionError::Timeout("opening the connection"),
    })?;
    debug!("ChatGPT engine: connected (HTTP {})", response.status());
    set_read_timeout(&mut socket, POLL_READ_TIMEOUT)?;
    socket_stream(&mut socket)?
        .set_write_timeout(Some(limits.write))
        .map_err(|e| LiveSessionError::Stream(e.to_string()))?;
    Ok(socket)
}

/// Connect to the first reachable address, each attempt bounded by `timeout`.
fn connect_first(
    addresses: impl Iterator<Item = std::net::SocketAddr>,
    timeout: Duration,
) -> Result<TcpStream, LiveSessionError> {
    let mut last_error = LiveSessionError::Stream("no address for the dictation service".into());
    // Stops at the first address that accepts the connection.
    for address in addresses {
        match TcpStream::connect_timeout(&address, timeout) {
            Ok(stream) => return Ok(stream),
            Err(e) if e.kind() == ErrorKind::TimedOut => {
                last_error = LiveSessionError::Timeout("connecting to the dictation service")
            }
            Err(e) => {
                last_error = LiveSessionError::Stream(format!("cannot connect: {}", e.kind()))
            }
        }
    }
    Err(last_error)
}

fn handshake_failure(error: tungstenite::Error) -> LiveSessionError {
    match error {
        tungstenite::Error::Http(refusal) => {
            LiveSessionError::Stream(format!("connection refused with HTTP {}", refusal.status()))
        }
        tungstenite::Error::Io(e)
            if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) =>
        {
            LiveSessionError::Timeout("opening the connection")
        }
        other => LiveSessionError::Stream(other.to_string()),
    }
}

fn socket_stream(socket: &mut Socket) -> Result<&mut TcpStream, LiveSessionError> {
    match socket.get_mut() {
        MaybeTlsStream::Plain(stream) => Ok(stream),
        MaybeTlsStream::NativeTls(stream) => Ok(stream.get_mut()),
        _ => Err(LiveSessionError::Stream("unsupported TLS backend".into())),
    }
}

fn set_read_timeout(socket: &mut Socket, timeout: Duration) -> Result<(), LiveSessionError> {
    socket_stream(socket)?
        .set_read_timeout(Some(timeout))
        .map_err(|e| LiveSessionError::Stream(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    const SHORT_LIMITS: NetworkLimits = NetworkLimits {
        connect: Duration::from_millis(300),
        handshake: Duration::from_millis(300),
        write: Duration::from_millis(300),
        start: Duration::from_millis(600),
        finish: Duration::from_millis(600),
    };

    #[test]
    fn connect_to_silent_server_times_out_instead_of_hanging() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("ws://{}/", listener.local_addr().unwrap());
        // Accepts the connection and never answers the upgrade.
        let server = std::thread::spawn(move || {
            let (_stream, _) = listener.accept().unwrap();
            std::thread::sleep(Duration::from_secs(2));
        });
        let started = Instant::now();
        let result = ChatGptSession::connect_to(
            &url,
            &super::super::auth::test_sign_in(),
            None,
            SHORT_LIMITS,
        );
        assert!(matches!(result, Err(LiveSessionError::Timeout(_))));
        assert!(started.elapsed() < Duration::from_secs(2));
        server.join().unwrap();
    }

    #[test]
    fn connect_to_closed_port_fails_fast() {
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let url = format!("ws://127.0.0.1:{port}/");
        let result = ChatGptSession::connect_to(
            &url,
            &super::super::auth::test_sign_in(),
            None,
            SHORT_LIMITS,
        );
        assert!(result.is_err());
    }
}
