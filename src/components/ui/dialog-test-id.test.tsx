/**
 * The dialog's test id covers the whole modal, title, body and footer (feature 002, 002-02 of
 * the Penflow alignment: `replacement-settings-import-dialog` holds Cancel and Apply).
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
Object.assign(globalThis, { document: { body: {} } });
const { Dialog } = await import("./Dialog");

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
