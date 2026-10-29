//! Explicit finite key classification; new stored fields require a reviewed entry.

use super::descriptor::{NumericRange, RuntimeEffect, SettingDescriptor, SettingType};

// Rows keep all policy dimensions together; no classification derives from names/types at runtime.
macro_rules! inventory {
 ($($(#[$gate:meta])? $key:literal => ($ty:ident,$nullable:literal,$range:expr,$values:expr,$readonly:literal,$secret:literal,$owner:literal,$effect:ident);)*) => {
  pub(super) const DESCRIPTORS:&[SettingDescriptor]=&[$($(#[$gate])? SettingDescriptor {key:$key,value_type:SettingType::$ty,nullable:$nullable,range:$range,allowed_values:$values,read_only:$readonly,secret:$secret,owner_domain:$owner,runtime_effect:RuntimeEffect::$effect}),*];
 };
}
// Mutation bounds match current UI controls (hold threshold, volume, history,
// paste delays and recording buffer); increments are suggestions, not constraints.
const fn range(min: f64, max: f64) -> Option<NumericRange> {
    Some(NumericRange { min, max })
}

// @spec FR-002: Complete typed key classification — .specs/features/008-settings-cli/spec.md#fr-002
inventory! {
 "settings_schema_version" => (Integer,false,range(0.0,4294967295.0),&[],true,false,"general",None);
 "bindings" => (Object,false,None,&[],false,false,"shortcuts",None);
 "shortcut_activation" => (String,false,None,&["toggle","push_to_talk","hold_or_toggle"],false,false,"general",Shortcuts);
 "hold_threshold_ms" => (Integer,false,range(100.0,1000.0),&[],false,false,"general",Shortcuts);
 "audio_feedback" => (Boolean,false,None,&[],false,false,"general",AudioOutput);
 "audio_feedback_volume" => (Number,false,range(0.0,1.0),&[],false,false,"general",AudioOutput);
 "sound_theme" => (String,false,None,&["marimba","pop","custom"],false,false,"general",AudioOutput);
 "start_hidden" => (Boolean,false,None,&[],false,false,"general",None);
 "autostart_enabled" => (Boolean,false,None,&[],false,false,"general",Autostart);
 "update_checks_enabled" => (Boolean,false,None,&[],false,false,"general",Updater);
 "show_whats_new_on_update" => (Boolean,false,None,&[],false,false,"general",None);
 "whats_new_last_seen_version" => (String,false,None,&[],true,false,"general",None);
 "selected_model" => (String,false,None,&[],true,false,"general",None);
 "onboarding_completed" => (Boolean,false,None,&[],true,false,"general",None);
 "always_on_microphone" => (Boolean,false,None,&[],false,false,"microphones",AudioInput);
 "selected_microphone" => (String,true,None,&[],true,false,"microphones",None);
 "microphone_priority" => (Array,false,None,&[],false,false,"microphones",AudioInput);
 "selected_channel" => (Integer,true,range(0.0,65535.0),&[],false,false,"microphones",AudioInput);
 "clamshell_microphone" => (String,true,None,&[],false,false,"microphones",AudioInput);
 "selected_output_device" => (String,true,None,&[],false,false,"microphones",AudioOutput);
 "translate_to_english" => (Boolean,false,None,&[],false,false,"general",None);
 "selected_language" => (String,false,None,&[],false,false,"general",None);
 "overlay_position" => (String,false,None,&["top","bottom"],false,false,"general",Overlay);
 "debug_mode" => (Boolean,false,None,&[],false,false,"general",Logging);
 "log_level" => (String,false,None,&["trace","debug","info","warn","error"],false,false,"general",Logging);
 "custom_words" => (Array,false,None,&[],false,false,"vocabulary",None);
 "word_replacements" => (Array,false,None,&[],false,false,"replacements",None);
 "model_unload_timeout" => (String,false,None,&["never","immediately","min2","min5","min10","min15","hour1","sec15"],false,false,"general",Model);
 "word_correction_threshold" => (Number,false,range(0.0,1.0),&[],false,false,"general",None);
 "history_limit" => (Integer,false,range(0.0,1000.0),&[],false,false,"general",History);
 "recording_retention_period" => (String,false,None,&["never","preserve_limit","days3","weeks2","months3"],false,false,"general",History);
 "paste_method" => (String,false,None,crate::smart_paste::setting_policy::PASTE_METHOD_VALUES,false,false,"output",None);
 "clipboard_handling" => (String,false,None,&["dont_modify","copy_to_clipboard"],false,false,"output",None);
 #[cfg(not(feature = "destination-return"))]
 "auto_submit" => (Boolean,false,None,&[],false,false,"output",None);
 // 009 (gate `destination-return` open): Automatic Enter replaces the auto-submit switch, which
 // stays stored unchanged without a descriptor for a closed build.
 #[cfg(feature = "destination-return")]
 "automatic_enter" => (String,false,None,&["never","always","pin_only"],false,false,"output",None);
 "auto_submit_key" => (String,false,None,&["enter","ctrl_enter","cmd_enter"],false,false,"output",None);
 "post_process_enabled" => (Boolean,false,None,&[],false,false,"general",Shortcuts);
 "post_process_provider_id" => (String,false,None,&[],true,false,"general",None);
 "post_process_providers" => (Array,false,None,&[],true,false,"general",None);
 "post_process_api_keys" => (Object,false,None,&[],true,true,"general",None);
 "post_process_models" => (Object,false,None,&[],true,false,"general",None);
 "post_process_prompts" => (Array,false,None,&[],false,false,"prompts",None);
 "post_process_selected_prompt_id" => (String,true,None,&[],false,false,"general",None);
 // 005 FR-007: remembered by the dictation stop; an id must name a library prompt.
 "plain_dictation_prompt_id" => (String,true,None,&[],false,false,"general",None);
 "mute_while_recording" => (Boolean,false,None,&[],false,false,"general",None);
 "append_trailing_space" => (Boolean,false,None,&[],false,false,"output",None);
 // 015: smart capitalization, read at each insertion (no live effect).
 "smart_capitalization" => (Boolean,false,None,&[],false,false,"output",None);
 // 012: favorite destination applications, changed by the badge star and the destination domain;
 // listed before `return_to_dictation_app`, which 010's key follows directly when its gate is open.
 "destination_favorites" => (Array,false,None,&[],false,false,"destination",None);
 "return_to_dictation_app" => (Boolean,false,None,&[],false,false,"output",None);
 // 010 (gate `destination-return` open): read at dictation start, refused off macOS.
 #[cfg(feature = "destination-return")]
 "show_recovery_bubble_without_text_field" => (Boolean,false,None,&[],false,false,"output",None);
 "app_language" => (String,false,None,&[],false,false,"general",Language);
 "theme" => (String,false,None,&["system","light","dark"],false,false,"general",Theme);
 "experimental_enabled" => (Boolean,false,None,&[],false,false,"general",None);
 "lazy_stream_close" => (Boolean,false,None,&[],false,false,"general",None);
 "keyboard_implementation" => (String,false,None,&["tauri","handy_keys"],false,false,"general",Shortcuts);
 "show_tray_icon" => (Boolean,false,None,&[],false,false,"general",Tray);
 "paste_delay_ms" => (Integer,false,range(10.0,5000.0),&[],false,false,"output",None);
 "paste_delay_after_ms" => (Integer,false,range(10.0,5000.0),&[],false,false,"output",None);
 "reliable_paste" => (Boolean,false,None,&[],false,false,"output",None);
 // 007 AC-019 (experimental): read per dictation by smart paste, no live effect.
 "paste_verification" => (Boolean,false,None,&[],false,false,"output",None);
 "smart_paste_migration_version" => (Integer,false,range(0.0,4294967295.0),&[],true,false,"output",None);
 "typing_tool" => (String,false,None,&["auto","wtype","kwtype","dotool","ydotool","xdotool"],false,false,"output",None);
 "external_script_path" => (String,true,None,&[],false,false,"output",None);
 "filler_word_removal_enabled" => (Boolean,false,None,&[],false,false,"general",None);
 "custom_filler_words" => (Array,true,None,&[],false,false,"vocabulary",None);
 "chinese_script" => (String,false,None,&["as_transcribed","simplified","traditional"],false,false,"general",None);
 "transcribe_accelerator" => (String,false,None,&["auto","cpu","gpu"],false,false,"general",Model);
 "ort_accelerator" => (String,false,None,&["auto","cpu","cuda","directml","rocm"],false,false,"general",Model);
 "transcribe_gpu_device" => (String,true,None,&[],false,false,"general",Model);
 "extra_recording_buffer_ms" => (Integer,false,range(0.0,1500.0),&[],false,false,"general",None);
 "vad_enabled" => (Boolean,false,None,&[],false,false,"general",None);
 "vad_backend" => (String,false,None,&["silero","earshot"],false,false,"general",Vad);
 "overlay_style" => (String,false,None,&["none","minimal","live"],false,false,"general",Overlay);
}
