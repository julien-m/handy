import { describe, expect, test } from "bun:test";
import { alphabetical, displayOrder } from "./alphabetical";

const text = (value: string) => value;

describe("alphabetical display (alphabetical_display)", () => {
  test("sorts ignoring case and accents, numbers by value, saved index kept", () => {
    const saved = ["zoom", "Été", "abc", "item 10", "item 2", "Apple"];
    expect(alphabetical(saved, text, "fr").map((p) => p.item)).toEqual([
      "abc",
      "Apple",
      "Été",
      "item 2",
      "item 10",
      "zoom",
    ]);
    expect(alphabetical(saved, text, "fr")[0].index).toBe(2);
  });

  test("equal texts keep their saved order", () => {
    const saved = ["b", "A", "a"];
    expect(alphabetical(saved, text, "en").map((p) => p.index)).toEqual([
      1, 2, 0,
    ]);
  });

  test("a frozen order does not move while a row is edited", () => {
    const frozen = ["a", "b", "c"];
    const edited = ["zzz", "b", "c", "new"];
    const keys = ["a", "b", "c", "d"];
    const order = displayOrder(
      edited,
      (item) => keys[edited.indexOf(item)],
      text,
      "en",
      frozen,
    );
    expect(order.map((p) => p.item)).toEqual(["zzz", "b", "c", "new"]);
    expect(
      displayOrder(edited, (item) => item, text, "en", null).map((p) => p.item),
    ).toEqual(["b", "c", "new", "zzz"]);
  });
});
