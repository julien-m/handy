// Renders the microphone priority editor states to static HTML with Handy's built CSS, for a
// visual comparison with the mockups in headless Chromium (not a native window proof).
import { mock, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { readFileSync, writeFileSync, readdirSync } from "node:fs";

const REPO = "/Users/julienm/projects/handy-wt/ui-001-microphones";
const OUT = "/private/tmp/claude-501/-Users-julienm-projects-handy/6c63dd2f-66f1-4748-8d50-174f39c2881a/scratchpad/ui-001/render";
const en = JSON.parse(readFileSync(`${REPO}/src/i18n/locales/en/translation.json`, "utf8"));
const lookup = (key: string, params?: Record<string, string>) => {
  let value: any = en;
  for (const part of key.split(".")) value = value?.[part];
  let text = typeof value === "string" ? value : key;
  for (const [name, v] of Object.entries(params ?? {})) text = text.replaceAll(`{{${name}}}`, String(v));
  return text;
};
mock.module("react-i18next", () => ({ useTranslation: () => ({ t: lookup }) }));
mock.module("@/hooks/useSettings", () => ({ useSettings: () => ({}) }));
mock.module("@/stores/settingsStore", () => ({ useSettingsStore: () => null }));
mock.module("@/hooks/use-microphone-status", () => ({ useMicrophoneStatus: () => null }));
mock.module("sonner", () => ({ toast: {} }));

const { PrioritySection } = await import(`${REPO}/src/components/settings/microphone-priority/priority-editor.tsx`);
const { EffectiveInputRow } = await import(`${REPO}/src/components/settings/microphone-priority/effective-input.tsx`);
const { ConflictAlert, DraftBar } = await import(`${REPO}/src/components/settings/microphone-priority/draft-feedback.tsx`);
const { SettingsGroup } = await import(`${REPO}/src/components/ui/SettingsGroup.tsx`);
const { SettingContainer } = await import(`${REPO}/src/components/ui/SettingContainer.tsx`);
const { Dropdown } = await import(`${REPO}/src/components/ui/Dropdown.tsx`);
const { EMPTY_DRAFT, editOrder, addChoice } = await import(`${REPO}/src/components/settings/microphone-priority/draft-state.ts`);
const { mergeStatusRead, EMPTY_STATUS } = await import(`${REPO}/src/components/settings/microphone-priority/availability.ts`);

const css = readdirSync(`${REPO}/dist/assets`).filter((f) => f.startsWith("main-") && f.endsWith(".css"))[0];
const noop = () => undefined;
const asyncNoop = async () => undefined;

const clamshellRow = (value: string) => (
  <SettingContainer title={lookup("settings.sound.microphonePriority.clamshell.title")} description={lookup("settings.sound.microphonePriority.clamshell.description")} descriptionMode="inline" grouped>
    <Dropdown options={[{ value, label: value }]} selectedValue={value} onSelect={noop} />
  </SettingContainer>
);

const page = (name: string, opts: any) => {
  const saved = { order: opts.saved, revision: 4 };
  const statusView = opts.failed ? { status: opts.status, failed: true } : mergeStatusRead(EMPTY_STATUS, opts.status);
  const draft = opts.draft ?? EMPTY_DRAFT;
  const order = draft.order ?? saved.order;
  const editor: any = {
    saved, draft, order, dirty: opts.dirty ?? false, saveEnabled: true,
    enumerated: ["USB headset", "MacBook Pro Microphone", "Studio Display Microphone"],
    status: { ...statusView, refresh: asyncNoop },
    move: noop, remove: noop, add: noop, reset: noop, discard: noop, reload: asyncNoop, save: asyncNoop, refreshAll: asyncNoop,
  };
  const body = renderToStaticMarkup(
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <div className="space-y-6">
        <ConflictAlert visible={draft.conflict} onReload={noop} />
        <SettingsGroup title={lookup("settings.sound.title")}>
          <PrioritySection editor={editor} />
          <EffectiveInputRow view={statusView} />
          {clamshellRow(opts.clamshell)}
        </SettingsGroup>
        <DraftBar visible={editor.dirty} phase={draft.savePhase} error={draft.saveError} saveEnabled onDiscard={noop} onSave={noop} />
      </div>
    </div>,
  );
  const html = `<!doctype html><html><head><meta charset="utf-8"><link rel="stylesheet" href="${REPO}/dist/assets/${css}"></head><body class="bg-background text-text"><div class="p-4" style="width:515px">${body}</div></body></html>`;
  writeFileSync(`${OUT}/${name}.html`, html);
};

const now = new Date(Date.now() - 3000).toISOString();
const st = (availability: any[], effective: any, warning: any = null) => ({
  configured_order: availability.map((a) => a.name), availability, availability_observed_at: now, effective_input: effective, warning,
});

test("render", () => {
  page("microphone-settings", {
    saved: ["USB headset", "MacBook Pro Microphone", "Studio Display Microphone"],
    status: st([{ name: "USB headset", status: "unavailable" }, { name: "MacBook Pro Microphone", status: "available" }, { name: "Studio Display Microphone", status: "available" }],
      { scope: "current", name: "MacBook Pro Microphone", reason: "fallback", capture_generation: 1 }),
    draft: { ...editOrder(EMPTY_DRAFT, { order: ["MacBook Pro Microphone", "USB headset", "Studio Display Microphone"], revision: 4 }, ["USB headset", "MacBook Pro Microphone", "Studio Display Microphone"]) },
    dirty: true, clamshell: "Studio Display",
  });
  page("microphone-settings-empty", {
    saved: [],
    status: st([], { scope: "not_used_yet", name: null, reason: null, capture_generation: null }),
    clamshell: "None",
  });
  const base = { order: ["USB headset", "MacBook Pro Microphone"], revision: 4 };
  let draft = editOrder(EMPTY_DRAFT, base, ["USB headset", "MacBook Pro Microphone"]);
  draft = addChoice(draft, base, "USB headset");
  draft = { ...draft, conflict: true };
  page("microphone-settings-error", {
    saved: ["MacBook Pro Microphone"],
    status: st([{ name: "USB headset", status: "unknown" }, { name: "MacBook Pro Microphone", status: "unknown" }], { scope: "not_used_yet", name: null, reason: null, capture_generation: null }, "enumeration_failed"),
    failed: true, draft, dirty: true, clamshell: "None",
  });
});
