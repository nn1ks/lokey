import * as odynKeyboard from "lokey-editor-odyn/keyboard";
import * as odynKeyboardActionId from "lokey-editor-odyn/keyboard/action-id";
import * as capnp from "capnp-es";
import { HoldTap, Layer, NoAction, Sequence, Transparent } from "./index";
import type { KeyAction, KeyCode, RootKeyAction } from "./index";
import type { Layout } from "../layouts";
import { englishUsKeyCodes } from "./layouts/english-us";
import { germanKeyCodes } from "./layouts/german";
import type { KeyCodeEncoding } from "./layouts/types";

export type { Layout } from "../layouts";

export type ActionEntries = {
  rootActionEntry: odynKeyboard.KeyboardActionEntry;
  childActionEntries: odynKeyboard.KeyboardActionEntry[];
};

type KeyCodeType = "Keyboard" | "Consumer";

function keyCode(value: number, type: KeyCodeType): capnp.Message {
  const message = new capnp.Message();
  const keyCodeAction = message.initRoot(odynKeyboard.KeyCodeAction);
  keyCodeAction.keyCode = value;
  switch (type) {
    case "Keyboard":
      keyCodeAction.type = odynKeyboard.KeyCodeType.KEYBOARD;
      break;
    case "Consumer":
      keyCodeAction.type = odynKeyboard.KeyCodeType.CONSUMER;
      break;
  }
  return message;
}

function sequence(
  action1EntryId: number,
  action2EntryId: number,
): capnp.Message {
  const message = new capnp.Message();
  const sequenceAction = message.initRoot(odynKeyboard.SequenceAction);
  sequenceAction.action1EntryId = action1EntryId;
  sequenceAction.action2EntryId = action2EntryId;
  return message;
}

type KeyboardActionType = "KeyCode" | "Sequence";

function keyboardAction(
  type: KeyboardActionType,
  keyCode: capnp.Message,
): odynKeyboard.KeyboardAction {
  const message = new capnp.Message();
  const keyboardAction = message.initRoot(odynKeyboard.KeyboardAction);
  switch (type) {
    case "KeyCode":
      keyboardAction.actionId = odynKeyboardActionId.keyCode;
      break;
    case "Sequence":
      keyboardAction.actionId = odynKeyboardActionId.sequence;
      break;
  }
  const buffer = keyCode.toArrayBuffer();
  keyboardAction._initPayload(buffer.byteLength).copyBuffer(buffer);
  return keyboardAction;
}

function keyboardActionEntry(
  entryId: number,
  action: odynKeyboard.KeyboardAction,
): odynKeyboard.KeyboardActionEntry {
  const message = new capnp.Message();
  const keyboardActionEntry = message.initRoot(
    odynKeyboard.KeyboardActionEntry,
  );
  keyboardActionEntry.entryId = entryId;
  keyboardActionEntry.action = action;
  return keyboardActionEntry;
}

function encodeKeyCodeSimple(
  value: number,
  nextEntryId: () => number,
): ActionEntries {
  return {
    rootActionEntry: keyboardActionEntry(
      nextEntryId(),
      keyboardAction("KeyCode", keyCode(value, "Keyboard")),
    ),
    childActionEntries: [],
  };
}

function encodeKeyCodeSequence(
  keyCode1: number,
  keyCode2: number,
  nextEntryId: () => number,
): ActionEntries {
  const entryId1 = nextEntryId();
  const entryId2 = nextEntryId();

  const entry1 = keyboardActionEntry(
    entryId1,
    keyboardAction("KeyCode", keyCode(keyCode1, "Keyboard")),
  );
  const entry2 = keyboardActionEntry(
    entryId2,
    keyboardAction("KeyCode", keyCode(keyCode2, "Keyboard")),
  );

  const sequenceAction = keyboardAction(
    "Sequence",
    sequence(entryId1, entryId2),
  );

  return {
    rootActionEntry: keyboardActionEntry(nextEntryId(), sequenceAction),
    childActionEntries: [entry1, entry2],
  };
}

