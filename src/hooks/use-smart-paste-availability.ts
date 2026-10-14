/**
 * Whether the backend offers smart paste (feature 007): macOS only, and only once dictation
 * delivers through it. False until the backend answered, so nothing offers smart paste early.
 */
import { useEffect, useState } from "react";
import { commands } from "@/bindings";

/**
 * Ask the backend once per component mount.
 * @returns true when the paste method selector may offer smart paste
 */
export function useSmartPasteAvailability(): boolean {
  const [available, setAvailable] = useState(false);
  useEffect(() => {
    let mounted = true;
    commands
      .isSmartPasteAvailable()
      .then((answer) => {
        if (mounted) setAvailable(answer);
      })
      .catch((error: unknown) => {
        // Unanswered: keep the existing methods only, never offer smart paste by guess.
        console.warn("Smart paste availability unavailable:", error);
      });
    return () => {
      mounted = false;
    };
  }, []);
  return available;
}
