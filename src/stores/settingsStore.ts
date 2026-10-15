import { createElement } from "react";
import { create } from "zustand";
import { subscribeWithSelector } from "zustand/middleware";
import { listen } from "@tauri-apps/api/event";
import type {
  AppSettings as Settings,
  AudioDevice,
  ChineseScript,
  ModelUnloadTimeout,
  TranscribeAcceleratorSetting,
  OrtAcceleratorSetting,
  RefusedShortcut,
  ShortcutActivation,
  ShortcutEdit,
  ShortcutEditError,
  VadBackend,
} from "@/bindings";
import { commands } from "@/bindings";
import { toast } from "sonner";
import { keepStoredRefusals } from "@/components/settings/shortcut-list-model";
import i18n from "@/i18n";
import {
  failedCommandRecovery,
  isCurrentRevision,
  SETTINGS_ERROR_KEYS,
  SettingsCommandFailure,
  settingsErrorDisplay,
  settingsErrorKey,
  throwIfSettingsError,
} from "./settings-errors";

interface SettingsStore {
  settings: Settings | null;
  // Revision of `settings`; refresh responses older than it are discarded.
  revision: number | null;
  defaultSettings: Settings | null;
  isLoading: boolean;
  isUpdating: Record<string, boolean>;
  audioDevices: AudioDevice[];
  // The last microphone listing failed: `audioDevices` then holds only the synthetic default,
  // which is not evidence that a device is absent (feature 001, FR-002).
  audioDevicesFailed: boolean;
  outputDevices: AudioDevice[];
  customSounds: { start: boolean; stop: boolean };
  postProcessModelOptions: Record<string, string[]>;
  // null until loadUpdateChecksLocked() resolves
  updateChecksLocked: boolean | null;
  // Shortcuts the system refused when post-processing was last enabled;
  // their rows carry a warning until they are changed.
  refusedShortcuts: RefusedShortcut[];
  // Another writer (CLI, other window) committed a newer revision than the one a write expected;
  // the settings window shows "Settings changed elsewhere" until Reload.
  revisionConflict: boolean;
  // The last Reload of that alert could not read the settings: the alert stays, with an error.
  conflictReloadFailed: boolean;

  // Actions
  reloadAfterConflict: () => Promise<void>;
  initialize: () => Promise<void>;
  loadDefaultSettings: () => Promise<void>;
  loadUpdateChecksLocked: () => Promise<void>;
  updateSetting: <K extends keyof Settings>(
    key: K,
    value: Settings[K],
  ) => Promise<boolean>;
  resetSetting: (key: keyof Settings) => Promise<void>;
  refreshSettings: () => Promise<void>;
  refreshAudioDevices: () => Promise<void>;
  refreshOutputDevices: () => Promise<void>;
  editBindingShortcuts: (
    id: string,
    edit: ShortcutEdit,
  ) => Promise<ShortcutEditError | null>;
  setPostProcessEnabled: (enabled: boolean) => Promise<RefusedShortcut[]>;
  clearRefusedShortcuts: () => void;
  getSetting: <K extends keyof Settings>(key: K) => Settings[K] | undefined;
  isUpdatingKey: (key: string) => boolean;
  playTestSound: (soundType: "start" | "stop") => Promise<void>;
  checkCustomSounds: () => Promise<void>;
  setPostProcessProvider: (providerId: string) => Promise<void>;
  updatePostProcessSetting: (
    settingType: "base_url" | "api_key" | "model",
    providerId: string,
    value: string,
  ) => Promise<void>;
  updatePostProcessBaseUrl: (
    providerId: string,
    baseUrl: string,
  ) => Promise<void>;
  updatePostProcessApiKey: (
    providerId: string,
    apiKey: string,
  ) => Promise<void>;
  updatePostProcessModel: (providerId: string, model: string) => Promise<void>;
  fetchPostProcessModels: (providerId: string) => Promise<string[]>;
  setPostProcessModelOptions: (providerId: string, models: string[]) => void;

