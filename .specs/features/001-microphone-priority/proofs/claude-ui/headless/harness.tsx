// Browser harness: the real PriorityList with a stateful order, for keyboard focus and pointer
// reorder checks in headless Chromium (not a native window proof).
import React, { useState } from "react";
import { createRoot } from "react-dom/client";
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import en from "/Users/julienm/projects/handy-wt/ui-001-microphones/src/i18n/locales/en/translation.json";
import { PriorityList } from "/Users/julienm/projects/handy-wt/ui-001-microphones/src/components/settings/microphone-priority/priority-list";
import { moveChoice, removeChoice } from "/Users/julienm/projects/handy-wt/ui-001-microphones/src/components/settings/microphone-priority/draft-state";

void i18n.use(initReactI18next).init({ resources: { en: { translation: en } }, lng: "en", interpolation: { escapeValue: false } });

const Harness = () => {
  const [order, setOrder] = useState(["USB headset", "MacBook Pro Microphone", "Studio Display Microphone"]);
  (window as any).currentOrder = order;
  return (
    <div className="p-4" style={{ width: 480 }}>
      <PriorityList
        order={order}
        availability={order.map(() => "available" as const)}
        busy={false}
        onMove={(from, to) => setOrder((o) => moveChoice(o, from, to))}
        onRemove={(index) => setOrder((o) => removeChoice(o, index))}
      />
    </div>
  );
};
createRoot(document.getElementById("root")!).render(<Harness />);
