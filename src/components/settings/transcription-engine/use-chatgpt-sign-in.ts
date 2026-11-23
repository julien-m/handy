/** Hook reading the ChatGPT sign-in status used by the ChatGPT engine. */
import { useEffect, useState } from "react";
import { commands, type ChatGptSignInStatus } from "@/bindings";

/**
 * Read the ChatGPT sign-in status while the calling card is mounted, again
 * whenever the window regains focus (e.g. after "codex login" in a terminal).
 * A failed read reports `signed_out` so the sign-in guidance shows instead of nothing.
 * @returns the status, or null before the first read
 */
export const useChatGptSignIn = (): ChatGptSignInStatus | null => {
  const [status, setStatus] = useState<ChatGptSignInStatus | null>(null);
  useEffect(() => {
    let isCurrent = true;
    const readStatus = () => {
      commands
        .getChatgptSignInStatus()
        .then((next) => {
          if (isCurrent) setStatus(next);
        })
        .catch(() => {
          if (isCurrent) setStatus("signed_out");
        });
    };
    readStatus();
    window.addEventListener("focus", readStatus);
    return () => {
      isCurrent = false;
      window.removeEventListener("focus", readStatus);
    };
  }, []);
  return status;
};
