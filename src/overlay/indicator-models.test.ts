/**
 * Rules of the recording indicator additions: destination badge and menu (006), prompt pill
 * and menu (005), notices (005, 006, 007) and the window extent. Run with
 * `bun run test:indicator`.
 */
import { describe, expect, test } from "bun:test";
import type {
  DestinationMenu,
  DestinationSnapshot,
  DictationPromptView,
  ListedPrompt,
  PromptLibraryView,
  PromptProcessingFailed,
  SessionPhase,
} from "@/bindings";
import en from "@/i18n/locales/en/translation.json";
import {
  badgeView,
  menuMatches,
  menuSections,
  monogram,
  newerSnapshot,
  refusalKey,
  safeIconSource,
} from "./destination-model";
import {
  addNotice,
  aiFailureNotice,
  handOverNotice,
  holdRemaining,
  liveNotices,
  nextExpiry,
  NOTICE_MIN_MS,
} from "./notice-model";
import {
  compactShortcut,
  defaultPromptName,
  newerPromptView,
  pillVisible,
  promptMenuRows,
  promptRefusalKey,
  refusalClosesMenu,
} from "./prompt-model";
import { extentChanged, extentRequest } from "./use-overlay-extent";

const app = { kind: "application", display_name: "ChatGPT", icon_ref: null };
const snapshot = (
  over: Partial<DestinationSnapshot> = {},
): DestinationSnapshot => ({
  dictation_id: 7,
  destination_generation: 1,
  selection_version: 1,
  phase: "recording",
  return_enabled: true,
  support: { state: "supported" },
  selection: app as DestinationSnapshot["selection"],
  source: "captured",
  pin: null,
  disposition: "pending",
  ...over,
});

describe("destination badge (006 AC-003, AC-004)", () => {
  test("gate closed: no session, nothing is shown", () => {
    expect(badgeView(null)).toEqual({ kind: "hidden" });
  });
  test("setting off, unsupported system or ended session hide the badge", () => {
    expect(badgeView(snapshot({ return_enabled: false })).kind).toBe("hidden");
    const unsupported = snapshot({
      support: { state: "unsupported", reason: "frontmost_unreadable" },
    });
    expect(badgeView(unsupported).kind).toBe("hidden");
    for (const phase of ["finished", "cancelled"] as SessionPhase[]) {
      expect(badgeView(snapshot({ phase })).kind).toBe("hidden");
    }
  });
  test("an application shows icon and name, changeable until insertion", () => {
    expect(badgeView(snapshot())).toEqual({
      kind: "application",
      name: "ChatGPT",
      iconRef: null,
      canChange: true,
      pinned: false,
    });
    expect(badgeView(snapshot({ phase: "processing" }))).toMatchObject({
      canChange: true,
    });
    expect(badgeView(snapshot({ phase: "inserting" }))).toMatchObject({
      canChange: false,
    });
  });
  test("without destination the badge says Frontmost app", () => {
    const view = badgeView(
      snapshot({ selection: { kind: "frontmost_at_end" } }),
    );
    expect(view).toEqual({ kind: "frontmost", canChange: true });
  });
});

describe("destination snapshot ordering", () => {
  test("late events of an older dictation, generation or version are ignored", () => {
    const current = snapshot({ dictation_id: 8, selection_version: 3 });
    expect(newerSnapshot(current, snapshot())).toBe(current);
    const olderVersion = snapshot({ dictation_id: 8, selection_version: 2 });
    expect(newerSnapshot(current, olderVersion)).toBe(current);
    const newerGeneration = snapshot({
      dictation_id: 8,
      destination_generation: 2,
    });
    expect(newerSnapshot(current, newerGeneration)).toBe(newerGeneration);
  });
  test("a later phase of the same version wins, an earlier one does not", () => {
    const inserting = snapshot({ phase: "inserting" });
    expect(newerSnapshot(snapshot(), inserting)).toBe(inserting);
    const recording = snapshot();
    expect(newerSnapshot(inserting, recording)).toBe(inserting);
    expect(newerSnapshot(null, recording)).toBe(recording);
  });
});

