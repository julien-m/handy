// Visibility rule of "Show the bubble when no text field is focused" (010 AC-007).
import type { OSType } from "@/lib/utils/keyboard";

// @spec AC-007: Toggle only on macOS with the gate open — .specs/features/010-recovery-bubble/spec.md#ac-007
/**
 * Business rule (plan "Rollout gate"): the settings snapshot carries the key only with the
 * `destination-return` gate open, and field detection exists on macOS only, where Windows and
 * Linux refuse writes; so the row shows only for a boolean value on macOS.
 */
export const showsNoFieldBubbleToggle = (
  value: boolean | null | undefined,
  os: OSType,
): boolean => typeof value === "boolean" && os === "macos";
