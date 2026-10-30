/**
 * Availability, effective input, refresh fencing and drop position of the microphone priority
 * editor (feature 001). Run with `bun run test:microphone-priority`.
 */
import { describe, expect, test } from "bun:test";
import type { MicrophoneStatus } from "@/bindings";
import {
  EMPTY_STATUS,
  addableNames,
  availabilityOf,
  createReadFence,
  describeEffective,
  enumeratedNames,
  mergeStatusRead,
  reasonMessage,
  secondsSince,
} from "./availability";
import { dropIndexAt } from "./use-pointer-reorder";

const ROOT = "settings.sound.microphonePriority";

const status = (
  overrides: Partial<MicrophoneStatus> = {},
): MicrophoneStatus => ({
  configured_order: ["USB headset", "MacBook"],
  availability: [
    { name: "USB headset", status: "unavailable" },
    { name: "MacBook", status: "available" },
  ],
  availability_observed_at: "2026-10-14T10:00:00Z",
  effective_input: {
    scope: "current",
    name: "MacBook",
    reason: "fallback",
    capture_generation: 3,
  },
  warning: null,
  ...overrides,
});

// @spec AC-002: Availability statuses, unknown on refresh errors — .specs/features/001-microphone-priority/spec.md#ac-002
describe("availability", () => {
  const view = mergeStatusRead(EMPTY_STATUS, status());

  test("each saved choice keeps its observed status", () => {
    expect(availabilityOf("USB headset", view, null)).toBe("unavailable");
    expect(availabilityOf("MacBook", view, null)).toBe("available");
  });

  test("a name only in the draft is classified from the enumeration", () => {
    const names = ["MacBook", "Dock", "Dock"];
    expect(availabilityOf("Studio", view, names)).toBe("unavailable");
    expect(availabilityOf("Dock", view, names)).toBe("ambiguous");
    expect(availabilityOf("Studio", view, null)).toBe("unknown");
  });

  test("a failed read shows unknown and keeps the last effective input", () => {
    const failed = mergeStatusRead(view, null);
    expect(failed.failed).toBe(true);
    expect(failed.status?.effective_input.name).toBe("MacBook");
    expect(availabilityOf("MacBook", failed, ["MacBook"])).toBe("unknown");
  });

  test("an enumeration failure warning marks the read failed", () => {
    const read = status({ warning: "enumeration_failed" });
    expect(mergeStatusRead(EMPTY_STATUS, read).failed).toBe(true);
  });

  test("the synthetic system default entry is not offered for Add", () => {
    const names = addableNames([
      { index: "default", name: "Default", is_default: true },
      { index: "0", name: "MacBook", is_default: false },
      { index: "1", name: "default", is_default: false },
    ]);
    expect(JSON.stringify(names)).toBe(JSON.stringify(["MacBook", "default"]));
  });

  test("a failed device listing leaves draft-only names unknown, not unavailable", () => {
    // After a failed listing the store keeps only the synthetic default entry.
    const fallback = [{ index: "default", name: "Default", is_default: true }];
    const enumerated = enumeratedNames(fallback, true);
    expect(enumerated).toBe(null);
    expect(availabilityOf("Studio", view, enumerated)).toBe("unknown");
    expect(availabilityOf("USB headset", view, enumerated)).toBe("unavailable");
    const listed = enumeratedNames(fallback, false);
    expect(JSON.stringify(listed)).toBe("[]");
    expect(availabilityOf("Studio", view, listed)).toBe("unavailable");
    expect(enumeratedNames([], false)).toBe(null);
  });
});

/** Status view of one successful read. */
const read = (overrides: Partial<MicrophoneStatus>) =>
  mergeStatusRead(EMPTY_STATUS, status(overrides));

// @spec AC-002: Effective input shown apart from the saved order — .specs/features/001-microphone-priority/spec.md#ac-002
describe("effective input", () => {
  test("a fallback names the unavailable first choice", () => {
    const view = describeEffective(mergeStatusRead(EMPTY_STATUS, status()));
    expect(view.name).toBe("MacBook");
    expect(view.current).toBe(true);
    expect(view.reason?.key).toBe(`${ROOT}.reason.fallbackUnavailable`);
    expect(view.reason?.params?.name).toBe("USB headset");
  });

  test("a fallback without an observed cause stays generic", () => {
    const reason = reasonMessage("fallback", "MacBook", [
      { name: "USB headset", status: "available" },
      { name: "MacBook", status: "available" },
    ]);
    expect(reason.key).toBe(`${ROOT}.reason.fallback`);
  });

  test("the last recording is labelled as such", () => {
    const view = describeEffective(
      read({
        effective_input: {
          scope: "last_recording",
          name: "USB headset",
          reason: "first_choice",
          capture_generation: 2,
        },
      }),
    );
    expect(view.current).toBe(false);
    expect(view.reason?.key).toBe(`${ROOT}.reason.first_choice`);
  });

  test("not used yet after a launch, loading or unknown before a read", () => {
    const notUsed = describeEffective(
      read({
        effective_input: {
          scope: "not_used_yet",
          name: null,
          reason: null,
          capture_generation: null,
        },
      }),
    );
    expect(notUsed.label?.key).toBe(`${ROOT}.inUse.notUsedYet`);
    expect(describeEffective(EMPTY_STATUS).label?.key).toBe("common.loading");
    const failed = { status: null, failed: true };
    expect(describeEffective(failed).label?.key).toBe(`${ROOT}.status.unknown`);
  });

  test("an unnamed system default is described", () => {
    const view = describeEffective(
      read({
        effective_input: {
          scope: "current",
          name: null,
          reason: "system_default",
          capture_generation: 1,
        },
      }),
    );
    expect(view.label?.key).toBe(`${ROOT}.inUse.systemDefault`);
  });
});

// @spec AC-002: Refresh every 5 s and on Retry, late replies dropped — .specs/features/001-microphone-priority/spec.md#ac-002
describe("refresh", () => {
  test("only the newest read may update the view", () => {
    const fence = createReadFence();
    const first = fence.begin();
    const second = fence.begin();
    expect(fence.isCurrent(first)).toBe(false);
    expect(fence.isCurrent(second)).toBe(true);
    fence.invalidate();
    expect(fence.isCurrent(second)).toBe(false);
  });

  test("checked age in whole seconds, never negative", () => {
    const at = "2026-10-14T10:00:00Z";
    expect(secondsSince(at, Date.parse(at) + 3400)).toBe(3);
    expect(secondsSince(at, Date.parse(at) - 500)).toBe(0);
    expect(secondsSince("not a time", 0)).toBe(null);
  });
});

// @spec FR-001: Pointer reorder drop position — .specs/features/001-microphone-priority/spec.md#fr-001
describe("drop position", () => {
  const rows = [
    { top: 0, bottom: 40 },
    { top: 40, bottom: 80 },
    { top: 80, bottom: 120 },
  ];

  test("the row under the pointer, clamped to the list", () => {
    expect(dropIndexAt(rows, 90)).toBe(2);
    expect(dropIndexAt(rows, 40)).toBe(1);
    expect(dropIndexAt(rows, -10)).toBe(0);
    expect(dropIndexAt(rows, 500)).toBe(2);
    expect(dropIndexAt([], 10)).toBe(-1);
  });
});
