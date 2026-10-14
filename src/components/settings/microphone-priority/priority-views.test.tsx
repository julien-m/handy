/**
 * Markup of the microphone priority editor views (feature 001): the Penflow test IDs of each
 * mockup state, the accessible names and states of the controls, and the visual states of the
 * behavioral contract. Run with `bun run test:microphone-priority`.
 */
import { describe, expect, mock, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import type { MicrophoneStatus } from "@/bindings";

// Keys are rendered with their values so the markup shows which message and name are used.
mock.module("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, string>) =>
      params ? `${key}${JSON.stringify(params)}` : key,
  }),
}));
const { PriorityList } = await import("./priority-list");
const { AddControl, AddErrorText, CheckLine } = await import(
  "./priority-controls"
);
const { EffectiveInputRow } = await import("./effective-input");
const { ConflictAlert, DraftBar } = await import("./draft-feedback");

const ROOT = "settings.sound.microphonePriority";
const has = (markup: string, text: string) => markup.includes(text);
const count = (markup: string, text: string) => markup.split(text).length - 1;
const noop = () => undefined;

const list = renderToStaticMarkup(
  <PriorityList
    order={["USB headset", "MacBook Pro Microphone", "Studio Display"]}
    availability={["unavailable", "available", "available"]}
    busy={false}
    onMove={noop}
    onRemove={noop}
    onFocusRequest={noop}
  />,
);

// @spec AC-001: List, add/remove and keyboard reorder controls — .specs/features/001-microphone-priority/spec.md#ac-001
describe("priority list (microphone-settings)", () => {
  test("carries the contract test IDs on every row", () => {
    expect(has(list, 'data-testid="microphone-settings-choices"')).toBe(true);
    for (const id of [
      "choice",
      "choice-rank",
      "choice-name",
      "choice-status",
      "choice-move-up",
      "choice-move-down",
      "choice-remove",
    ]) {
      expect(count(list, `data-testid="microphone-settings-${id}"`)).toBe(3);
    }
  });

  test("names each control after its microphone", () => {
    expect(
      has(
        list,
        `${ROOT}.moveUp{&quot;name&quot;:&quot;MacBook Pro Microphone&quot;}`,
      ),
    ).toBe(true);
    expect(
      has(list, `${ROOT}.remove{&quot;name&quot;:&quot;USB headset&quot;}`),
    ).toBe(true);
  });

  test("edge moves stay focusable but inert", () => {
    expect(count(list, 'aria-disabled="true"')).toBe(2);
    expect(has(list, " disabled=")).toBe(false);
    expect(count(list, "focus-visible:ring-2")).toBe(9);
  });

  test("shows each status as text", () => {
    expect(count(list, `${ROOT}.status.available`)).toBe(2);
    expect(count(list, `${ROOT}.status.unavailable`)).toBe(1);
    expect(has(list, "bg-yellow-500/20 text-yellow-400")).toBe(true);
  });

  test("every row control is a focus target named after its microphone", () => {
    expect(count(list, "data-focus-key=")).toBe(9);
    expect(has(list, 'data-focus-key="USB headset\nremove"')).toBe(true);
    expect(has(list, 'data-focus-key="Studio Display\nup"')).toBe(true);
  });

  test("ranks follow the order", () => {
    expect(has(list, 'data-testid="microphone-settings-choice-rank">1<')).toBe(
      true,
    );
    expect(has(list, 'data-testid="microphone-settings-choice-rank">3<')).toBe(
      true,
    );
  });
});

describe("empty list (microphone-settings-empty)", () => {
  const markup = renderToStaticMarkup(
    <PriorityList
      order={[]}
      availability={[]}
      busy={false}
      onMove={noop}
      onRemove={noop}
      onFocusRequest={noop}
    />,
  );
  test("explains that the system default is used", () => {
    for (const id of ["empty", "empty-title", "empty-body"]) {
      expect(has(markup, `data-testid="microphone-settings-empty-${id}"`)).toBe(
        true,
      );
    }
    expect(has(markup, `${ROOT}.emptyBody`)).toBe(true);
    expect(has(markup, "microphone-settings-choices")).toBe(false);
  });
});

const addProps = {
  enumerated: ["USB headset", "MacBook"],
  listingFailed: false,
  addError: null,
  accepted: false,
  busy: false,
  onAdd: noop,
  onOpen: noop,
};

