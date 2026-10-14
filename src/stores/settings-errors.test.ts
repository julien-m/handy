import assert from "node:assert/strict";
import type { SettingsCommandError } from "@/bindings";
import {
  failedCommandRecovery,
  isCurrentRevision,
  isSettingsCommandError,
  SETTINGS_ERROR_KEYS,
  SettingsCommandFailure,
  settingsErrorKey,
  throwIfSettingsError,
} from "./settings-errors";

const refusal = (
  overrides: Partial<SettingsCommandError>,
): SettingsCommandError => ({
  code: "validation_failed",
  message: "Setting value failed validation.",
  field: null,
  reason: null,
  outcome: null,
  current_revision: null,
  ...overrides,
});

assert.equal(
  settingsErrorKey(
    refusal({ field: "transcribe_accelerator", reason: "gpu_device_required" }),
  ),
  SETTINGS_ERROR_KEYS.gpuDeviceRequired,
);
assert.equal(
  settingsErrorKey(
    refusal({ field: "paste_delay_ms", reason: "out_of_range" }),
  ),
  SETTINGS_ERROR_KEYS.outOfRange,
);
assert.equal(
  settingsErrorKey(
    refusal({ field: "paste_method", reason: "canonical_value" }),
  ),
  SETTINGS_ERROR_KEYS.validationFailed,
);
assert.equal(
  settingsErrorKey(
    refusal({
      code: "internal_error",
      outcome: "committed",
      current_revision: 7,
    }),
  ),
  SETTINGS_ERROR_KEYS.committedNotApplied,
);
assert.equal(
  settingsErrorKey(
    refusal({ field: "paste_method", reason: "unsupported_platform" }),
  ),
  SETTINGS_ERROR_KEYS.unsupportedPlatform,
);
assert.equal(
  settingsErrorKey(refusal({ code: "unavailable", field: "vad_backend" })),
  SETTINGS_ERROR_KEYS.unavailable,
);
assert.equal(
  settingsErrorKey(refusal({ code: "persistence_failed" })),
  SETTINGS_ERROR_KEYS.persistenceFailed,
);
assert.equal(
  settingsErrorKey(refusal({ code: "revision_conflict", current_revision: 3 })),
  SETTINGS_ERROR_KEYS.revisionConflict,
);
assert.equal(
  settingsErrorKey(refusal({ code: "not_found" })),
  SETTINGS_ERROR_KEYS.generic,
);

assert.ok(isSettingsCommandError(refusal({})));
assert.ok(!isSettingsCommandError("Setting value failed validation."));
assert.ok(!isSettingsCommandError(null));

assert.doesNotThrow(() => throwIfSettingsError({ status: "ok", data: null }));
assert.doesNotThrow(() => throwIfSettingsError(undefined));
assert.throws(
  () => throwIfSettingsError({ status: "error", error: refusal({}) }),
  (thrown: unknown) =>
    thrown instanceof SettingsCommandFailure &&
    thrown.detail.code === "validation_failed",
);
assert.throws(
  () => throwIfSettingsError({ status: "error", error: "legacy string" }),
  /legacy string/,
);

assert.ok(isCurrentRevision(5, null));
assert.ok(isCurrentRevision(6, 5));
assert.ok(isCurrentRevision(5, 5));
assert.ok(!isCurrentRevision(4, 5));

// Recovery of the optimistic value after a failed command: the store is authoritative.
const committed = new SettingsCommandFailure(
  refusal({
    code: "internal_error",
    outcome: "committed",
    current_revision: 5,
  }),
);
const unknown = new SettingsCommandFailure(
  refusal({ code: "unavailable", outcome: "unknown" }),
);
const refused = new SettingsCommandFailure(refusal({}));
assert.equal(failedCommandRecovery(committed, 4, 4), "refresh");
assert.equal(failedCommandRecovery(unknown, 4, 4), "refresh");
assert.equal(failedCommandRecovery(refused, 4, 4), "rollback");
assert.equal(failedCommandRecovery(refused, 4, 6), "refresh");
assert.equal(failedCommandRecovery(new Error("ipc"), null, null), "rollback");

console.log("settings-errors: all assertions passed");
