/**
 * Commits of the prompt editor against stubbed commands: the name and text
 * and the provider override are sent in order, only when they changed, a
 * refusal says what was already saved, and a `committed` or `unknown`
 * outcome is resolved by rereading the library. Run with
 * `bun run test:prompt-settings`.
 */
import { beforeEach, describe, expect, mock, test } from "bun:test";
import type { PromptProviderOverride, SettingsCommandError } from "@/bindings";

type Reply =
  | { status: "ok"; data: unknown }
  | { status: "error"; error: SettingsCommandError };
const calls: Array<[string, unknown[]]> = [];
let replies: Record<string, Reply[]> = {};
const ok = (data: unknown = null): Reply => ({ status: "ok", data });
const answer = (name: string, args: unknown[]) => {
  calls.push([name, args]);
  return Promise.resolve(replies[name]?.shift() ?? ok());
};
const commands = {
  addPostProcessPrompt: (name: string, prompt: string) =>
    answer("add", [name, prompt]),
  updatePostProcessPrompt: (id: string, name: string, prompt: string) =>
    answer("update", [id, name, prompt]),
  setPostProcessPromptProviderOverride: (
    id: string,
    override: PromptProviderOverride | null,
  ) => answer("override", [id, override]),
  getPromptLibrary: () => answer("library", []),
};
// The real bindings need Tauri IPC; the commits only need these commands.
mock.module("@/bindings", () => ({ commands }));
// Dynamic import: the module must load after the mock above is registered.
const { createPrompt, updatePrompt } = await import("./prompt-commands");
const { EMPTY_DRAFT } = await import("./prompt-draft-model");

const refusal = (reason: SettingsCommandError["reason"]): Reply => ({
  status: "error",
  error: {
    code: "validation_failed",
    message: "Invalid value.",
    field: "post_process_prompts",
    reason,
    outcome: null,
    current_revision: null,
  },
});
const saved = {
  name: "Shorten",
  text: "Shorten ${output}",
  provider: { providerId: "", model: "" },
};

// @spec AC-001: Create and edit commit the validated prompt — .specs/features/005-custom-ai-prompts/spec.md#ac-001
describe("prompt commits", () => {
  beforeEach(() => {
    calls.length = 0;
    replies = {};
  });

  test("create sends the trimmed name and the verbatim text, then the override", async () => {
    replies.add = [ok({ id: "new-id", name: "Formal", prompt: " x " })];
    const draft = {
      name: " Formal ",
      text: " x ",
      provider: { providerId: "custom", model: "llama3.2" },
    };
    expect(await createPrompt(draft, EMPTY_DRAFT)).toEqual({
      status: "saved",
      id: "new-id",
    });
    expect(calls).toEqual([
      ["add", ["Formal", " x "]],
      ["override", ["new-id", { provider_id: "custom", model: "llama3.2" }]],
    ]);
  });

  test("a refused creation saves nothing", async () => {
    replies.add = [refusal("prompt_name_duplicate")];
    const outcome = await createPrompt({ ...saved }, EMPTY_DRAFT);
    expect(outcome.status).toBe("refused");
    expect(outcome.id).toBeNull();
    expect(calls.map(([name]) => name)).toEqual(["add"]);
  });

  test("a refused override after creation reports the created prompt", async () => {
    replies.add = [ok({ id: "new-id" })];
    replies.override = [refusal("prompt_override_incomplete")];
    const draft = { ...saved, provider: { providerId: "openai", model: "" } };
    const outcome = await createPrompt(draft, EMPTY_DRAFT);
    expect(outcome).toMatchObject({
      status: "refused",
      id: "new-id",
      contentSaved: true,
    });
  });

  test("an update sends only what changed", async () => {
    await updatePrompt("p1", { ...saved, text: "New ${output}" }, saved);
    expect(calls).toEqual([["update", ["p1", "Shorten", "New ${output}"]]]);
    calls.length = 0;
    const override = {
      ...saved,
      provider: { providerId: "custom", model: "m" },
    };
    await updatePrompt("p1", override, saved);
    expect(calls).toEqual([
      ["override", ["p1", { provider_id: "custom", model: "m" }]],
    ]);
  });

  test("returning to the default provider clears the override", async () => {
    const before = { ...saved, provider: { providerId: "custom", model: "m" } };
    await updatePrompt("p1", saved, before);
    expect(calls).toEqual([["override", ["p1", null]]]);
  });

  test("a refused content update stops before the override", async () => {
    replies.update = [refusal("prompt_text_empty")];
    const draft = {
      name: "Shorten",
      text: " ",
      provider: { providerId: "custom", model: "m" },
    };
    const outcome = await updatePrompt("p1", draft, saved);
    expect(outcome).toMatchObject({ status: "refused", contentSaved: false });
    expect(calls.map(([name]) => name)).toEqual(["update"]);
  });
});

const uncertain = (outcome: "committed" | "unknown"): Reply => ({
  status: "error",
  error: {
    code: "persistence_failed",
    message: "Not confirmed.",
    field: "post_process_prompts",
    reason: null,
    outcome,
    current_revision: null,
  },
});
const libraryWith = (
  prompts: Array<{ id: string; name: string; prompt: string }>,
): Reply =>
  ok({
    default_prompt_id: null,
    post_process_enabled: true,
    prompts: prompts.map((prompt) => ({ ...prompt, provider_override: null })),
  });

// @spec AC-001: A stored-but-unconfirmed creation is never a refusal — .specs/features/005-custom-ai-prompts/spec.md#ac-001
describe("committed or unknown outcomes", () => {
  beforeEach(() => {
    calls.length = 0;
    replies = {};
  });

  test("a committed creation found in the library is saved and edited", async () => {
    replies.add = [uncertain("committed")];
    replies.library = [
      libraryWith([
        { id: "made", name: "Shorten", prompt: "Shorten ${output}" },
      ]),
    ];
    const outcome = await createPrompt(
      { ...saved, name: " Shorten " },
      EMPTY_DRAFT,
    );
    expect(outcome).toEqual({ status: "saved", id: "made" });
    expect(calls.map(([name]) => name)).toEqual(["add", "library"]);
  });

  test("an unknown creation missing from the library stays refused", async () => {
    replies.add = [uncertain("unknown")];
    replies.library = [libraryWith([])];
    const outcome = await createPrompt({ ...saved }, EMPTY_DRAFT);
    expect(outcome).toMatchObject({ status: "refused", id: null });
  });

  test("a validation refusal is not reread", async () => {
    replies.add = [refusal("prompt_name_duplicate")];
    await createPrompt({ ...saved }, EMPTY_DRAFT);
    expect(calls.map(([name]) => name)).toEqual(["add"]);
  });

  test("an unknown update whose values are stored counts as saved", async () => {
    replies.update = [uncertain("unknown")];
    replies.library = [
      libraryWith([{ id: "p1", name: "Shorten", prompt: "New ${output}" }]),
    ];
    const outcome = await updatePrompt(
      "p1",
      { ...saved, text: "New ${output}" },
      saved,
    );
    expect(outcome).toEqual({ status: "saved", id: "p1" });
  });
});
