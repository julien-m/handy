// Re-export all audio components
mod attempt;
mod device;
mod recorder;
mod resampler;
mod utils;
mod visualizer;

pub use attempt::{AttemptFailure, OpenLedger, MAX_STALLED_OPENS};
pub use device::{list_input_devices, list_output_devices, CpalDeviceInfo};
pub use recorder::{
    is_microphone_access_denied, is_no_input_device_error, AudioRecorder, LossCallback, VadPolicy,
    OPEN_ATTEMPT_TIMEOUT,
};
pub use resampler::FrameResampler;
pub use utils::{read_wav_samples, save_wav_file, verify_wav_file};
pub use visualizer::AudioVisualiser;
