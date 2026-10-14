/**
 * Stubbed replacement commands shared by the editor store tests (feature 002). Importing this
 * module registers the `@/bindings` mock, so a test file imports it before loading the stores
 * dynamically. Each scenario sets the replies on `stub`; calls are recorded in order.
 */
import { mock } from "bun:test";
import type {
  ImportMode,
  ReplacementPreview,
  ReplacementRule,
  ReplacementRulesView,
  RuleDraft,
  RuleIssue,
  SettingsCommandError,
} from "@/bindings";

/** Typed reply of a stubbed command. */
export type Reply<T> =
  | { status: "ok"; data: T }
  | { status: "error"; error: SettingsCommandError };

/**
 * A saved rule.
 * @returns the rule
 */
export const rule = (
  id: string,
  recognized: string,
  replacement: string,
): ReplacementRule => ({ id, recognized, replacement });

/**
 * Saved rules at a revision.
 * @returns the view
 */
export const view = (
  revision: number,
  rules: ReplacementRule[],
): ReplacementRulesView => ({ revision, rules });

/**
 * A preview with no rows, completed by `overrides`.
 * @returns the preview
 */
export const emptyPreview = (
  overrides: Partial<ReplacementPreview>,
): ReplacementPreview => ({
  mode: "replace",
  input_sha256: "0",
  lines: [],
  set_issues: [],
  candidate: [],
  is_noop: false,
  ignored_lines: 0,
  error_lines: 0,
  omitted_lines: 0,
  ...overrides,
});

/**
 * Typed revision conflict.
 * @param current revision the settings are at
 * @returns the refusal
 */
export const conflict = (current: number): SettingsCommandError => ({
  code: "revision_conflict",
  message: "Settings changed since the expected revision.",
  field: null,
  reason: null,
  outcome: null,
  current_revision: current,
});

/** Replies and recorded calls of the stubbed commands. */
export const stub = {
  current: view(4, [rule("a", "Cloud Code", "Claude Code")]),
  checkIssues: [] as RuleIssue[],
  /** Replies of the next Saves, in order; then a successful Save. */
  saveReplies: [] as Array<Reply<ReplacementRulesView>>,
  /** Replies of the next Applies, in order; then a successful Apply. */
  applyReplies: [] as Array<Reply<ReplacementRulesView>>,
  /** When set, Save waits for it before answering (a Save in flight). */
  saveGate: null as Promise<void> | null,
  /** When set, Apply waits for it before answering (an Apply in flight). */
  applyGate: null as Promise<void> | null,
  /** When set, the validator waits for it before answering (a check in flight). */
  checkGate: null as Promise<void> | null,
  previewRevision: 4,
  /** Extra fields of every preview (a preview that changed). */
  previewExtra: {} as Partial<ReplacementPreview>,
  pickReply: { status: "ok", data: null } as unknown,
  exportReply: { status: "ok", data: null } as unknown,
  saveCalls: [] as Array<[RuleDraft[], number]>,
  applyCalls: [] as Array<[string, ImportMode, number]>,
  previewCalls: [] as Array<[string, ImportMode]>,
};

/** Restore the default replies and clear the recorded calls. */
export const resetStubs = (): void => {
  stub.current = view(4, [rule("a", "Cloud Code", "Claude Code")]);
  stub.checkIssues = [];
  stub.saveReplies = [];
  stub.applyReplies = [];
  stub.saveGate = null;
  stub.applyGate = null;
  stub.checkGate = null;
  stub.previewRevision = 4;
  stub.previewExtra = {};
  stub.saveCalls.length = 0;
  stub.applyCalls.length = 0;
  stub.previewCalls.length = 0;
};

const saveWordReplacements = async (rules: RuleDraft[], revision: number) => {
  stub.saveCalls.push([rules, revision]);
  if (stub.saveGate !== null) await stub.saveGate;
  return (
    stub.saveReplies.shift() ?? {
      status: "ok",
      data: view(revision + 1, stub.current.rules),
    }
  );
};

const previewWordReplacements = async (input: string, mode: ImportMode) => {
  stub.previewCalls.push([input, mode]);
  const preview = emptyPreview({
    mode,
    is_noop: input.endsWith("same"),
    ...stub.previewExtra,
  });
  return { status: "ok", data: { revision: stub.previewRevision, preview } };
};

const applyWordReplacements = async (
  input: string,
  mode: ImportMode,
  revision: number,
) => {
  stub.applyCalls.push([input, mode, revision]);
  if (stub.applyGate !== null) await stub.applyGate;
  return (
    stub.applyReplies.shift() ?? { status: "ok", data: view(revision + 1, []) }
  );
};

const commands = {
  getWordReplacements: async () => stub.current,
  checkWordReplacements: async () => {
    // Read the issues before waiting, as the backend answers for the draft it was sent.
    const issues = stub.checkIssues;
    if (stub.checkGate !== null) await stub.checkGate;
    return issues;
  },
  saveWordReplacements,
  previewWordReplacements,
  applyWordReplacements,
  importWordReplacementsFile: async () => stub.pickReply,
  exportWordReplacementsFile: async () => stub.exportReply,
};
mock.module("@/bindings", () => ({ commands }));
