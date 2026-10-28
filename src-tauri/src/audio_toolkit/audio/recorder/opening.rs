//! One bounded open attempt of the capture stream (feature 001, FR-003). Each attempt owns a
//! fresh stream-error flag and hands the config its device accepted back through the handshake:
//! only the caller publishes them, and only after a successful handshake. An attempt abandoned
//! after its timeout can therefore neither mark the stream that did open as lost (its late
//! backend error stays on its own flag) nor write the recorder's config cache.
use std::error::Error;
use std::sync::{
    atomic::AtomicBool,
    mpsc::{self, Receiver, SyncSender},
    Arc, Mutex, MutexGuard, PoisonError,
};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use cpal::{
    traits::{DeviceTrait, StreamTrait},
    Device, SupportedStreamConfig,
};
use rtrb::Consumer;

use super::{
    open_error, run_consumer, AudioFrameCallback, AudioRecorder, CaptureProcessor,
    CaptureTransportState, Cmd, LevelCallback, LossCallback, VadConfig,
};
use crate::audio_toolkit::audio::attempt::{self, AttemptFailure, AttemptWorker};

/// Config a device accepted, keyed by the device name, to cache once its open is published.
pub(super) type CacheEntry = Option<(String, SupportedStreamConfig)>;

/// What a native open produced on the capture worker.
pub(super) struct OpenedStream<S> {
    /// Keeps the device capturing; created, kept and dropped on the capture worker.
    pub(super) stream: S,
    pub(super) sample_rate: u32,
    pub(super) samples: Consumer<f32>,
    pub(super) cache_entry: CacheEntry,
}

/// Signature shared by the `build_stream` instantiations of every sample format.
type BuildStream = fn(
    &Device,
    &SupportedStreamConfig,
    usize,
    Option<usize>,
    Arc<CaptureTransportState>,
    Arc<AtomicBool>,
) -> Result<(cpal::Stream, Consumer<f32>), cpal::BuildStreamError>;

/// Native open of one device, run on the capture worker.
pub(super) struct NativeOpen {
    pub(super) device: Device,
    pub(super) config_cache: Arc<Mutex<CacheEntry>>,
    pub(super) selected_channel: Option<usize>,
}

impl NativeOpen {
    /// Build and start the input stream; its error callback sets `stream_error`, the flag of this
    /// attempt only.
    pub(super) fn open(
        self,
        transport: Arc<CaptureTransportState>,
        stream_error: Arc<AtomicBool>,
    ) -> Result<OpenedStream<cpal::Stream>, String> {
        let config_started = Instant::now();
        let device_name = self.device.name().unwrap_or_default();
        let (config, was_cached) = self.config_for(&device_name)?;
        log::debug!(
            "mic worker init: fetch_config={:?} (cached={was_cached})",
            config_started.elapsed()
        );
        log_stream_config(&self.device, &config, self.selected_channel);
        let (stream, samples) = self.start(&config, transport, stream_error)?;
        // The device accepted this config; the caller caches it once the open is published, so
        // the next open skips the HAL property queries entirely.
        let cache_entry =
            (!was_cached && !device_name.is_empty()).then(|| (device_name, config.clone()));
        Ok(OpenedStream {
            stream,
            sample_rate: config.sample_rate().0,
            samples,
            cache_entry,
        })
    }

    /// Build the input stream and start it playing.
    fn start(
        &self,
        config: &SupportedStreamConfig,
        transport: Arc<CaptureTransportState>,
        stream_error: Arc<AtomicBool>,
    ) -> Result<(cpal::Stream, Consumer<f32>), String> {
        let build_started = Instant::now();
        let (stream, samples) = self.build(config, transport, stream_error)?;
        let build_elapsed = build_started.elapsed();
        let play_started = Instant::now();
        stream
            .play()
            .map_err(|e| format!("Failed to start microphone stream: {e}"))?;
        log::debug!(
            "mic worker init: build_stream={build_elapsed:?} play={:?}",
            play_started.elapsed()
        );
        Ok((stream, samples))
    }