  // Internal state setters
  setSettings: (settings: Settings | null) => void;
  setDefaultSettings: (defaultSettings: Settings | null) => void;
  setLoading: (loading: boolean) => void;
  setUpdating: (key: string, updating: boolean) => void;
  setAudioDevices: (devices: AudioDevice[]) => void;
  setOutputDevices: (devices: AudioDevice[]) => void;
  setCustomSounds: (sounds: { start: boolean; stop: boolean }) => void;
}

/**
 * Show a refused settings command explicitly (translated from its typed code/reason).
 * @param error error thrown by a settings action
 */
const SAVE_FAILED_TEST_ID = "settings-write-errors-save-failed";

const notifySettingsError = (error: unknown): void => {
  const detail = error instanceof SettingsCommandFailure ? error.detail : null;
  const key = detail ? settingsErrorKey(detail) : SETTINGS_ERROR_KEYS.generic;
  switch (settingsErrorDisplay(detail)) {
    case "conflict-alert":
      useSettingsStore.setState({
        revisionConflict: true,
        conflictReloadFailed: false,
      });
      return;
    case "save-failed-toast":
      // Test ids of the settings-write-errors contract on the notice and its text.
      toast.error(
        createElement(
          "span",
          { "data-testid": `${SAVE_FAILED_TEST_ID}-title` },
          i18n.t(key),
        ),
        { testId: SAVE_FAILED_TEST_ID },
      );
      return;
    default:
      toast.error(i18n.t(key));
  }
};

// Shown once per session: every later change is refused with its own message anyway.
let readOnlyWarned = false;

// Note: Default settings are now fetched from Rust via commands.getDefaultSettings()
// This ensures platform-specific defaults (like overlay_position, shortcuts, paste_method) work correctly

const DEFAULT_AUDIO_DEVICE: AudioDevice = {
  index: "default",
  name: "Default",
  is_default: true,
};

// The "Default" entry of the device selectors is stored as null (system default); every other
// name, "default" included (a real ALSA device on Linux), is stored as a literal device name.
const toStoredDevice = (value: unknown): string | null =>
  value === null || value === undefined || value === DEFAULT_AUDIO_DEVICE.name
    ? null
    : (value as string);

