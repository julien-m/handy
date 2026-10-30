// LiveSpec traceability: @spec(AC-001)
/**
 * Markup of the "Remember this field / terminal" entry of the badge menu (013): nothing without
 * an offer, the field or terminal label (and its glyph) per offer, a pointer press that keeps
 * the user's focus, a click that remembers, and the feedback in a status line. Run with
 * `bun run test:indicator`.
 */
import { beforeAll, describe, expect, test } from "bun:test";
import i18n from "i18next";
import { createElement, isValidElement, type ReactElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import type { DestinationMenu, RememberOffer } from "@/bindings";
import { keepFocus } from "./destination-badge";
import {
  DefaultInputEntry,
  DefaultInputView,
  type DefaultInputViewProps,
} from "./default-input-entry";
import { answerFeedback, offerLabel } from "./default-input-model";
import {
  chatgpt,
  destination,
  initIndicatorI18n,
  render,
  session,
} from "./indicator-test-fixtures";

beforeAll(initIndicatorI18n);

const MENU = "recording-indicator-destination-menu-menu";

/** View props for `offer`, translated the way the entry translates them. */
const viewProps = (
  offer: RememberOffer,
  patch: Partial<DefaultInputViewProps> = {},
): DefaultInputViewProps => {
  const label = offerLabel(offer);
  return {
    menuId: MENU,
    kind: label.kind,
    label: i18n.t(label.key, label.values),
    feedback: null,
    onRemember: () => undefined,
    ...patch,
  };
};
const html = (props: DefaultInputViewProps): string =>
  renderToStaticMarkup(createElement(DefaultInputView, props));

const codex: RememberOffer = {
  kind: "field",
  app_name: "Codex",
  remembered: false,
};
const cmux: RememberOffer = {
  kind: "terminal",
  app_name: "cmux",
  remembered: false,
};

/** The menu item element the view returns, found in its fragment. */
const menuItem = (
  props: DefaultInputViewProps,
): ReactElement<{
  onClick: () => void;
  onMouseDown: unknown;
}> => {
  const fragment = DefaultInputView(props) as ReactElement<{
    children: unknown[];
  }>;
  const button = fragment.props.children.find(
    (child) => isValidElement(child) && child.type === "button",
  );
  if (!isValidElement(button)) throw new Error("no menu item");
  return button as ReactElement<{ onClick: () => void; onMouseDown: unknown }>;
};

describe("remember entry (AC-001)", () => {
  test("without an offer nothing is rendered, in the menu neither", () => {
    expect(
      renderToStaticMarkup(
        createElement(DefaultInputEntry, {
          dictationId: 7,
          selectionVersion: 1,
        }),
      ),
    ).toBe("");
    const menu: DestinationMenu = {
      dictation_id: 7,
      selection_version: 1,
      frontmost_at_end_current: false,
      entries: [],
    };
    const markup = render({
      destination: destination(session(chatgpt), { status: "open", menu }),
    });
    expect(markup).toContain(`data-testid="${MENU}-footer"`);
    expect(markup).not.toContain("-remember-input");
  });
  test("a field names the app; a terminal has its own label and glyph", () => {
    const field = html(viewProps(codex));
    expect(field).toContain(`data-testid="${MENU}-remember-input"`);
    expect(field).toContain('role="menuitem"');
    expect(field).toContain(
      `>${i18n.t(offerLabel(codex).key, { app: "Codex" })}<`,
    );
    expect(field).toContain("Codex");
    expect(field).toContain("lucide-text-cursor-input");
    const terminal = html(viewProps(cmux));
    expect(terminal).toContain(`>${i18n.t(offerLabel(cmux).key)}<`);
    expect(terminal).toContain("lucide-square-terminal");
    expect(terminal).not.toContain("lucide-text-cursor-input");
  });
  test("an application already remembered says 'again'", () => {
    const again = { ...codex, remembered: true };
    const markup = html(viewProps(again));
    expect(markup).toContain(
      `>${i18n.t(offerLabel(again).key, { app: "Codex" })}<`,
    );
    expect(markup).not.toContain(
      `>${i18n.t(offerLabel(codex).key, { app: "Codex" })}<`,
    );
  });
  test("the glyph is hidden from screen readers; the press keeps the user's focus", () => {
    expect(html(viewProps(codex))).toMatch(/<svg[^>]*aria-hidden="true"/);
    expect(menuItem(viewProps(codex)).props.onMouseDown).toBe(keepFocus);
  });
  test("a click remembers, and the feedback is a status line", () => {
    let calls = 0;
    const props = viewProps(codex, { onRemember: () => (calls += 1) });
    menuItem(props).props.onClick();
    expect(calls).toBe(1);
    expect(html(props)).not.toContain("-remember-feedback");
    const saved = answerFeedback({
      status: "saved",
      value: { ...codex, remembered: true },
    });
    const text = i18n.t(saved.key, saved.values);
    const markup = html({ ...props, feedback: text });
    expect(markup).toMatch(
      new RegExp(`role="status" data-testid="${MENU}-remember-feedback">`),
    );
    expect(markup).toContain(`>${text}<`);
  });
});