    /// The cached config of `device_name`, or the device's preferred one; `true` when cached.
    fn config_for(&self, device_name: &str) -> Result<(SupportedStreamConfig, bool), String> {
        let cached = lock_cache(&self.config_cache)
            .as_ref()
            .filter(|(name, _)| !device_name.is_empty() && name == device_name)
            .map(|(_, config)| config.clone());
        match cached {
            Some(config) => Ok((config, true)),
            None => AudioRecorder::get_preferred_config(&self.device)
                .map(|config| (config, false))
                .map_err(|e| format!("Failed to fetch preferred config: {e}")),
        }
    }

    fn build(
        &self,
        config: &SupportedStreamConfig,
        transport: Arc<CaptureTransportState>,
        stream_error: Arc<AtomicBool>,
    ) -> Result<(cpal::Stream, Consumer<f32>), String> {
        let build: BuildStream = match config.sample_format() {
            cpal::SampleFormat::U8 => AudioRecorder::build_stream::<u8>,
            cpal::SampleFormat::I8 => AudioRecorder::build_stream::<i8>,
            cpal::SampleFormat::I16 => AudioRecorder::build_stream::<i16>,
            cpal::SampleFormat::I32 => AudioRecorder::build_stream::<i32>,
            cpal::SampleFormat::F32 => AudioRecorder::build_stream::<f32>,
            sample_format => return Err(format!("Unsupported sample format: {sample_format:?}")),
        };
        let channels = config.channels() as usize;
        build(
            &self.device,
            config,
            channels,
            self.selected_channel,
            transport,
            stream_error,
        )
        .map_err(|e| format!("Failed to build input stream: {e}"))
    }
}

fn log_stream_config(device: &Device, config: &SupportedStreamConfig, selected: Option<usize>) {
    let channels = config.channels() as usize;
    log::info!(
        "Using device: {:?}\nSample rate: {}\nChannels: {}\nFormat: {:?}",
        device.name(),
        config.sample_rate().0,
        channels,
        config.sample_format()
    );
    match selected {
        Some(channel) if channel < channels => {
            log::info!("Using selected input channel: {}", channel + 1)
        }
        Some(channel) => log::warn!(
            "Selected input channel {} is out of range for a {}-channel device; averaging all channels instead",
            channel + 1,
            channels
        ),
        None => log::info!("Averaging all {} input channels", channels),
    }
}

/// The cache even after a panic elsewhere: an entry is replaced whole, so it stays valid.
fn lock_cache(cache: &Mutex<CacheEntry>) -> MutexGuard<'_, CacheEntry> {
    cache.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Recorder ports the consumer of a published stream needs.
struct ConsumerPorts {
    vad: Option<VadConfig>,
    level_cb: Option<LevelCallback>,
    audio_cb: Option<AudioFrameCallback>,
    salvage: Arc<Mutex<Vec<f32>>>,
    loss_cb: Option<LossCallback>,
    stream_error: Arc<AtomicBool>,
}

/// Handshake of an attempt: the config to cache, or the worker's initialization error.
type Handshake = Result<CacheEntry, String>;

