/**
 * Minimal types for the parts of Bun's built-in test runner used by store tests (`bun test`),
 * so `tsc` checks them without adding the Bun type package to the app's type roots.
 */
declare module "bun:test" {
  /** Register a test case. */
  export function test(name: string, body: () => void | Promise<void>): void;
  /** Group test cases. */
  export function describe(name: string, body: () => void): void;
  /** Run `body` before each test of the enclosing group. */
  export function beforeEach(body: () => void | Promise<void>): void;
  /** Assertions on `actual`. */
  export function expect(actual: unknown): { toBe(expected: unknown): void };
  /** Module mocking: replaces `specifier` for every later import. */
  export const mock: {
    module(specifier: string, factory: () => Record<string, unknown>): void;
  };
}
