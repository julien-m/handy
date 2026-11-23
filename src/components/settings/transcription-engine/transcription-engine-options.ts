// @spec FR-002: Engine tabs model — .specs/features/024-transcription-engine/spec.md#fr-002
/** Pure model of the transcription engine tabs (Local / API) and their engines. */
import type { TranscriptionEngine } from "@/bindings";

/** Tab grouping the engines: on-device models or online APIs. */
export type EngineTab = "local" | "api";

/** Tabs in display order. */
export const ENGINE_TABS: readonly EngineTab[] = ["local", "api"];

/** Online API engines, in display order; future providers are added here. */
export const API_ENGINES: readonly TranscriptionEngine[] = ["chatgpt"];

const STORABLE_ENGINES: readonly TranscriptionEngine[] = [
  "local",
  ...API_ENGINES,
];

/**
 * Read the stored engine, treating anything unknown as the local engine (the default).
 * @param stored value from the settings store
 * @returns a known engine
 */
export const selectedEngine = (stored: unknown): TranscriptionEngine =>
  STORABLE_ENGINES.find((engine) => engine === stored) ?? "local";

/**
 * The tab that holds an engine.
 * @param engine a known engine
 * @returns "local" for the local model, "api" for online APIs
 */
export const tabOfEngine = (engine: TranscriptionEngine): EngineTab =>
  engine === "local" ? "local" : "api";

/**
 * Whether dictations are transcribed by the selected local model; with an API
 * engine the local model is only the fallback and cannot be switched from the list.
 * @param stored value from the settings store
 * @returns true for the local engine
 */
export const usesLocalModel = (stored: unknown): boolean =>
  selectedEngine(stored) === "local";

/**
 * The tab reached with an arrow key from `current`, wrapping around.
 * @param current the focused tab
 * @param step +1 (next) or -1 (previous)
 * @returns the tab to focus and show
 */
export const adjacentTab = (current: EngineTab, step: 1 | -1): EngineTab => {
  const index = ENGINE_TABS.indexOf(current);
  return ENGINE_TABS[(index + step + ENGINE_TABS.length) % ENGINE_TABS.length];
};