const menu: DestinationMenu = {
  dictation_id: 7,
  selection_version: 1,
  frontmost_at_end_current: false,
  entries: [
    {
      token: 2,
      display_name: "ChatGPT",
      icon_ref: null,
      current: true,
      frontmost: false,
      pinned: false,
    },
    {
      token: 1,
      display_name: "Comet",
      icon_ref: null,
      current: false,
      frontmost: true,
      pinned: false,
    },
  ],
};

describe("destination menu (006 AC-004)", () => {
  test("the application frontmost now comes first, then the others", () => {
    const sections = menuSections(menu);
    expect(sections.frontmost?.display_name).toBe("Comet");
    expect(sections.others.map((entry) => entry.display_name)).toEqual([
      "ChatGPT",
    ]);
    expect(sections.frontmostAtEndCurrent).toBe(false);
  });
  test("a menu of another version no longer matches the session", () => {
    expect(menuMatches(menu, snapshot())).toBe(true);
    expect(menuMatches(menu, snapshot({ selection_version: 2 }))).toBe(false);
    expect(menuMatches(menu, null)).toBe(false);
  });
  test("refusals are translated and the keys exist", () => {
    const destination = en.overlay.destination.refused;
    expect(refusalKey("stale")).toBe("overlay.destination.refused.stale");
    expect(Object.keys(destination).sort()).toEqual([
      "frozen",
      "stale",
      "unavailable",
    ]);
  });
  test("icons render only inline images, else a monogram", () => {
    expect(safeIconSource("data:image/png;base64,AA")).toBe(
      "data:image/png;base64,AA",
    );
    expect(safeIconSource("https://example.com/icon.png")).toBeNull();
    expect(safeIconSource(null)).toBeNull();
    expect(monogram(" comet")).toBe("C");
    expect(monogram("")).toBe("?");
  });
});

const listed = (
  id: string,
  name: string,
  shortcuts: string[] = [],
  issues = 0,
) =>
  ({
    id,
    name,
    prompt: "text",
    provider_override: null,
    starter: true,
    issues: Array(issues).fill("prompt_name_empty"),
    shortcuts,
    readiness: {
      provider_id: null,
      model: null,
      is_override: false,
      ready: true,
      missing: null,
    },
  }) as ListedPrompt;
const library: PromptLibraryView = {
  default_prompt_id: "min",
  post_process_enabled: true,
  prompts: [
    listed("min", "Minimal correction", ["ctrl+option+m"]),
    listed("broken", "Broken", [], 1),
  ],
};
const view = (
  over: Partial<DictationPromptView> = {},
): DictationPromptView => ({
  session_id: 3,
  prompt_id: "min",
  prompt_name: "Minimal correction",
  selectable: true,
  ...over,
});

