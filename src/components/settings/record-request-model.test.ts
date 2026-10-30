// Tests of the one-shot recorder start requests (004 "Change" and "Add"):
// a request starts its recorder once and is then forgotten, so a row that
// leaves the list and comes back, or a recorder mounted later, never starts
// recording on its own. Run with `bun run test:shortcut-states`.
import { describe, expect, test } from "bun:test";
import {
  NO_RECORD_REQUEST,
  dropUnlistedRequest,
  isNewRecordRequest,
  markRequestHandled,
  requestFor,
  requestRecording,
} from "./record-request-model";

describe("Change on an inherited conflict", () => {
  test("only the row of the requested shortcut receives the request", () => {
    const requested = requestRecording(NO_RECORD_REQUEST, "alt+space");
    expect(requestFor(requested, "alt+space")).toBe(1);
    expect(requestFor(requested, "f5")).toBe(0);
  });

  test("a handled request is cleared: a remounted row does not record", () => {
    const requested = requestRecording(NO_RECORD_REQUEST, "alt+space");
    const handled = markRequestHandled(requested, 1);
    const remounted = requestFor(handled, "alt+space");
    expect(remounted).toBe(0);
    // A freshly mounted chip has handled nothing yet (0).
    expect(isNewRecordRequest(remounted, 0)).toBe(false);
  });

  test("a request whose row left the list is dropped", () => {
    const requested = requestRecording(NO_RECORD_REQUEST, "alt+space");
    const dropped = dropUnlistedRequest(requested, ["f5", "f6"]);
    expect(dropped.request).toBeNull();
    // The shortcut comes back (CLI, another window): nothing starts.
    expect(requestFor(dropped, "alt+space")).toBe(0);
    expect(dropUnlistedRequest(requested, ["alt+space"])).toBe(requested);
  });

  test("an older handled report never clears a newer request", () => {
    const first = requestRecording(NO_RECORD_REQUEST, "alt+space");
    const second = requestRecording(first, "f5");
    expect(markRequestHandled(second, 1)).toBe(second);
    expect(requestFor(second, "f5")).toBe(2);
  });

  test("request numbers are never reused after a clear", () => {
    const handled = markRequestHandled(
      requestRecording(NO_RECORD_REQUEST, "alt+space"),
      1,
    );
    const again = requestRecording(handled, "alt+space");
    expect(requestFor(again, "alt+space")).toBe(2);
    // The chip that handled request 1 still starts for request 2.
    expect(isNewRecordRequest(2, 1)).toBe(true);
  });
});

describe("recorder side", () => {
  test("a request is acted on once, never on re-render", () => {
    expect(isNewRecordRequest(0, 0)).toBe(false);
    expect(isNewRecordRequest(3, 0)).toBe(true);
    expect(isNewRecordRequest(3, 3)).toBe(false);
  });
});
