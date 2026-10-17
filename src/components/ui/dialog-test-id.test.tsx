/**
 * The dialog's test id covers the whole modal, title, body and footer (feature 002, 002-02 of
 * the Penflow alignment: `replacement-settings-import-dialog` holds Cancel and Apply).
 * The 002 dialogs and the commit failure note carry the contract test ids of
 * `replacement-settings` (`delete-dialog`, `commit-failure`).
 * Run with `bun test src/components/ui/dialog-test-id.test.tsx`.
 */
import { describe, expect, mock, test } from "bun:test";
import * as ReactDom from "react-dom";
import { renderToStaticMarkup } from "react-dom/server";

// Static markup has no DOM: the portal renders in place, and `document.body` is only its target.
mock.module("react-dom", () => ({
  ...ReactDom,
  createPortal: (node: React.ReactNode) => node,
}));
mock.module("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));
Object.assign(globalThis, { document: { body: {} } });
const { Dialog } = await import("./Dialog");
const { DeleteRuleDialog } = await import(
  "../settings/word-replacements/delete-rule-dialog"
);
const { CommitFailureNote } = await import(
  "../settings/word-replacements/commit-failure-note"
);

const APPLY = "Apply import";
const BODY = "Body";
const markup = renderToStaticMarkup(
  <Dialog
    open
    title="Import rules"
    closeLabel="Close"
    onOpenChange={() => undefined}
    footer={<button type="button">{APPLY}</button>}
    testId="replacement-settings-import-dialog"
  >
    <p>{BODY}</p>
  </Dialog>,
);

describe("dialog test id", () => {
  test("is on the modal element itself", () => {
    expect(
      markup.includes('role="dialog" aria-modal="true" aria-labelledby='),
    ).toBe(true);
    const start = markup.indexOf(
      'data-testid="replacement-settings-import-dialog"',
    );
    expect(start > markup.indexOf('role="dialog"')).toBe(true);
    expect(markup.indexOf("Import rules") > start).toBe(true);
    expect(markup.indexOf("Apply import") > start).toBe(true);
  });
});

describe("replacement dialogs and notes (002 contract ids)", () => {
  test("the deletion dialog, its body, Cancel and Delete", () => {
    const html = renderToStaticMarkup(
      <DeleteRuleDialog
        rule={{
          key: "a",
          id: null,
          recognized: "cloud code",
          replacement: "Claude Code",
        }}
        onCancel={() => undefined}
        onConfirm={() => undefined}
      />,
    );
    for (const id of ["", "-body", "-cancel", "-confirm"]) {
      expect(
        html.includes(`data-testid="replacement-settings-delete-dialog${id}"`),
      ).toBe(true);
    }
  });

  test("a commit conflict: the note, its message and Reload", () => {
    const html = renderToStaticMarkup(
      <CommitFailureNote
        failure={{ key: "k", conflict: true, uncertain: false, items: [] }}
        onReload={() => undefined}
        testId="replacement-settings-bulk-commit-failure"
      />,
    );
    for (const id of ["", "-message", "-reload"]) {
      expect(
        html.includes(
          `data-testid="replacement-settings-bulk-commit-failure${id}"`,
        ),
      ).toBe(true);
    }
  });
});