describe("prompt pill and menu (005 AC-003)", () => {
  test("the pill shows only while recording with AI processing on", () => {
    expect(pillVisible(view(), true)).toBe(true);
    expect(pillVisible(view(), false)).toBe(false);
    expect(pillVisible(view({ selectable: false }), true)).toBe(false);
    expect(pillVisible(null, true)).toBe(false);
  });
  test("shortcut badges use modifier glyphs on macOS, Handy words elsewhere", () => {
    expect(compactShortcut("ctrl+option+m", "macos")).toBe("⌃ ⌥ M");
    expect(compactShortcut("command+shift_left+space", "macos")).toBe(
      "⌘ ⇧ Space",
    );
    expect(compactShortcut("ctrl+alt+m", "windows")).toBe("Ctrl + Alt + M");
  });
  test("None comes first, the current prompt is marked, invalid prompts are disabled", () => {
    const rows = promptMenuRows(library, view(), "macos");
    expect(rows.map((row) => row.id)).toEqual([null, "min", "broken"]);
    expect(rows.map((row) => row.current)).toEqual([false, true, false]);
    expect(rows[1].shortcut).toBe("⌃ ⌥ M");
    expect(rows[2].usable).toBe(false);
    const none = promptMenuRows(library, view({ prompt_id: null }), "macos");
    expect(none[0].current).toBe(true);
  });
  test("the note names the default prompt, untouched by the choice", () => {
    expect(defaultPromptName(library)).toBe("Minimal correction");
    expect(
      defaultPromptName({ ...library, default_prompt_id: null }),
    ).toBeNull();
  });
  test("an older session never replaces the current view; null clears it", () => {
    const current = view({ session_id: 5 });
    expect(newerPromptView(current, view({ session_id: 4 }))).toBe(current);
    expect(newerPromptView(current, null)).toBeNull();
  });
  test("each refusal has a translated message; ended sessions close the menu", () => {
    const keys = Object.keys(en.overlay.prompt.refused).sort();
    expect(keys).toEqual([
      "processingOff",
      "promptMissing",
      "promptNotUsable",
      "sessionEnded",
    ]);
    expect(promptRefusalKey("prompt_not_usable")).toBe(
      "overlay.prompt.refused.promptNotUsable",
    );
    expect(refusalClosesMenu("session_ended")).toBe(true);
    expect(refusalClosesMenu("prompt_missing")).toBe(false);
  });
});

const failure: PromptProcessingFailed = {
  session_id: 3,
  prompt_name: "Minimal correction",
  reason: "timeout",
  notice_ms: 1000,
};

describe("indicator notices (005 AC-005, 007 AC-012)", () => {
  test("the AI notice lasts at least 4 s and never claims a successful insertion", () => {
    const notice = aiFailureNotice(failure, 100);
    expect(notice.until).toBe(100 + NOTICE_MIN_MS);
    expect(notice.testId).toBe("recording-indicator-ai-failed-notice");
    expect(en.overlay.fallbackNotice).toContain("failed");
    expect(en.overlay.fallbackNotice).not.toMatch(/success|succeeded/i);
  });
  test("a newer notice of the same kind replaces the older one; expired ones drop", () => {
    const first = addNotice([], aiFailureNotice(failure, 0), 0);
    const second = addNotice(first, handOverNotice(10), 10);
    expect(second.map((notice) => notice.kind)).toEqual([
      "ai_failed",
      "hand_over",
    ]);
    const replaced = addNotice(second, aiFailureNotice(failure, 20), 20);
    expect(replaced.map((notice) => notice.kind)).toEqual([
      "hand_over",
      "ai_failed",
    ]);
    expect(
      liveNotices(replaced, NOTICE_MIN_MS + 15).map((n) => n.kind),
    ).toEqual(["ai_failed"]);
  });
  test("the window is held for the longest notice and released when none is left", () => {
    const notices = addNotice(
      [handOverNotice(0)],
      aiFailureNotice(failure, 1000),
      1000,
    );
    expect(holdRemaining(notices, 1000)).toBe(NOTICE_MIN_MS);
    expect(nextExpiry(notices, 1000)).toBe(NOTICE_MIN_MS - 1000);
    expect(holdRemaining([], 0)).toBe(0);
    expect(nextExpiry([], 0)).toBeNull();
  });
});

describe("window extent", () => {
  test("requests are rounded up with slack so transitions send few of them", () => {
    expect(extentRequest(297.4, 41.2)).toEqual({ width: 304, height: 48 });
    expect(extentRequest(-5, 0)).toEqual({ width: 8, height: 8 });
    expect(extentChanged(null, { width: 8, height: 8 })).toBe(true);
    expect(
      extentChanged({ width: 8, height: 8 }, { width: 8, height: 8 }),
    ).toBe(false);
  });
});
