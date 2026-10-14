// Typed refusal of a shortcut change, thrown by the settings store so the
// caller can translate it instead of showing the backend's English text.
import type { BindingResponse, ShortcutEditError } from "@/bindings";

/** A shortcut change the backend refused, with its typed reason. */
export class ShortcutEditFailure extends Error {
  readonly editError: ShortcutEditError;

  /**
   * @param editError Typed refusal from the backend.
   * @param message English description, kept for logs.
   */
  constructor(editError: ShortcutEditError, message: string) {
    super(message);
    this.name = "ShortcutEditFailure";
    this.editError = editError;
  }
}

/**
 * Error to throw for a refused legacy `change_binding`/`reset_binding`
 * response: typed when the backend gave the reason, plain otherwise.
 * @param response Response with `success: false`.
 * @returns The error describing the refusal.
 */
export const bindingFailure = (response: BindingResponse): Error => {
  const message = response.error || "Failed to update binding";
  return response.edit_error
    ? new ShortcutEditFailure(response.edit_error, message)
    : new Error(message);
};