const settingUpdaters: {
  [K in keyof Settings]?: (value: Settings[K]) => Promise<unknown>;
} = {
  always_on_microphone: (value) =>
    commands.updateMicrophoneMode(value as boolean),
  audio_feedback: (value) =>
    commands.changeAudioFeedbackSetting(value as boolean),
  audio_feedback_volume: (value) =>
    commands.changeAudioFeedbackVolumeSetting(value as number),
  sound_theme: (value) => commands.changeSoundThemeSetting(value as string),
  start_hidden: (value) => commands.changeStartHiddenSetting(value as boolean),
  autostart_enabled: (value) =>
    commands.changeAutostartSetting(value as boolean),
  update_checks_enabled: (value) =>
    commands.changeUpdateChecksSetting(value as boolean),
  show_whats_new_on_update: (value) =>
    commands.changeShowWhatsNewOnUpdateSetting(value as boolean),
  whats_new_last_seen_version: (value) =>
    commands.changeWhatsNewLastSeenVersionSetting(value as string),
  shortcut_activation: (value) =>
    commands.changeShortcutActivationSetting(value as ShortcutActivation),
  hold_threshold_ms: (value) =>
    commands.changeHoldThresholdMsSetting(value as number),
  // Feature 001: `selected_microphone` is read-only; the priority editor saves the complete
  // list through `set_microphone_priority` (microphone-priority/use-priority-editor.ts).
  selected_channel: (value) =>
    commands.setSelectedChannel((value as number | null | undefined) ?? null),
  clamshell_microphone: (value) =>
    commands.setClamshellMicrophone(toStoredDevice(value)),
  selected_output_device: (value) =>
    commands.setSelectedOutputDevice(toStoredDevice(value)),
  recording_retention_period: (value) =>
    commands.updateRecordingRetentionPeriod(value as string),
  translate_to_english: (value) =>
    commands.changeTranslateToEnglishSetting(value as boolean),
  selected_language: (value) =>
    commands.changeSelectedLanguageSetting(value as string),
  overlay_position: (value) =>
    commands.changeOverlayPositionSetting(value as string),
  debug_mode: (value) => commands.changeDebugModeSetting(value as boolean),
  custom_words: (value) => commands.updateCustomWords(value as string[], null),
  word_correction_threshold: (value) =>
    commands.changeWordCorrectionThresholdSetting(value as number),
  paste_delay_ms: (value) =>
    commands.changePasteDelayMsSetting(value as number),
  paste_delay_after_ms: (value) =>
    commands.changePasteDelayAfterMsSetting(value as number),
  reliable_paste: (value) =>
    commands.changeReliablePasteSetting(value as boolean),
  paste_method: (value) => commands.changePasteMethodSetting(value as string),
  typing_tool: (value) => commands.changeTypingToolSetting(value as string),
  external_script_path: (value) =>
    commands.changeExternalScriptPathSetting(value as string | null),
  clipboard_handling: (value) =>
    commands.changeClipboardHandlingSetting(value as string),
  auto_submit: (value) => commands.changeAutoSubmitSetting(value as boolean),
  auto_submit_key: (value) =>
    commands.changeAutoSubmitKeySetting(value as string),
  // 009 (gate `destination-return` open): refused `not_found` by a closed build, which never
  // shows this control.
  automatic_enter: (value) =>
    commands.changeAutomaticEnterSetting(value as string),
  history_limit: (value) => commands.updateHistoryLimit(value as number),
  post_process_enabled: (value) => togglePostProcess(value as boolean),
  post_process_selected_prompt_id: (value) =>
    commands.setPostProcessSelectedPrompt(value as string),
  mute_while_recording: (value) =>
    commands.changeMuteWhileRecordingSetting(value as boolean),
  append_trailing_space: (value) =>
    commands.changeAppendTrailingSpaceSetting(value as boolean),
  // 006: refused with the typed reason `unsupported_platform` on a build without the destination
  // capability; the refusal is shown and the optimistic value rolled back like any other.
  return_to_dictation_app: (value) =>
    commands.changeReturnToDictationAppSetting(value as boolean),
  // 010 (gate `destination-return` open): refused `unsupported_platform` off macOS and
  // `not_found` by a closed build, which never shows this control.
  show_recovery_bubble_without_text_field: (value) =>
    commands.changeShowRecoveryBubbleWithoutTextFieldSetting(value as boolean),
  log_level: (value) => commands.setLogLevel(value as any),
  app_language: (value) => commands.changeAppLanguageSetting(value as string),
  theme: (value) => commands.changeThemeSetting(value as string),
  experimental_enabled: (value) =>
    commands.changeExperimentalEnabledSetting(value as boolean),
  lazy_stream_close: (value) =>
    commands.changeLazyStreamCloseSetting(value as boolean),
  overlay_style: (value) => commands.changeOverlayStyleSetting(value as string),
  vad_enabled: (value) => commands.changeVadEnabledSetting(value as boolean),
  // Rejected switches (e.g. mid-recording) roll the dropdown back in updateSetting.
  vad_backend: (value) => commands.changeVadBackendSetting(value as VadBackend),
  filler_word_removal_enabled: (value) =>
    commands.changeFillerWordRemovalEnabledSetting(value as boolean),
  chinese_script: (value) =>
    commands.changeChineseScriptSetting(value as ChineseScript),
  show_tray_icon: (value) =>
    commands.changeShowTrayIconSetting(value as boolean),
  transcribe_accelerator: (value) =>
    commands.changeTranscribeAcceleratorSetting(
      value as TranscribeAcceleratorSetting,
    ),
  ort_accelerator: (value) =>
    commands.changeOrtAcceleratorSetting(value as OrtAcceleratorSetting),
  transcribe_gpu_device: (value) =>
    commands.changeTranscribeGpuDevice(value as string | null),
  extra_recording_buffer_ms: (value) =>
    commands.changeExtraRecordingBufferSetting(value as number),
  model_unload_timeout: (value) =>
    commands.setModelUnloadTimeout(value as ModelUnloadTimeout),
};

