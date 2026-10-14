/**
 * Vocabulary editor state (feature 003): the saved list with its revision, the status for the
 * selected model, the single pending commit and the paste preview. Every commit carries the
 * revision the UI shows (Apply: the one its preview was computed on); stale answers (older
 * revision, superseded request, edited paste text) are discarded. The backend commands are injected so the rules run without Tauri.
 */
import { createStore, type StoreApi } from "zustand/vanilla";
import type {
  Result,
  SettingsCommandError,
  VocabularyImportMode,
  VocabularyImportPreview,
  VocabularyList,
  VocabularyStatusReport,
} from "@/bindings";
import { isSettingsCommandError } from "./settings-errors";

/** Backend commands the editor uses (the generated `commands` object satisfies it). */
export interface VocabularyApi {
  getVocabulary: () => Promise<VocabularyList>;
  getVocabularyStatus: () => Promise<
    Result<VocabularyStatusReport, SettingsCommandError>
  >;
  addVocabularyTerm: (
    term: string,
    expectedRevision: number,
  ) => Promise<Result<VocabularyList, SettingsCommandError>>;
  removeVocabularyOccurrence: (
    index: number,
    term: string,
    expectedRevision: number,
  ) => Promise<Result<VocabularyList, SettingsCommandError>>;
  previewVocabularyImport: (
    input: string,
    mode: VocabularyImportMode,
  ) => Promise<VocabularyImportPreview>;
  applyVocabularyImport: (
    input: string,
    mode: VocabularyImportMode,
    expectedRevision: number,
  ) => Promise<Result<VocabularyList, SettingsCommandError>>;
}

/** A commit the editor can retry. */
export type VocabularyOperation =
  | { kind: "add"; term: string }
  | { kind: "remove"; index: number; term: string }
  | { kind: "apply"; input: string };

/** Behavioral AC visual states of the editor. */
export type EditorPhase = "idle" | "loading" | "error" | "success";

export type StatusState =
  | { kind: "loading" }
  | { kind: "ready"; report: VocabularyStatusReport }
  | { kind: "failed"; error: SettingsCommandError | null };

export interface VocabularyFailure {
  operation: VocabularyOperation;
  error: SettingsCommandError | null;
}

export interface VocabularyState {
  list: VocabularyList | null;
  listFailed: boolean;
  status: StatusState;
  pending: VocabularyOperation | null;
  phase: EditorPhase;
  failure: VocabularyFailure | null;
  draft: string;
  pasteOpen: boolean;
  pasteText: string;
  preview: VocabularyImportPreview | null;
  /** Text the current preview answers. */
  previewText: string | null;
}

export interface VocabularyActions {
  loadList: () => Promise<void>;
  loadStatus: () => Promise<void>;
  setDraft: (draft: string) => void;
  add: () => Promise<boolean>;
  remove: (index: number, term: string) => Promise<boolean>;
  openPaste: () => void;
  closePaste: () => void;
  setPasteText: (text: string) => void;
  loadPreview: () => Promise<void>;
  apply: () => Promise<boolean>;
  retry: () => Promise<boolean>;
}

export type VocabularyStore = StoreApi<VocabularyState & VocabularyActions>;

/** The mode the paste box uses: new terms are appended, the saved list is kept. */
export const PASTE_MODE: VocabularyImportMode = "merge";

const INITIAL: VocabularyState = {
  list: null,
  listFailed: false,
  status: { kind: "loading" },
  pending: null,
  phase: "idle",
  failure: null,
  draft: "",
  pasteOpen: false,
  pasteText: "",
  preview: null,
  previewText: null,
};

/** What the action helpers share: the API, the store accessors and the request counters. */
interface Context {
  api: VocabularyApi;
  get: () => VocabularyState & VocabularyActions;
  set: (partial: Partial<VocabularyState>) => void;
  /** Counters that let only the latest request of each kind update the state. */
  seq: { list: number; status: number; preview: number };
}

/**
 * Whether a failed commit leaves the shown list possibly stale, so it must be reread before a
 * retry: conflicts, missing occurrences, unconfirmed or committed outcomes, untyped failures.
 * @param error typed refusal, or null for a failure without one
 * @returns true when the list (and the paste preview) must be reloaded
 */
export const needsReload = (error: SettingsCommandError | null): boolean =>
  error === null ||
  error.code === "revision_conflict" ||
  error.code === "not_found" ||
  error.outcome !== null;

const toError = (error: unknown): SettingsCommandError | null =>
  isSettingsCommandError(error) ? error : null;

/**
 * Adopt a list unless a newer revision is already shown.
 * @returns true when `incoming` replaced the shown list
 */
const adoptList = (
  { get, set }: Context,
  incoming: VocabularyList,
): boolean => {
  const shown = get().list;
  if (shown !== null && incoming.revision < shown.revision) return false;
  set({ list: incoming, listFailed: false });
  return true;
};

const loadList = async (ctx: Context) => {
  const { api, set, seq } = ctx;
  const ticket = ++seq.list;
  try {
    const list = await api.getVocabulary();
    if (ticket === seq.list) adoptList(ctx, list);
  } catch (error) {
    // The editor keeps the last list it showed (or none) and shows the error with Retry;
    // nothing is written.
    console.error("Vocabulary read failed:", error);
    if (ticket === seq.list) set({ listFailed: true });
  }
};