impl AudioRecorder {
    /// Run `init` on a capture worker and publish its stream when the handshake arrives within
    /// `timeout`. A timed-out worker is abandoned, never joined: if its native open completes
    /// later it drops the stream itself (see `attempt`), and its flag and config are never
    /// published.
    pub(super) fn open_attempt<S, F>(
        &mut self,
        timeout: Duration,
        init: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        S: 'static,
        F: FnOnce(Arc<CaptureTransportState>, Arc<AtomicBool>) -> Result<OpenedStream<S>, String>
            + Send
            + 'static,
    {
        // No stream is open from here: the recorder's flag no longer describes one.
        self.stream_error = Arc::new(AtomicBool::new(false));
        let (owner, worker_side) = attempt::begin(&self.open_ledger).map_err(open_error)?;
        let stream_error = Arc::new(AtomicBool::new(false));
        let (cmd_tx, cmd_rx) = mpsc::channel::<Cmd>();
        let (init_tx, init_rx) = mpsc::sync_channel::<Handshake>(1);
        let ports = self.consumer_ports(Arc::clone(&stream_error));
        let worker = std::thread::spawn(move || {
            serve_attempt(init, worker_side, &init_tx, cmd_rx, ports);
        });
        match owner.wait(&init_rx, timeout) {
            Ok(cache_entry) => {
                self.publish(cache_entry, stream_error, cmd_tx, worker);
                Ok(())
            }
            Err(failure) => Err(self.fail_attempt(failure, worker)),
        }
    }

    /// Make the attempt's stream, flag and config the recorder's.
    fn publish(
        &mut self,
        cache_entry: CacheEntry,
        stream_error: Arc<AtomicBool>,
        cmd_tx: mpsc::Sender<Cmd>,
        worker: JoinHandle<()>,
    ) {
        if cache_entry.is_some() {
            *lock_cache(&self.config_cache) = cache_entry;
        }
        self.stream_error = stream_error;
        self.cmd_tx = Some(cmd_tx);
        self.worker_handle = Some(worker);
    }

    /// Error of a failed attempt, whose worker is joined unless it was abandoned.
    fn fail_attempt(&mut self, failure: AttemptFailure, worker: JoinHandle<()>) -> Box<dyn Error> {
        // A failed open may mean the cached config went stale (device re-plugged, rate/format
        // changed in the OS): the next attempt re-queries the device.
        *lock_cache(&self.config_cache) = None;
        // An abandoned worker may be blocked in a native call: it is never joined.
        if failure != AttemptFailure::TimedOut {
            let _ = worker.join();
        }
        open_error(failure)
    }

    fn consumer_ports(&self, stream_error: Arc<AtomicBool>) -> ConsumerPorts {
        ConsumerPorts {
            vad: self.vad.clone(),
            level_cb: self.level_cb.clone(),
            audio_cb: self.audio_cb.clone(),
            salvage: Arc::clone(&self.salvage),
            loss_cb: self.loss_cb.clone(),
            stream_error,
        }
    }
}

/// Capture worker of one attempt: native open, handshake, then the consumer of a published
/// stream.
fn serve_attempt<S, F>(
    init: F,
    attempt: AttemptWorker,
    init_tx: &SyncSender<Handshake>,
    cmd_rx: Receiver<Cmd>,
    ports: ConsumerPorts,
) where
    F: FnOnce(Arc<CaptureTransportState>, Arc<AtomicBool>) -> Result<OpenedStream<S>, String>,
{
    let transport = Arc::new(CaptureTransportState::default());
    let opened = init(Arc::clone(&transport), Arc::clone(&ports.stream_error));
    if let Some(opened) = handshake(opened, attempt, init_tx) {
        run_published(opened, cmd_rx, transport, ports);
    }
}

/// Worker side of the handshake: the opened stream when the caller still wants it. Order
/// matters: a stream the caller gave up on is dropped before `attempt` ends, so the device is
/// released before the attempt stops counting as stalled.
fn handshake<S>(
    opened: Result<OpenedStream<S>, String>,
    attempt: AttemptWorker,
    init_tx: &SyncSender<Handshake>,
) -> Option<OpenedStream<S>> {
    let mut opened = match opened {
        Ok(opened) => opened,
        Err(message) => {
            log::error!("{message}");
            let _ = init_tx.send(Err(message));
            return None;
        }
    };
    if !attempt.publish() {
        // The caller gave up on this attempt: release the device at once.
        drop(opened);
        return None;
    }
    let _ = init_tx.send(Ok(opened.cache_entry.take()));
    Some(opened)
}

/// Run the consumer of a published stream until the recorder closes it, then release it.
fn run_published<S>(
    opened: OpenedStream<S>,
    cmd_rx: Receiver<Cmd>,
    transport: Arc<CaptureTransportState>,
    ports: ConsumerPorts,
) {
    // Timestamp for the play()-returned -> first-samples gap the handshake can't see.
    let processor = CaptureProcessor::new(
        opened.sample_rate,
        ports.vad,
        ports.level_cb,
        ports.audio_cb,
        Instant::now(),
    )
    .with_stream_ports(ports.salvage, ports.loss_cb);
    run_consumer(
        processor,
        opened.samples,
        cmd_rx,
        transport,
        ports.stream_error,
    );
    drop(opened.stream);
}
