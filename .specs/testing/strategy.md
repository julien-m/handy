# Testing Strategy: Handy

## Observed Tools

- @playwright/test

## Observed Commands

- `bun src/lib/utils/keyboard.test.ts`
- `playwright test`
- `playwright test --ui`

## Evidence Boundary

Commands above are observed and deduplicated; they were not executed during initialization.
Run relevant unit, integration and native/frontend tests before claiming application success.
