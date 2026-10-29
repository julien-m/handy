// State of the "Remember this field / terminal" entry of the badge menu (013): asks the backend
// what the open menu may remember, then remembers it on demand. Answers for an older dictation
// or selection version are ignored; remembering never changes the destination or the pin.
import { useCallback, useEffect, useRef, useState } from "react";
import { commands, type RememberOffer } from "@/bindings";
import {
  menuRequestKey,
  settleRemember,
  type TranslatedText,
} from "./default-input-model";

/** What the entry needs: the offer (null: no entry), the feedback and the remember action. */
export interface DefaultInputControls {
  offer: RememberOffer | null;
  feedback: TranslatedText | null;
  remember: () => void;
}

/**
 * The offer and the remember action of the open menu.
 * @param dictationId dictation of the open menu
 * @param selectionVersion selection version of the open menu
 * @returns the offer, the feedback of the last remember and the remember action
 */
export const useDefaultInput = (
  dictationId: number,
  selectionVersion: number,
): DefaultInputControls => {
  const [offer, setOffer] = useState<RememberOffer | null>(null);
  const [feedback, setFeedback] = useState<TranslatedText | null>(null);
  const current = useRef(menuRequestKey(dictationId, selectionVersion));

  useEffect(() => {
    const key = menuRequestKey(dictationId, selectionVersion);
    current.current = key;
    setOffer(null);
    setFeedback(null);
    commands
      .defaultInputOffer(dictationId, selectionVersion)
      .then((answer) => {
        if (current.current === key) setOffer(answer);
      })
      .catch((error: unknown) => {
        // No offer is the safe answer: the entry stays hidden.
        console.warn("Default input offer unavailable:", error);
      });
  }, [dictationId, selectionVersion]);

  const remember = useCallback(() => {
    const key = menuRequestKey(dictationId, selectionVersion);
    const pending = commands.rememberDefaultInput(
      dictationId,
      selectionVersion,
    );
    void settleRemember(pending).then((outcome) => {
      if (current.current !== key) return;
      setFeedback(outcome.feedback);
      if (outcome.offer) setOffer(outcome.offer);
    });
  }, [dictationId, selectionVersion]);

  return { offer, feedback, remember };
};
