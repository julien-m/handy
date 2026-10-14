// State of the prompt test panel (005 AC-007, Behavioral AC `async_action`):
// the editable sample prefilled with the last transcript, one test at a
// time on the editor's unsaved draft, and the test cancelled when the
// editor closes. A test never inserts text or writes history (backend).
import { type MutableRefObject, useEffect, useRef, useState } from "react";
import { commands } from "@/bindings";
import type { PromptTestRequest } from "@/bindings";
import {
  IDLE_TEST_VIEW,
  type TestPanelState,
  type TestView,
  latestTranscriptOf,
  refusedTestViewOf,
  testViewOf,
} from "./prompt-test-model";

/** The panel's sample, state and actions. */
export interface PromptTestModel {
  sample: string;
  setSample: (sample: string) => void;
  state: TestPanelState;
  view: TestView;
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

/**
 * Test panel state for the draft described by `request`.
 * @param requestOf Builds the request from the current draft and sample.
 */
export const usePromptTest = (
  requestOf: (sample: string) => PromptTestRequest,
): PromptTestModel => {
  const [sample, setSample] = usePrefilledSample();
  const [view, setView] = useState<TestView>(IDLE_TEST_VIEW);
  const [isRunning, setIsRunning] = useState(false);
  const running = useRef(false);
  const mounted = useMountedTest(running);
  const run = async () => {
    // One test at a time: the control is busy until this one answers.
    if (running.current) return;
    running.current = true;
    setIsRunning(true);
    try {
      const result = await commands.testPostProcessPrompt(requestOf(sample));
      if (!mounted.current) return;
      setView(
        result.status === "ok"
          ? testViewOf(result.data)
          : refusedTestViewOf(result.error),
      );
    } catch (failure) {
      console.error("Failed to run the prompt test:", failure);
      if (mounted.current) setView(GENERIC_FAILURE);
    } finally {
      running.current = false;
      if (mounted.current) setIsRunning(false);
    }
  };
  const state = isRunning ? "loading" : view.state;
  return { sample, setSample, state, view, run };
};
