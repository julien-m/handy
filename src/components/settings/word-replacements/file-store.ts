/**
 * Rule files of the replacement editor (feature 002, AC-004/AC-005): the native import dialog,
 * the import preview in merge or replace mode, its atomic Apply at the preview's revision, and
 * the export of the saved rules. Paths stay native; the editor only sees file names and text.
 */
import { create } from "zustand";
import {
  commands,
  type ImportMode,
  type ReplacementImportFile,
  type ReplacementPreviewView,
} from "@/bindings";
import {
  type CommitFailure,
  type FileRefusal,
  UNEXPECTED_FAILURE,
  canApplyPreview,
  commitFailure,
  exportFailureKey,
  fileRefusal,
} from "./preview-model";
import { useReplacementEditor } from "./editor-store";

/** Import dialog lifecycle. */
export type ImportPhase =
  | "idle"
  | "picking"
  | "previewing"
  | "ready"
  | "applying";

/** A refused import file, named in its alert. */
export interface ImportRefusal {
  readonly fileName: string;
  readonly kind: FileRefusal;
}

/** Outcome of an export, for the confirmation or error message. */
export type ExportOutcome =
  | { readonly status: "cancelled" }
  | {
      readonly status: "exported";
      readonly fileName: string;
      readonly count: number;
      readonly durable: boolean;
    }
  | {
      readonly status: "failed";
      readonly fileName: string;
      readonly key: string;
    };

export interface FileStore {
  phase: ImportPhase;
  file: ReplacementImportFile | null;
  mode: ImportMode;
  preview: ReplacementPreviewView | null;
  refusal: ImportRefusal | null;
  failure: CommitFailure | null;
  exporting: boolean;
  pickImport: () => Promise<void>;
  setMode: (mode: ImportMode) => Promise<void>;
  applyImport: () => Promise<boolean>;
  closeImport: () => void;
  dismissRefusal: () => void;
  exportRules: () => Promise<ExportOutcome>;
}

let previewSequence = 0;

export const useReplacementFiles = create<FileStore>()(() => ({
  phase: "idle",
  file: null,
  mode: "merge",
  preview: null,
  refusal: null,
  failure: null,
  exporting: false,
  pickImport,
  setMode,
  applyImport,
  closeImport,
  dismissRefusal: () => patch({ refusal: null }),
  exportRules,
}));

const files = (): FileStore => useReplacementFiles.getState();
const patch = (next: Partial<FileStore>): void =>
  useReplacementFiles.setState(next);

/**
 * Whether Apply import may commit: a preview of the current mode without errors that changes
 * the rules.
 * @param store file state
 * @returns true when Apply import is enabled
 */
export const canApplyImport = (store: FileStore): boolean =>
  store.phase === "ready" &&
  store.preview !== null &&
  canApplyPreview(store.preview.preview) &&
  // After a revision conflict, Reload previews the file again before any Apply.
  store.failure?.conflict !== true;

/**
 * Open the native import dialog, then preview the chosen file in the current mode. A refused
 * file shows its alert and opens nothing; a cancelled dialog changes nothing.
 * Side effects: one `import_word_replacements_file` call, then one preview.
 */
async function pickImport(): Promise<void> {
  if (files().phase !== "idle") return;
  patch({ phase: "picking", refusal: null, failure: null });
  try {
    const result = await commands.importWordReplacementsFile();
    if (result.status === "error") {
      const { file_name: fileName, error } = result.error;
      patch({ phase: "idle", refusal: { fileName, kind: fileRefusal(error) } });
      return;
    }
    if (result.data === null) return patch({ phase: "idle" });
    patch({ file: result.data, mode: "merge", preview: null });
    await previewImport();
  } catch (error) {
    console.error("Failed to open the import dialog:", error);
    patch({ phase: "idle", refusal: { fileName: "", kind: "unreadable" } });
  }
}

/**
 * Preview the chosen file in `mode`.
 * @param mode merge or replace
 */
async function setMode(mode: ImportMode): Promise<void> {
  if (files().file === null || files().phase === "applying") return;
  patch({ mode });
  await previewImport();
}

/** Preview the chosen file in the current mode; a newer request wins. */
async function previewImport(): Promise<void> {
  const { file, mode } = files();
  if (file === null) return;
  previewSequence += 1;
  const sequence = previewSequence;
  patch({ phase: "previewing", failure: null });
  try {
    const result = await commands.previewWordReplacements(file.text, mode);
    if (sequence !== previewSequence) return;
    if (result.status === "ok") patch({ phase: "ready", preview: result.data });
    else
      patch({
        phase: "ready",
        preview: null,
        failure: commitFailure(result.error),
      });
  } catch (error) {
    console.error("Failed to preview the imported rules:", error);
    if (sequence === previewSequence) {
      patch({ phase: "ready", preview: null, failure: UNEXPECTED_FAILURE });
    }
  }
}

/**
 * Apply the previewed import in one commit at the preview's revision (AC-005). A conflict keeps
 * the dialog open: the file is previewed again against the new rules before any Apply.
 * Side effects: one `apply_word_replacements` call; the list editor takes the saved rules.
 * @returns true when the rules were imported
 */
async function applyImport(): Promise<boolean> {
  const { file, mode, preview } = files();
  if (!canApplyImport(files()) || file === null || preview === null) {
    return false;
  }
  patch({ phase: "applying", failure: null });
  try {
    const result = await commands.applyWordReplacements(
      file.text,
      mode,
      preview.revision,
    );
    if (result.status === "error")
      return failImport(commitFailure(result.error));
    useReplacementEditor.getState().acceptCommitted(result.data);
    closeImport();
    return true;
  } catch (error) {
    console.error("Failed to import replacement rules:", error);
    return failImport(UNEXPECTED_FAILURE);
  }
}

function failImport(failure: CommitFailure): false {
  patch({ phase: "ready", failure });
  if (failure.uncertain) void useReplacementEditor.getState().loadRules();
  return false;
}

/** Close the import dialog; nothing is saved. */
function closeImport(): void {
  previewSequence += 1;
  patch({ phase: "idle", file: null, preview: null, failure: null });
}

/**
 * Re-preview the chosen file after a conflict, against the rules saved now.
 * Side effects: rereads the saved rules, then previews.
 */
export const reloadImportPreview = async (): Promise<void> => {
  await useReplacementEditor.getState().loadRules();
  await previewImport();
};

/**
 * Export the saved rules through the native save dialog.
 * Side effects: one `export_word_replacements_file` call, which writes the chosen file.
 * @returns what happened, for the confirmation or the error message
 */
async function exportRules(): Promise<ExportOutcome> {
  if (files().exporting) return { status: "cancelled" };
  patch({ exporting: true });
  try {
    const result = await commands.exportWordReplacementsFile();
    if (result.status === "error") {
      const { file_name: fileName, error } = result.error;
      return { status: "failed", fileName, key: exportFailureKey(error) };
    }
    if (result.data === null) return { status: "cancelled" };
    const { file_name: fileName, count, durable } = result.data;
    return { status: "exported", fileName, count, durable };
  } catch (error) {
    console.error("Failed to export replacement rules:", error);
    return { status: "failed", fileName: "", key: UNEXPECTED_FAILURE.key };
  } finally {
    patch({ exporting: false });
  }
}
