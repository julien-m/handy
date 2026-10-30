/**
 * Shared fixtures of the recording indicator markup tests (features 005, 006, 007): English
 * translations, a destination session, a prompt library, the controls the stack receives and
 * the render of the stack around a stand-in Handy pill. Used by `indicator-render.test.tsx` and
 * `indicator-gates-render.test.tsx`; not part of the application bundle.
 */
import i18n from "i18next";
import { createElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { initReactI18next } from "react-i18next";
import type {
  DestinationSnapshot,
  DictationPromptView,
  PromptLibraryView,
} from "@/bindings";
import en from "@/i18n/locales/en/translation.json";
import { badgeView } from "./destination-model";
import { IndicatorStack } from "./indicator-stack";
import type { OverlayNotice } from "./notice-model";
import type {
  DestinationControls,
  DestinationMenuState,
} from "./use-destination";
import type { LibraryState, PromptControls } from "./use-dictation-prompt";

/** English translations for the rendered texts (call once per test file, in `beforeAll`). */
export const initIndicatorI18n = async (): Promise<void> => {
  await i18n.use(initReactI18next).init({
    lng: "en",
    resources: { en: { translation: en } },
    interpolation: { escapeValue: false },
  });
};

export const noop = () => undefined;
export const PILL =
  '<div class="scard compact" data-testid="recording-indicator-pill">pill</div>';

export const session = (
  selection: DestinationSnapshot["selection"],
  phase = "recording",
) =>
  ({
    dictation_id: 7,
    destination_generation: 1,
    selection_version: 1,
    phase,
    return_enabled: true,
    support: { state: "supported" },
    selection,
    source: selection.kind === "application" ? "captured" : "frontmost_at_end",
    pin: null,
    disposition: "pending",
  }) as DestinationSnapshot;
export const chatgpt = {
  kind: "application",
  display_name: "ChatGPT",
  icon_ref: null,
} as const;

export const destination = (
  snapshot: DestinationSnapshot | null,
  menu: DestinationMenuState = { status: "closed" },
): DestinationControls => ({
  snapshot,
  latest: { current: snapshot },
  badge: badgeView(snapshot),
  menu,
  feedbackKey: null,
  toggleMenu: noop,
  closeMenu: noop,
  choose: noop,
  remove: noop,
  pin: noop,
  unpin: noop,
  toggleFavorite: noop,
});

export const library: PromptLibraryView = {
  default_prompt_id: "min",
  post_process_enabled: true,
  prompts: [
    {
      id: "min",
      name: "Minimal correction",
      prompt: "text",
      provider_override: null,
      starter: true,
      issues: [],
      shortcuts: ["ctrl+option+m"],
      readiness: {
        provider_id: null,
        model: null,
        is_override: false,
        ready: true,
        missing: null,
      },
    },
  ],
};
export const promptView: DictationPromptView = {
  session_id: 3,
  prompt_id: "min",
  prompt_name: "Minimal correction",
  selectable: true,
  remembered: false,
};
export const prompt = (
  view: DictationPromptView | null,
  menuOpen = false,
  state: LibraryState = { status: "ready", library },
): PromptControls => ({
  view,
  library: state,
  menuOpen,
  feedbackKey: null,
  openMenu: noop,
  closeMenu: noop,
  select: noop,
});

export const render = (
  options: {
    destination?: DestinationControls;
    prompt?: PromptControls;
    notices?: OverlayNotice[];
    showIndicator?: boolean;
    form?: "compact" | "live";
  } = {},
): string => {
  const pill: ReactNode = createElement(
    "div",
    { className: "scard compact", "data-testid": "recording-indicator-pill" },
    "pill",
  );
  return renderToStaticMarkup(
    createElement(IndicatorStack, {
      position: "bottom",
      direction: "ltr",
      form: options.form ?? "compact",
      showIndicator: options.showIndicator ?? true,
      recording: true,
      dictation: 1,
      destination: options.destination ?? destination(null),
      prompt: options.prompt ?? prompt(null),
      notices: options.notices ?? [],
      osType: "macos",
      children: pill,
    }),
  );
};
