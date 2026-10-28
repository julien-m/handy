// Browser harness of the replacement rules editor (feature 002): the real section against the
// fake Tauri IPC of `word-replacements-fake.ts`, served by the Vite dev server for
// `tests/word-replacements-focus.spec.ts`. Above it, a vocabulary-like group stands for the
// section shown before Replacements on the Advanced page.
import { backend } from "./word-replacements-fake";
import React from "react";
import { createRoot } from "react-dom/client";
import "@/i18n";
import "@/App.css";
import { SettingsGroup } from "@/components/ui/SettingsGroup";
import { WordReplacements } from "@/components/settings/word-replacements/word-replacements";

const Page: React.FC = () => (
  <div className="p-4 max-w-3xl flex flex-col gap-4">
    <SettingsGroup title="Vocabulary">
      <p className="p-4 text-sm">Previous section</p>
    </SettingsGroup>
    <WordReplacements />
  </div>
);

const element = document.getElementById("root");
if (element !== null) createRoot(element).render(<Page />);

(window as unknown as Record<string, unknown>).harness = { backend };
