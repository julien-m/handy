/**
 * Whether this instance can return dictation text to its destination application (006,
 * AC-007): undefined until the backend answered, null when it has no destination manager.
 */
import { useEffect, useState } from "react";
import { commands, type DestinationSupport } from "@/bindings";

/**
 * Ask the backend once per component mount.
 * @returns The support answer, undefined while it is pending.
 */
export function useDestinationSupport(): DestinationSupport | null | undefined {
  const [support, setSupport] = useState<DestinationSupport | null | undefined>(
    undefined,
  );
  useEffect(() => {
    let mounted = true;
    commands
      .destinationSupport()
      .then((answer) => {
        if (mounted) setSupport(answer);
      })
      .catch((error: unknown) => {
        // Unanswered: show the setting as unavailable rather than guess support.
        console.warn("Destination support unavailable:", error);
        if (mounted) setSupport(null);
      });
    return () => {
      mounted = false;
    };
  }, []);
  return support;
}
