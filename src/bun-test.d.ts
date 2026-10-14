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
  /** Run `body` once before the tests of the enclosing group. */
  export function beforeAll(body: () => void | Promise<void>): void;
  /** The matchers the frontend tests use. */
  interface Matchers {
    toBe(expected: unknown): void;
    toEqual(expected: unknown): void;
    toMatchObject(expected: object): void;
    toBeNull(): void;
    toContain(expected: unknown): void;
    toMatch(expected: RegExp | string): void;
    toBeLessThan(expected: number): void;
  }
  /** Assertions on `actual`; `not` negates them. */
  export function expect(actual: unknown): Matchers & { not: Matchers };
  /** Module mocking: replaces `specifier` for every later import. */
  export const mock: {
    module(specifier: string, factory: () => Record<string, unknown>): void;
  };
}
