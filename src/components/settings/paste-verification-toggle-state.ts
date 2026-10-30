// Visibility rule of the experimental "Verify the paste" switch (007 AC-019).

// @spec AC-019: Verification only with smart paste — .specs/features/007-smart-paste/spec.md#ac-019
/**
 * Business rule (007 AC-019): the verification runs only after the smart-paste Cmd+V, so the
 * switch shows in the Experimental section only while smart paste is the paste method (smart
 * paste is selectable only where its rollout gate is open, on macOS).
 */
export const showsPasteVerificationToggle = (
  pasteMethod: string | null | undefined,
): boolean => pasteMethod === "smart_paste";