const loadStatus = async ({ api, set, seq }: Context) => {
  const ticket = ++seq.status;
  let status: StatusState;
  try {
    const result = await api.getVocabularyStatus();
    status =
      result.status === "ok"
        ? { kind: "ready", report: result.data }
        : { kind: "failed", error: result.error };
  } catch (error) {
    status = { kind: "failed", error: toError(error) };
  }
  if (ticket === seq.status) set({ status });
};

/** Send one commit for `operation` at `revision`. */
const send = (
  api: VocabularyApi,
  operation: VocabularyOperation,
  revision: number,
): Promise<Result<VocabularyList, SettingsCommandError>> => {
  switch (operation.kind) {
    case "add":
      return api.addVocabularyTerm(operation.term, revision);
    case "remove":
      return api.removeVocabularyOccurrence(
        operation.index,
        operation.term,
        revision,
      );
    default:
      return api.applyVocabularyImport(operation.input, PASTE_MODE, revision);
  }
};

const succeed = (
  ctx: Context,
  operation: VocabularyOperation,
  list: VocabularyList,
) => {
  const { get, set } = ctx;
  adoptList(ctx, list);
  set({ pending: null, phase: "success", failure: null });
  if (operation.kind === "add") set({ draft: "" });
  if (operation.kind === "apply") {
    set({ pasteOpen: false, pasteText: "", preview: null, previewText: null });
  }
  void get().loadStatus();
};

const fail = async (
  { get, set }: Context,
  operation: VocabularyOperation,
  error: SettingsCommandError | null,
) => {
  set({ pending: null, phase: "error", failure: { operation, error } });
  if (!needsReload(error)) return;
  await get().loadList();
  if (operation.kind === "apply") await get().loadPreview();
};

/**
 * Dispatch `operation` unless another commit is pending.
 * @param revision expected revision; defaults to the shown list's (Apply passes its preview's)
 * @returns true when the backend committed it
 */
const commit = async (
  ctx: Context,
  operation: VocabularyOperation,
  revision?: number,
): Promise<boolean> => {
  const { api, get, set } = ctx;
  const { pending, list } = get();
  if (pending !== null || list === null) return false;
  set({ pending: operation, phase: "loading" });
  let result: Result<VocabularyList, SettingsCommandError>;
  try {
    result = await send(api, operation, revision ?? list.revision);
  } catch (error) {
    await fail(ctx, operation, toError(error));
    return false;
  }
  if (result.status === "ok") {
    succeed(ctx, operation, result.data);
    return true;
  }
  await fail(ctx, operation, result.error);
  return false;
};

const loadPreview = async ({ api, get, set, seq }: Context) => {
  const ticket = ++seq.preview;
  const text = get().pasteText;
  if (text === "") {
    set({ preview: null, previewText: null });
    return;
  }
  try {
    const preview = await api.previewVocabularyImport(text, PASTE_MODE);
    // Discard an answer to older text or superseded by a newer request.
    if (ticket === seq.preview && get().pasteText === text) {
      set({ preview, previewText: text });
    }
  } catch (error) {
    console.error("Vocabulary preview failed:", error);
    if (ticket === seq.preview) set({ preview: null, previewText: null });
  }
};

/**
 * Apply the previewed text at the revision the preview was computed on: if the list changed
 * since, the backend answers a revision conflict and the preview is reread, so Apply never
 * commits a merge other than the one shown.
 */
const apply = (ctx: Context): Promise<boolean> => {
  const { preview, previewText, pasteText } = ctx.get();
  if (!preview || !preview.applicable || previewText !== pasteText) {
    return Promise.resolve(false);
  }
  const operation: VocabularyOperation = { kind: "apply", input: pasteText };
  return commit(ctx, operation, preview.revision);
};

const retry = (ctx: Context): Promise<boolean> => {
  const failure = ctx.get().failure;
  if (!failure) return Promise.resolve(false);
  if (failure.operation.kind === "apply") return apply(ctx);
  return commit(ctx, failure.operation);
};

/**
 * Create a vocabulary editor store.
 * @param api backend commands (the generated `commands` in the app, stubs in tests)
 * @returns a vanilla zustand store; nothing is loaded until `loadList`/`loadStatus` run
 */
export const createVocabularyStore = (api: VocabularyApi): VocabularyStore =>
  createStore<VocabularyState & VocabularyActions>()((set, get) => {
    const ctx: Context = {
      api,
      get,
      set,
      seq: { list: 0, status: 0, preview: 0 },
    };
    return {
      ...INITIAL,
      loadList: () => loadList(ctx),
      loadStatus: () => loadStatus(ctx),
      setDraft: (draft) => set(draftChange(get(), draft)),
      add: () => commit(ctx, { kind: "add", term: get().draft }),
      remove: (index, term) => commit(ctx, { kind: "remove", index, term }),
      openPaste: () => set({ pasteOpen: true, phase: "idle", failure: null }),
      closePaste: () => set(closedPaste(get())),
      setPasteText: (pasteText) => set({ pasteText }),
      loadPreview: () => loadPreview(ctx),
      apply: () => apply(ctx),
      retry: () => retry(ctx),
    };
  });

/** Typing clears a refusal of the typed term (the Behavioral AC "error clears"). */
const draftChange = (
  state: VocabularyState,
  draft: string,
): Partial<VocabularyState> =>
  state.failure?.operation.kind === "add"
    ? { draft, failure: null, phase: "idle" }
    : { draft };

const closedPaste = (state: VocabularyState): Partial<VocabularyState> => ({
  pasteOpen: false,
  pasteText: "",
  preview: null,
  previewText: null,
  ...(state.failure?.operation.kind === "apply"
    ? { failure: null, phase: "idle" as const }
    : {}),
});