// @spec AC-001: Duplicate choice refused with an actionable error — .specs/features/001-microphone-priority/spec.md#ac-001
describe("Add control", () => {
  test("empty: placeholder, enabled", () => {
    const markup = renderToStaticMarkup(<AddControl {...addProps} />);
    expect(has(markup, 'data-testid="microphone-settings-add"')).toBe(true);
    expect(has(markup, 'data-state="empty"')).toBe(true);
    expect(has(markup, `${ROOT}.addPlaceholder`)).toBe(true);
  });

  test("invalid: the duplicate stays shown with its message", () => {
    const addError = {
      name: "USB headset",
      message: {
        key: `${ROOT}.duplicateChoice`,
        params: { name: "USB headset" },
      },
    };
    const markup = renderToStaticMarkup(
      <>
        <AddControl {...addProps} addError={addError} />
        <AddErrorText error={addError} />
      </>,
    );
    expect(has(markup, 'data-state="invalid"')).toBe(true);
    expect(has(markup, "ring-red-500")).toBe(true);
    expect(has(markup, '<span class="truncate">USB headset</span>')).toBe(true);
    expect(has(markup, 'role="alert"')).toBe(true);
    expect(
      has(markup, 'data-testid="microphone-settings-error-add-error"'),
    ).toBe(true);
  });

  test("valid: a corrected choice was added, the control is empty again", () => {
    const markup = renderToStaticMarkup(<AddControl {...addProps} accepted />);
    expect(has(markup, 'data-state="valid"')).toBe(true);
    expect(has(markup, "ring-red-500")).toBe(false);
    expect(has(markup, `${ROOT}.addPlaceholder`)).toBe(true);
    expect(has(markup, 'data-focus-key="microphone-priority-add"')).toBe(true);
  });

  test("busy (write in flight or settings loading): disabled", () => {
    const markup = renderToStaticMarkup(<AddControl {...addProps} busy />);
    expect(count(markup, "disabled=")).toBe(1);
  });

  test("no device to add: disabled and named by the reason", () => {
    const markup = renderToStaticMarkup(
      <AddControl {...addProps} enumerated={[]} />,
    );
    expect(has(markup, `aria-label="${ROOT}.noDevices"`)).toBe(true);
    expect(count(markup, "disabled=")).toBe(1);
  });

  test("failed listing: disabled and named by the failure, not by an empty list", () => {
    const markup = renderToStaticMarkup(
      <AddControl {...addProps} enumerated={null} listingFailed />,
    );
    expect(has(markup, `aria-label="${ROOT}.refreshFailed"`)).toBe(true);
    expect(has(markup, `${ROOT}.noDevices`)).toBe(false);
    expect(count(markup, "disabled=")).toBe(1);
  });

  test("not listed yet: enabled, opening it lists the devices", () => {
    const markup = renderToStaticMarkup(
      <AddControl {...addProps} enumerated={null} />,
    );
    expect(has(markup, `aria-label="${ROOT}.addLabel"`)).toBe(true);
    expect(count(markup, "disabled=")).toBe(0);
  });
});

// @spec AC-002: Refresh line, failure pill and Retry — .specs/features/001-microphone-priority/spec.md#ac-002
describe("availability check line", () => {
  test("checked age beside Refresh availability", () => {
    const markup = renderToStaticMarkup(
      <CheckLine
        failed={false}
        observedAt={new Date().toISOString()}
        showChecked
        onRefresh={noop}
      />,
    );
    expect(has(markup, 'data-testid="microphone-settings-refresh"')).toBe(true);
    expect(has(markup, 'data-testid="microphone-settings-checked"')).toBe(true);
    expect(
      has(markup, `${ROOT}.checked{&quot;seconds&quot;:&quot;0&quot;}`),
    ).toBe(true);
  });

  test("failure pill and Retry (microphone-settings-error)", () => {
    const markup = renderToStaticMarkup(
      <CheckLine failed observedAt={null} showChecked onRefresh={noop} />,
    );
    expect(
      has(markup, 'data-testid="microphone-settings-error-refresh-error"'),
    ).toBe(true);
    expect(has(markup, 'data-testid="microphone-settings-error-retry"')).toBe(
      true,
    );
    expect(has(markup, "bg-red-500/20 text-red-400")).toBe(true);
    expect(has(markup, 'microphone-settings-refresh"')).toBe(false);
  });
});

const status = (
  effective: MicrophoneStatus["effective_input"],
): MicrophoneStatus => ({
  configured_order: ["USB headset", "MacBook Pro Microphone"],
  availability: [
    { name: "USB headset", status: "unavailable" },
    { name: "MacBook Pro Microphone", status: "available" },
  ],
  availability_observed_at: "2026-10-14T10:00:00Z",
  effective_input: effective,
  warning: null,
});

const view = (effective: MicrophoneStatus["effective_input"]) => ({
  status: status(effective),
  failed: false,
});