function encodeEncoding(
  encoding: KeyCodeEncoding,
  nextEntryId: () => number,
): ActionEntries {
  switch (encoding.kind) {
    case "simple":
      return encodeKeyCodeSimple(encoding.keyCode, nextEntryId);
    case "sequence":
      return encodeKeyCodeSequence(
        encoding.modifier,
        encoding.keyCode,
        nextEntryId,
      );
  }
}

function keyCodesForLayout(layout: Layout): Map<KeyCode, KeyCodeEncoding> {
  switch (layout) {
    case "english-us":
      return englishUsKeyCodes;
    case "german":
      return germanKeyCodes;
  }
}

export function encodeKeyCode(
  keyCodeAction: KeyCode,
  layout: Layout,
  nextEntryId: () => number,
): ActionEntries {
  const encoding = keyCodesForLayout(layout).get(keyCodeAction);
  if (!encoding) {
    throw new Error(`Unexpected key code "${keyCodeAction.value}".`);
  }
  return encodeEncoding(encoding, nextEntryId);
}

type ReverseKeyCodes = {
  simple: Map<number, KeyCode>;
  sequence: Map<string, KeyCode>;
};

function buildReverseKeyCodes(
  encodings: Map<KeyCode, KeyCodeEncoding>,
): ReverseKeyCodes {
  const simple = new Map<number, KeyCode>();
  const sequence = new Map<string, KeyCode>();

  for (const [keyCode, encoding] of encodings) {
    switch (encoding.kind) {
      case "simple":
        simple.set(encoding.keyCode, keyCode);
        break;
      case "sequence":
        sequence.set(`${encoding.modifier}:${encoding.keyCode}`, keyCode);
        break;
    }
  }

  return { simple, sequence };
}

const englishUsReverseKeyCodes = buildReverseKeyCodes(englishUsKeyCodes);
const germanReverseKeyCodes = buildReverseKeyCodes(germanKeyCodes);

function reverseKeyCodes(layout: Layout): ReverseKeyCodes {
  switch (layout) {
    case "english-us":
      return englishUsReverseKeyCodes;
    case "german":
      return germanReverseKeyCodes;
  }
}

function payloadBytes(action: odynKeyboard.KeyboardAction): Uint8Array {
  return action._hasPayload()
    ? new Uint8Array(action.payload.toArrayBuffer())
    : new Uint8Array();
}

function decodeKeyCodeAction(
  action: odynKeyboard.KeyboardAction,
  layout: Layout,
): KeyCode {
  const message = new capnp.Message(payloadBytes(action), false);
  const keyCodeAction = message.getRoot(odynKeyboard.KeyCodeAction);

  if (keyCodeAction.type !== odynKeyboard.KeyCodeType.KEYBOARD) {
    throw new Error("Consumer key codes are not supported yet.");
  }

  const keyCode = reverseKeyCodes(layout).simple.get(keyCodeAction.keyCode);
  if (!keyCode) {
    throw new Error(
      `Unknown key code 0x${keyCodeAction.keyCode.toString(16)} for layout ${layout}.`,
    );
  }

  return keyCode;
}

/** Returns the HID usage of a key code action entry, if it is one. */
function keyCodeHid(
  entry: odynKeyboard.KeyboardActionEntry | undefined,
): number | undefined {
  if (!entry) {
    return undefined;
  }

  const action = entry.action;
  if (action.actionId !== odynKeyboardActionId.keyCode) {
    return undefined;
  }

  const message = new capnp.Message(payloadBytes(action), false);
  const keyCodeAction = message.getRoot(odynKeyboard.KeyCodeAction);
  if (keyCodeAction.type !== odynKeyboard.KeyCodeType.KEYBOARD) {
    return undefined;
  }

  return keyCodeAction.keyCode;
}