/**
 * Reads the stored settings into the store; an older answer never rolls the window back.
 * @param get Store state reader.
 * @param set Store state writer.
 * @returns True once the window shows the stored settings (this view, or a newer one already
 * applied); false when they could not be read (logged).
 */
const loadSettingsView = async (
  get: () => SettingsStore,
  set: (partial: Partial<SettingsStore>) => void,
): Promise<boolean> => {
  try {
    const view = await commands.getSettingsView();
    const { revision, settings } = view;
    if (view.read_only && !readOnlyWarned) {
      readOnlyWarned = true;
      toast.warning(i18n.t(SETTINGS_ERROR_KEYS.readOnly));
    }
    // An older refresh answering after a newer one must not roll the UI back.
    if (!isCurrentRevision(revision, get().revision)) return true;
    set({ settings: normalizedSettings(settings), revision, isLoading: false });
    return true;
  } catch (error) {
    console.error("Failed to load settings:", error);
    set({ isLoading: false });
    return false;
  }
};

/** Stored settings with the defaults the selectors expect. */
const normalizedSettings = (settings: Settings): Settings => ({
  ...settings,
  always_on_microphone: settings.always_on_microphone ?? false,
  // The channel selector reads the first priority choice (feature 001); an empty
  // list is the system default.
  selected_microphone: settings.microphone_priority?.[0] ?? "Default",
  clamshell_microphone: settings.clamshell_microphone ?? "Default",
  selected_output_device: settings.selected_output_device ?? "Default",
});

