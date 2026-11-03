import type { KeyAction } from "./action";

/**
 * An action whose definition is not registered in the editor. The raw payload
 * is preserved so the action can be written back unchanged.
 */
export class UnknownAction implements KeyAction {
  constructor(
    readonly actionId: number,
    readonly payload: Uint8Array,
  ) {}
}
