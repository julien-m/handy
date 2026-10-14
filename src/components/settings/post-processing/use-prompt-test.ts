// State of the prompt test panel (005 AC-007, Behavioral AC `async_action`):
// the editable sample prefilled with the last transcript, one test at a
// time on the editor's unsaved draft, the result tied to the draft it ran
// on, and the test cancelled when the editor closes. A test never inserts
// text or writes history (backend).
import { type MutableRefObject, useEffect, useRef, useState } from "react";
import { commands } from "@/bindings";
import type { PromptProviderOverride, PromptTestRequest } from "@/bindings";
import {
  IDLE_TEST_VIEW,
  type TestPanelState,
  type TestView,
  isResultOutdated,
  latestTranscriptOf,
  refusedTestViewOf,
  testViewOf,
} from "./prompt-test-model";

/** The draft a test runs on: its text and provider override. */
export interface TestDraft {
  text: string;
  provider_override: PromptProviderOverride | null;
}

/** The panel's sample, state and actions. */
export interface PromptTestModel {
  sample: string;
  setSample: (sample: string) => void;
  state: TestPanelState;
  view: TestView;
  /** The shown result was produced for another text, provider, model or sample. */
  isOutdated: boolean;
  run: () => Promise<void>;
}

const GENERIC_FAILURE: TestView = {
  ...IDLE_TEST_VIEW,
  state: "error",
  message: { key: "errors.settings.generic", params: {} },
  canRetry: true,
};

/**
 * Prefills the sample once with the raw transcript of the latest history
 * entry, unless the user already typed one.
 */
const usePrefilledSample = (): [string, (sample: string) => void] => {
  const [sample, setSample] = useState("");
  const typed = useRef(false);
  useEffect(() => {
    const prefill = async () => {
      const result = await commands.getHistoryEntries(null, 1);
      if (result.status === "ok" && !typed.current) {
        setSample(latestTranscriptOf(result.data.entries));
      }
    };
    prefill().catch((failure) => {
      // An empty sample is a valid start: the user types one.
      console.error("Failed to read the last transcript:", failure);
    });
  }, []);
  const edit = (next: string) => {
    typed.current = true;
    setSample(next);
  };
  return [sample, edit];
};

/**
 * Cancels a running test when the editor closes, and tells whether the
 * panel is still shown (a late result is then ignored).
 */
const useMountedTest = (running: MutableRefObject<boolean>) => {
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
      if (running.current) void commands.cancelPostProcessPromptTest();
    };
  }, [running]);
  return mounted;
};

/** The result shown and the request it answers. */
interface ShownResult {
  view: TestView;
  tested: PromptTestRequest | null;
}

/**
 * Runs one test of `request` and returns what the panel shows for it.
 * @throws Error when the command cannot be invoked at all.
 */
const runTest = async (request: PromptTestRequest): Promise<TestView> => {
  const result = await commands.testPostProcessPrompt(request);
  return result.status === "ok"
    ? testViewOf(result.data)
    : refusedTestViewOf(result.error);
};

/**
 * Test panel state for the editor's current draft.
 * @param draft Text and provider override of the draft.
 */
export const usePromptTest = (draft: TestDraft): PromptTestModel => {
  const [sample, setSample] = usePrefilledSample();
  const [shown, setShown] = useState<ShownResult>({
    view: IDLE_TEST_VIEW,
    tested: null,
  });
  const [isRunning, setIsRunning] = useState(false);
  const running = useRef(false);
  const mounted = useMountedTest(running);
  const request: PromptTestRequest = { ...draft, sample };
  const run = async () => {
    // One test at a time: the control is busy until this one answers.
    if (running.current) return;
    running.current = true;
    setIsRunning(true);
    let view = GENERIC_FAILURE;
    try {
      view = await runTest(request);
    } catch (failure) {
      console.error("Failed to run the prompt test:", failure);
    }
    running.current = false;
    if (!mounted.current) return;
    setShown({ view, tested: request });
    setIsRunning(false);
  };
  const state = isRunning ? "loading" : shown.view.state;
  const isOutdated = isResultOutdated(shown.tested, request);
  return { sample, setSample, state, view: shown.view, isOutdated, run };
};
