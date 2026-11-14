import {
  KeyboardAction,
  KeyboardActionEntry,
} from "lokey-editor-odyn/keyboard";
import { Layout } from "../layouts";
import { ActionRegistry } from "./registry";

export interface EncodeContext {
  /** The host layout used to translate key codes. */
  readonly hostLayout: Layout;

  /** Returns the next available action entry ID. */
  nextActionEntryId(): number;

  /** Pushes an action entry to the context. */
  pushActionEntry(action: KeyboardActionEntry): void;
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
  selectAction(): void;

  addAction(action: import("./action").KeyAction): void;

  closePicker(): void;
}

export interface RemapLayersContext {
  /** Converts the given layer into the new layer as part of the remap. */
  convertLayer(index: number): number;
}