export const useSettingsStore = create<SettingsStore>()(
  subscribeWithSelector((set, get) => ({
    settings: null,
    revision: null,
    defaultSettings: null,
    isLoading: true,
    isUpdating: {},
    audioDevices: [],
    audioDevicesFailed: false,
    outputDevices: [],
    customSounds: { start: false, stop: false },
    postProcessModelOptions: {},
    updateChecksLocked: null,
    refusedShortcuts: [],
    revisionConflict: false,
    conflictReloadFailed: false,

    // Internal setters
    setSettings: (settings) => set({ settings }),
    setDefaultSettings: (defaultSettings) => set({ defaultSettings }),
    setLoading: (isLoading) => set({ isLoading }),
    setUpdating: (key, updating) =>
      set((state) => ({
        isUpdating: { ...state.isUpdating, [key]: updating },
      })),
    setAudioDevices: (audioDevices) => set({ audioDevices }),
    setOutputDevices: (outputDevices) => set({ outputDevices }),
    setCustomSounds: (customSounds) => set({ customSounds }),

    // Getters
    getSetting: (key) => get().settings?.[key],
    isUpdatingKey: (key) => get().isUpdating[key] || false,

    // Load settings from store
    refreshSettings: async () => {
      await loadSettingsView(get, set);
    },

    // Load audio devices
    refreshAudioDevices: async () => {
      try {
        const result = await commands.getAvailableMicrophones();
        if (result.status === "ok") {
          const devicesWithDefault = [
            DEFAULT_AUDIO_DEVICE,
            ...result.data.filter((d) => d.name !== DEFAULT_AUDIO_DEVICE.name),
          ];
          set({ audioDevices: devicesWithDefault, audioDevicesFailed: false });
        } else {
          set({
            audioDevices: [DEFAULT_AUDIO_DEVICE],
            audioDevicesFailed: true,
          });
        }
      } catch (error) {
        console.error("Failed to load audio devices:", error);
        set({ audioDevices: [DEFAULT_AUDIO_DEVICE], audioDevicesFailed: true });
      }
    },

    // Load output devices
    refreshOutputDevices: async () => {
      try {
        const result = await commands.getAvailableOutputDevices();
        if (result.status === "ok") {
          const devicesWithDefault = [
            DEFAULT_AUDIO_DEVICE,
            ...result.data.filter((d) => d.name !== DEFAULT_AUDIO_DEVICE.name),
          ];
          set({ outputDevices: devicesWithDefault });
        } else {
          set({ outputDevices: [DEFAULT_AUDIO_DEVICE] });
        }
      } catch (error) {
        console.error("Failed to load output devices:", error);
        set({ outputDevices: [DEFAULT_AUDIO_DEVICE] });
      }
    },

    // Play a test sound
    playTestSound: async (soundType: "start" | "stop") => {
      try {
        await commands.playTestSound(soundType);
      } catch (error) {
        console.error(`Failed to play test sound (${soundType}):`, error);
      }
    },

    checkCustomSounds: async () => {
      try {
        const sounds = await commands.checkCustomSounds();
        get().setCustomSounds(sounds);
      } catch (error) {
        console.error("Failed to check custom sounds:", error);
      }
    },

    // Update a specific setting
    updateSetting: async <K extends keyof Settings>(
      key: K,
      value: Settings[K],
    ) => {
      const { settings, setUpdating, revision: revisionAtStart } = get();
      const updateKey = String(key);
      const originalValue = settings?.[key];

      setUpdating(updateKey, true);

      try {
        set((state) => ({
          settings: state.settings ? { ...state.settings, [key]: value } : null,
        }));

        const updater = settingUpdaters[key];
        if (updater) {
          throwIfSettingsError(await updater(value));
        } else if (key !== "bindings" && key !== "selected_model") {
          console.warn(`No handler for setting: ${String(key)}`);
        }
        return true;
      } catch (error) {
        // A failure is always shown. A refusal is rolled back; a committed or possibly committed
        // value, or one superseded by a newer revision, is reread instead (the store decides).
        console.error(`Failed to update setting ${String(key)}:`, error);
        notifySettingsError(error);
        if (
          failedCommandRecovery(error, revisionAtStart, get().revision) ===
          "refresh"
        ) {
          await get().refreshSettings();
        } else {
          set((state) => ({
            settings: state.settings
              ? { ...state.settings, [key]: originalValue }
              : null,
          }));
        }
        return false;
      } finally {
        setUpdating(updateKey, false);
      }
    },

    // Reset a setting to its default value
    resetSetting: async (key) => {
      const { defaultSettings } = get();
      if (defaultSettings) {
        const defaultValue = defaultSettings[key];
        if (defaultValue !== undefined) {
          await get().updateSetting(key, defaultValue as any);
        }
      }
    },

    // Edit the shortcut list of one action; the backend validates, registers
    // and saves. Returns the typed refusal, if any (a refusal of the settings
    // service included), for the caller to show.
    editBindingShortcuts: async (id, edit) => {
      const updateKey = `binding_${id}`;
      get().setUpdating(updateKey, true);
      try {
        const response = await commands.editBindingShortcuts(id, edit);
        const saved = response.binding;
        if (saved) {
          set((state) => {
            const bindings = { ...state.settings?.bindings, [id]: saved };
            return {
              settings: state.settings ? { ...state.settings, bindings } : null,
              // A refused shortcut that was changed or removed is no
              // longer flagged.
              refusedShortcuts: keepStoredRefusals(
                state.refusedShortcuts,
                bindings,
              ),
            };
          });
        }
        return response.error;
      } finally {
        get().setUpdating(updateKey, false);
      }
    },

    // A keyboard implementation switch registers every shortcut again, so
    // the refusals of the last activation no longer describe the backend.
    clearRefusedShortcuts: () => set({ refusedShortcuts: [] }),

    // Reload after "Settings changed elsewhere": the store is authoritative, so the newer
    // revision replaces whatever this window held; the alert closes once it is shown. A failed
    // read keeps the alert open with an error instead of claiming the window is current.
    reloadAfterConflict: async () => {
      const isShown = await loadSettingsView(get, set);
      set(
        isShown
          ? { revisionConflict: false, conflictReloadFailed: false }
          : { conflictReloadFailed: true },
      );
    },

    // Enable or disable post-processing; the backend always saves the
    // choice. Returns the shortcuts the system refused, for the caller to
    // report. If the command itself fails, the previous value is shown
    // again and the error is thrown.
    setPostProcessEnabled: async (enabled) => {
      const updateKey = "post_process_enabled";
      const previous = get().settings?.post_process_enabled ?? false;
      const show = (value: boolean) =>
        set((state) => ({
          settings: state.settings
            ? { ...state.settings, post_process_enabled: value }
            : null,
        }));
      get().setUpdating(updateKey, true);
      show(enabled);
      try {
        return await togglePostProcess(enabled);
      } catch (error) {
        show(previous);
        throw error;
      } finally {
        get().setUpdating(updateKey, false);
      }
    },

    setPostProcessProvider: async (providerId) => {
      const {
        settings,
        setUpdating,
        refreshSettings,
        setPostProcessModelOptions,
      } = get();
      const updateKey = "post_process_provider_id";
      const previousId = settings?.post_process_provider_id ?? null;
      const revisionAtStart = get().revision;

      setUpdating(updateKey, true);

      if (settings) {
        set((state) => ({
          settings: state.settings
            ? { ...state.settings, post_process_provider_id: providerId }
            : null,
        }));
      }

      // Clear cached model options for the new provider so the dropdown
      // doesn't show stale models from a previous fetch or base_url.
      setPostProcessModelOptions(providerId, []);

      try {
        throwIfSettingsError(await commands.setPostProcessProvider(providerId));
        await refreshSettings();
      } catch (error) {
        console.error("Failed to set post-process provider:", error);
        notifySettingsError(error);
        if (
          failedCommandRecovery(error, revisionAtStart, get().revision) ===
          "refresh"
        ) {
          await refreshSettings();
        } else if (previousId !== null) {
          set((state) => ({
            settings: state.settings
              ? { ...state.settings, post_process_provider_id: previousId }
              : null,
          }));
        }
      } finally {
        setUpdating(updateKey, false);
      }
    },

    // Generic updater for post-processing provider settings
    updatePostProcessSetting: async (
      settingType: "base_url" | "api_key" | "model",
      providerId: string,
      value: string,
    ) => {
      const { setUpdating, refreshSettings } = get();
      const updateKey = `post_process_${settingType}:${providerId}`;

      setUpdating(updateKey, true);

      try {
        if (settingType === "base_url") {
          throwIfSettingsError(
            await commands.changePostProcessBaseUrlSetting(providerId, value),
          );
        } else if (settingType === "api_key") {
          throwIfSettingsError(
            await commands.changePostProcessApiKeySetting(providerId, value),
          );
        } else if (settingType === "model") {
          throwIfSettingsError(
            await commands.changePostProcessModelSetting(providerId, value),
          );
        }
        await refreshSettings();
      } catch (error) {
        console.error(
          `Failed to update post-process ${settingType.replace("_", " ")}:`,
          error,
        );
        notifySettingsError(error);
      } finally {
        setUpdating(updateKey, false);
      }
    },

    updatePostProcessBaseUrl: async (providerId, baseUrl) => {
      const { setUpdating, refreshSettings } = get();
      const updateKey = `post_process_base_url:${providerId}`;

      setUpdating(updateKey, true);

      try {
        // Persist the new base URL first.
        const urlResult = await commands.changePostProcessBaseUrlSetting(
          providerId,
          baseUrl,
        );
        if (urlResult.status === "error") {
          console.error("Failed to persist base URL:", urlResult.error);
          notifySettingsError(new SettingsCommandFailure(urlResult.error));
          return;
        }

        // Reset the stored model since the previous value is almost certainly
        // invalid for the new endpoint (e.g. switching Custom from Groq to
        // Cerebras). Only proceed if the reset succeeds.
        const modelResult = await commands.changePostProcessModelSetting(
          providerId,
          "",
        );
        if (modelResult.status === "error") {
          console.error("Failed to reset model setting:", modelResult.error);
          notifySettingsError(new SettingsCommandFailure(modelResult.error));
          return;
        }

        // Clear cached model options only after both backend writes succeed.
        set((state) => ({
          postProcessModelOptions: {
            ...state.postProcessModelOptions,
            [providerId]: [],
          },
        }));

        // Single refresh after both backend writes.
        await refreshSettings();
      } catch (error) {
        console.error("Failed to update post-process base URL:", error);
      } finally {
        setUpdating(updateKey, false);
      }
    },

    updatePostProcessApiKey: async (providerId, apiKey) => {
      // Clear cached models when API key changes - user should click refresh after
      set((state) => ({
        postProcessModelOptions: {
          ...state.postProcessModelOptions,
          [providerId]: [],
        },
      }));
      return get().updatePostProcessSetting("api_key", providerId, apiKey);
    },

    updatePostProcessModel: async (providerId, model) => {
      return get().updatePostProcessSetting("model", providerId, model);
    },

    fetchPostProcessModels: async (providerId) => {
      const updateKey = `post_process_models_fetch:${providerId}`;
      const { setUpdating, setPostProcessModelOptions } = get();

      setUpdating(updateKey, true);

      try {
        // Call Tauri backend command instead of fetch
        const result = await commands.fetchPostProcessModels(providerId);
        if (result.status === "ok") {
          setPostProcessModelOptions(providerId, result.data);
          return result.data;
        } else {
          console.error("Failed to fetch models:", result.error);
          return [];
        }
      } catch (error) {
        console.error("Failed to fetch models:", error);
        // Don't cache empty array on error - let user retry
        return [];
      } finally {
        setUpdating(updateKey, false);
      }
    },

    setPostProcessModelOptions: (providerId, models) =>
      set((state) => ({
        postProcessModelOptions: {
          ...state.postProcessModelOptions,
          [providerId]: models,
        },
      })),

    // Load default settings from Rust
    loadDefaultSettings: async () => {
      try {
        set({ defaultSettings: await commands.getDefaultSettings() });
      } catch (error) {
        console.error("Failed to load default settings:", error);
      }
    },

    // Check whether update checks are locked by system configuration
    // (e.g. HANDY_DISABLE_UPDATER, set by the Nix package)
    loadUpdateChecksLocked: async () => {
      try {
        const locked = await commands.isUpdateChecksLocked();
        set({ updateChecksLocked: locked });
      } catch (error) {
        console.error("Failed to check update checks lock state:", error);
        // Fail open: an unknown lock state means "not locked", otherwise the
        // update checker waits for it forever and checks never start.
        set({ updateChecksLocked: false });
      }
    },

    // Initialize everything
    initialize: async () => {
      const {
        refreshSettings,
        checkCustomSounds,
        loadDefaultSettings,
        loadUpdateChecksLocked,
      } = get();

      // Note: Audio devices are NOT refreshed here. The frontend (App.tsx)
      // is responsible for calling refreshAudioDevices/refreshOutputDevices
      // after onboarding completes. This avoids triggering permission dialogs
      // on macOS before the user is ready.
      await Promise.all([
        loadDefaultSettings(),
        refreshSettings(),
        checkCustomSounds(),
        loadUpdateChecksLocked(),
      ]);

      // Re-fetch settings when the backend changes them (e.g. language
      // reset during model switch). The backend is the source of truth.
      listen("model-state-changed", () => {
        get().refreshSettings();
      });
      // Every committed write (UI, internal repair, later the CLI) emits its revision and the
      // changed key names; values are refetched, never carried by the event.
      listen<{ revision: number; keys: string[] }>(
        "settings-changed",
        (event) => {
          get().refreshSettings();
          if (event.payload.keys.includes("microphone_priority")) {
            get().refreshAudioDevices();
          }
        },
      );
    },
  })),
);

// @spec AC-002: Post-processing activation behaves as before — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-002
/**
 * Toggles post-processing in the backend, which always saves the choice, as
 * before multiple shortcuts. Records the shortcuts the system refused so
 * their rows are flagged; disabling clears them.
 * @param enabled New value of `post_process_enabled`.
 * @returns The refused shortcuts, empty when disabling.
 */
async function togglePostProcess(enabled: boolean): Promise<RefusedShortcut[]> {
  const result = await commands.changePostProcessEnabledSetting(enabled);
  throwIfSettingsError(result);
  if (result.status === "error") return [];
  const refused = enabled ? result.data : [];
  useSettingsStore.setState({ refusedShortcuts: refused });
  return refused;
}
