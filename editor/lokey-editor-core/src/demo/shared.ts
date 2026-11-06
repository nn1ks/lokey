import { HostLayout, KeyboardActionEntry } from "lokey-editor-odyn/keyboard";

export type ActionId = number;
export type ActionEntryId = number;

export const numKeys = 36;
export const numLayers = 4;

export type State = {
  ledArrayHooks: bigint;
  keyboardActionEntries: Map<ActionEntryId, KeyboardActionEntry>;
  keyboardLayout: Array<ActionEntryId>;
  hostLayout: HostLayout;
};