export function decodeKeyboardActionEntry(
  entry: odynKeyboard.KeyboardActionEntry,
  resolveEntry: (
    entryId: number,
  ) => odynKeyboard.KeyboardActionEntry | undefined,
  layout: Layout,
): KeyAction {
  const action = entry.action;

  switch (action.actionId) {
    case odynKeyboardActionId.none:
      return new NoAction();
    case odynKeyboardActionId.transparent:
      return new Transparent();
    case odynKeyboardActionId.keyCode:
      return decodeKeyCodeAction(action, layout);
    case odynKeyboardActionId.layer: {
      const message = new capnp.Message(payloadBytes(action), false);
      return new Layer(message.getRoot(odynKeyboard.LayerAction).layer);
    }
    case odynKeyboardActionId.holdTap: {
      const message = new capnp.Message(payloadBytes(action), false);
      const holdTap = message.getRoot(odynKeyboard.HoldTapAction);
      const holdEntry = resolveEntry(holdTap.holdActionEntryId);
      const tapEntry = resolveEntry(holdTap.tapActionEntryId);

      return new HoldTap(
        holdEntry
          ? decodeKeyboardActionEntry(holdEntry, resolveEntry, layout)
          : new NoAction(),
        tapEntry
          ? decodeKeyboardActionEntry(tapEntry, resolveEntry, layout)
          : new NoAction(),
        holdTap.tappingTerm,
      );
    }
    case odynKeyboardActionId.sequence: {
      const message = new capnp.Message(payloadBytes(action), false);
      const sequenceAction = message.getRoot(odynKeyboard.SequenceAction);
      const entry1 = resolveEntry(sequenceAction.action1EntryId);
      const entry2 = resolveEntry(sequenceAction.action2EntryId);

      // A sequence that encodes a single character (e.g. German "=" as
      // Shift+0) collapses back into that key code.
      const hid1 = keyCodeHid(entry1);
      const hid2 = keyCodeHid(entry2);
      if (hid1 !== undefined && hid2 !== undefined) {
        const keyCode = reverseKeyCodes(layout).sequence.get(`${hid1}:${hid2}`);
        if (keyCode) {
          return keyCode;
        }
      }

      return new Sequence(
        entry1
          ? decodeKeyboardActionEntry(entry1, resolveEntry, layout)
          : new NoAction(),
        entry2
          ? decodeKeyboardActionEntry(entry2, resolveEntry, layout)
          : new NoAction(),
      );
    }
    case odynKeyboardActionId.perLayer:
      throw new Error(
        "A per-layer action is not a leaf action; use decodeKeyState.",
      );
    default:
      throw new Error(`Unknown keyboard action ID ${action.actionId}.`);
  }
}

export function decodeKeyState(
  entry: odynKeyboard.KeyboardActionEntry,
  resolveEntry: (
    entryId: number,
  ) => odynKeyboard.KeyboardActionEntry | undefined,
  layout: Layout,
): RootKeyAction {
  const action = entry.action;

  if (action.actionId !== odynKeyboardActionId.perLayer) {
    return {
      kind: "single",
      action: decodeKeyboardActionEntry(entry, resolveEntry, layout),
    };
  }

  const message = new capnp.Message(payloadBytes(action), false);
  const perLayer = message.getRoot(odynKeyboard.PerLayerAction);
  const actionEntryIds = perLayer.actionEntryIds;
  const layers = perLayer.layers;

  const actions: KeyAction[] = [];
  for (let i = 0; i < actionEntryIds.length; i++) {
    const layer = layers.get(i);
    const childEntry = resolveEntry(actionEntryIds.get(i));
    actions[layer] = childEntry
      ? decodeKeyboardActionEntry(childEntry, resolveEntry, layout)
      : new NoAction();
  }

  for (let layer = 0; layer < actions.length; layer++) {
    actions[layer] ??= new NoAction();
  }

  return { kind: "perLayer", actions };
}
