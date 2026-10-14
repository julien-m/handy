// Backend side of the two shortcut recorders (004): a start answers after an
// IPC round trip, and the chip may be gone by then (Discard of a blocked
// draft, a row removed by the CLI, a keyboard implementation switch). A start
// that succeeds after the chip unmounted is undone at once, and a chip that
// unmounts while it holds the global shortcuts suspended resumes them, so no
// shortcut ever stays suspended by a recorder that no longer exists.
import { useEffect, useRef, type MutableRefObject } from "react";
import { commands } from "@/bindings";
import type { RecorderStartFailure } from "./shortcut-draft-model";

/** How a recorder start ended. */
export type RecorderStart =
  | { status: "recording" }
  /** It started after the chip unmounted, and was stopped again. */
  | { status: "abandoned" }
  | { status: "refused"; failure: RecorderStartFailure };

/**
 * Asks the HandyKeys backend to record. It refuses while macOS Secure Input
 * is active (the listener would receive no key events and capture just the
 * modifier) and then also shows the Secure Input warning banner.
 * @param shortcutId Action whose shortcut is recorded.
 * @returns Null once recording, else why it could not start.
 */
const startBackendRecording = async (
  shortcutId: string,
): Promise<RecorderStartFailure | null> => {
  try {
    const result = await commands.startHandyKeysRecording(shortcutId);
    if (result.status === "ok") return null;
    const message = String(result.error);
    return message.includes("secure-input-active")
      ? { kind: "secure_input" }
      : { kind: "failed", message };
  } catch (error) {
    console.error("Failed to start recording:", error);
    return { kind: "failed", message: String(error) };
  }
};

/**
 * Starts HandyKeys recording; stops it again if the chip unmounted meanwhile
 * (the backend suspends every shortcut while it records).
 * @param shortcutId Action whose shortcut is recorded.
 * @param isMounted Whether the chip is still mounted when the answer comes.
 */
export const startHandyKeys = async (
  shortcutId: string,
  isMounted: () => boolean,
): Promise<RecorderStart> => {
  const failure = await startBackendRecording(shortcutId);
  if (failure) return { status: "refused", failure };
  if (isMounted()) return { status: "recording" };
  await commands.stopHandyKeysRecording().catch(console.error);
  return { status: "abandoned" };
};

/**
 * Suspends every global shortcut for a Tauri capture (none fires, or
 * swallows the keys, while recording); resumes them again if the chip
 * unmounted meanwhile.
 * @param isMounted Whether the chip is still mounted when the answer comes.
 */
export const suspendForCapture = async (
  isMounted: () => boolean,
): Promise<RecorderStart> => {
  await commands.suspendAllBindings().catch(console.error);
  if (isMounted()) return { status: "recording" };
  await commands.resumeAllBindings().catch(console.error);
  return { status: "abandoned" };
};

/**
 * Cleanup of a Tauri recorder: resumes the global shortcuts it suspended if
 * it unmounts while recording.
 * @param isSuspending The chip holds the shortcuts suspended.
 */
export const resumeOnUnmount =
  (isSuspending: MutableRefObject<boolean>) => (): void => {
    if (!isSuspending.current) return;
    isSuspending.current = false;
    void commands.resumeAllBindings().catch(console.error);
  };

/** False once the component unmounted (true again if React remounts it). */
export const useMounted = (): MutableRefObject<boolean> => {
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);
  return mounted;
};
