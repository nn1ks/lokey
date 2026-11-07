import { KeyboardAction } from "lokey-editor-odyn/keyboard";
import { Layout } from "../layouts";
import { ActionRegistry } from "./registry";

export interface EncodeContext {
  /** The host layout used to translate key codes. */
  readonly hostLayout: Layout;

  /** Pushes an action to the context and returns the ID of the created KeyboardActionEntry. */
  pushAction(action: KeyboardAction): number;
}

export interface DecodeContext {
  /** The host layout used to translate key codes. */
  readonly hostLayout: Layout;

  /** The registry holding the action types. */
  readonly registry: ActionRegistry;

  /** Resolves an action entry ID to its encoded action. */
  resolveEntry(actionEntryId: number): KeyboardAction; // TODO: return type: | undefined
}

export interface PickerContext {
  /** Selects the action and closes the picker. */
  onSelect(action: import("./action").KeyAction, close: boolean): void;
}

export interface RemapLayersContext {
  /** Maps a layer index through a layer move. */
  remapLayer(index: number): number;
}
