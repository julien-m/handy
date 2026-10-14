// Order written by the legacy single microphone selector of the settings window until the
// priority editor replaces it (feature 001, Step 6): the chosen device moves first and the other
// saved choices keep their order; the system default empties the list.

/**
 * The priority after choosing `choice` in the single selector.
 * @param current Committed order of microphone choices.
 * @param choice Chosen device name, or null for the system default.
 * @returns The complete order to save.
 */
export const promotedOrder = (
  current: readonly string[],
  choice: string | null,
): string[] =>
  choice === null ? [] : [choice, ...current.filter((name) => name !== choice)];