describe("In use now", () => {
  test("current input with its fallback reason", () => {
    const markup = renderToStaticMarkup(
      <EffectiveInputRow
        view={view({
          scope: "current",
          name: "MacBook Pro Microphone",
          reason: "fallback",
          capture_generation: 1,
        })}
      />,
    );
    for (const id of [
      "in-use",
      "in-use-title",
      "in-use-device",
      "in-use-reason",
    ]) {
      expect(has(markup, `data-testid="microphone-settings-${id}"`)).toBe(true);
    }
    expect(has(markup, ">MacBook Pro Microphone<")).toBe(true);
    expect(has(markup, `${ROOT}.reason.fallbackUnavailable`)).toBe(true);
    expect(has(markup, "bg-blue-500/20 text-blue-400")).toBe(true);
  });

  test("not used yet since launch", () => {
    const markup = renderToStaticMarkup(
      <EffectiveInputRow
        view={view({
          scope: "not_used_yet",
          name: null,
          reason: null,
          capture_generation: null,
        })}
      />,
    );
    expect(has(markup, `${ROOT}.inUse.notUsedYet`)).toBe(true);
    expect(has(markup, "microphone-settings-in-use-reason")).toBe(false);
  });
});

// @spec AC-007: Conflict banner with Reload, busy Save, failed write retry — .specs/features/001-microphone-priority/spec.md#ac-007
describe("draft feedback", () => {
  const bar = {
    visible: true,
    phase: "idle" as const,
    error: null,
    saveEnabled: true,
    onDiscard: noop,
    onSave: noop,
  };

  test("conflict banner (microphone-settings-error)", () => {
    const markup = renderToStaticMarkup(
      <ConflictAlert visible busy={false} onReload={noop} />,
    );
    expect(has(markup, 'role="alert"')).toBe(true);
    for (const id of ["conflict", "conflict-title", "reload"]) {
      expect(has(markup, `data-testid="microphone-settings-error-${id}"`)).toBe(
        true,
      );
    }
    expect(
      renderToStaticMarkup(
        <ConflictAlert visible={false} busy={false} onReload={noop} />,
      ),
    ).toBe("");
  });

  test("Reload stays focusable but inert while a write is in flight", () => {
    let reloads = 0;
    const markup = renderToStaticMarkup(
      <ConflictAlert visible busy onReload={() => (reloads += 1)} />,
    );
    expect(has(markup, 'aria-disabled="true"')).toBe(true);
    expect(has(markup, " disabled=")).toBe(false);
    expect(reloads).toBe(0);
  });

  test("Save enabled for a valid changed draft", () => {
    const markup = renderToStaticMarkup(<DraftBar {...bar} />);
    for (const id of ["draft", "draft-note", "discard", "save"]) {
      expect(has(markup, `data-testid="microphone-settings-${id}"`)).toBe(true);
    }
    expect(has(markup, 'data-state="enabled"')).toBe(true);
    expect(has(markup, `${ROOT}.draft.note`)).toBe(true);
  });

  test("Save busy while writing, cannot dispatch again", () => {
    const markup = renderToStaticMarkup(
      <DraftBar {...bar} phase="saving" saveEnabled={false} />,
    );
    expect(has(markup, 'data-state="loading"')).toBe(true);
    expect(has(markup, 'aria-busy="true"')).toBe(true);
    expect(has(markup, `${ROOT}.draft.saving`)).toBe(true);
    // Save keeps focus during the write: inactive through aria-disabled, never `disabled`.
    expect(has(markup, 'aria-disabled="true" aria-busy="true"')).toBe(true);
    expect(count(markup, " disabled=")).toBe(1);
  });

  test("failed write: error and Retry", () => {
    const markup = renderToStaticMarkup(
      <DraftBar
        {...bar}
        phase="failed"
        error={{ key: "errors.settings.persistenceFailed" }}
      />,
    );
    expect(has(markup, 'data-state="error"')).toBe(true);
    expect(has(markup, "errors.settings.persistenceFailed")).toBe(true);
    expect(has(markup, `>${ROOT}.retry<`)).toBe(true);
  });

  test("invalid draft: Save disabled with the validation error", () => {
    const markup = renderToStaticMarkup(
      <DraftBar
        {...bar}
        saveEnabled={false}
        error={{ key: `${ROOT}.errors.duplicate` }}
      />,
    );
    expect(has(markup, 'data-state="disabled"')).toBe(true);
    expect(has(markup, `${ROOT}.errors.duplicate`)).toBe(true);
  });

  test("hidden without unsaved changes", () => {
    expect(renderToStaticMarkup(<DraftBar {...bar} visible={false} />)).toBe(
      "",
    );
  });
});
