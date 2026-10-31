import * as capnp from "capnp-es";
import { State, ActionId, ActionEntryId, numKeys, numLayers } from "./shared";
import {
  HoldTapAction,
  KeyboardAction,
  KeyboardActionEntry,
  LayerAction,
  NoneAction,
  PerLayerAction,
  SequenceAction,
  TransparentAction,
} from "lokey-editor-odyn/keyboard";

import * as ledArrayHookId from "lokey-editor-odyn/led-array/hook-id";
import * as keyboardActionId from "lokey-editor-odyn/keyboard/action-id";
import {
  HoldTap,
  KeyCode,
  Layer,
  NoAction,
  Sequence,
  Transparent,
} from "../key-action";
import type { KeyAction } from "../key-action";
import { encodeKeyCode } from "../key-action/convert";

const layer0Actions: KeyAction[] = [
  KeyCode.Q,
  KeyCode.W,
  KeyCode.F,
  KeyCode.P,
  KeyCode.B,
  KeyCode.J,
  KeyCode.L,
  KeyCode.U,
  KeyCode.Y,
  KeyCode.Slash,
  KeyCode.A,
  KeyCode.R,
  KeyCode.S,
  KeyCode.T,
  KeyCode.G,
  KeyCode.M,
  KeyCode.N,
  KeyCode.E,
  KeyCode.I,
  KeyCode.O,
  KeyCode.Z,
  KeyCode.X,
  KeyCode.C,
  KeyCode.D,
  KeyCode.V,
  KeyCode.K,
  KeyCode.H,
  KeyCode.Comma,
  KeyCode.Dot,
  KeyCode.Minus,
  new HoldTap(new Layer(1), KeyCode.Escape),
  KeyCode.LShift,
  KeyCode.Space,
  KeyCode.Enter,
  new Layer(3),
  new HoldTap(new Layer(2), KeyCode.Backspace),
];

const layer1Actions: KeyAction[] = [
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
];

const layer2Actions: KeyAction[] = [
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
];

const layer3Actions: KeyAction[] = [
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
];

const actionsPerLayer: KeyAction[][] = [
  layer0Actions,
  layer1Actions,
  layer2Actions,
  layer3Actions,
];

function buildAction<T extends capnp.Struct>(
  actionId: ActionId,
  actionType: capnp.StructCtor<T>,
  setAction: (action: T) => void,
): KeyboardAction {
  const capnpMessage = new capnp.Message();
  const actionWrapper = capnpMessage.initRoot(KeyboardAction);
  actionWrapper.actionId = actionId;

  const capnpMessageInner = new capnp.Message();
  const action = capnpMessageInner.initRoot(actionType);
  setAction(action);
  const serializedAction = capnpMessageInner.toArrayBuffer();
  actionWrapper
    ._initPayload(serializedAction.byteLength)
    .copyBuffer(serializedAction);

  return actionWrapper;
}

function noneAction(): KeyboardAction {
  return buildAction(keyboardActionId.none, NoneAction, (_action) => {});
}

function transparentAction(): KeyboardAction {
  return buildAction(
    keyboardActionId.transparent,
    TransparentAction,
    (_action) => {},
  );
}

function layerAction(layer: number): KeyboardAction {
  return buildAction(keyboardActionId.layer, LayerAction, (action) => {
    action.layer = layer;
  });
}

function holdTapAction(
  holdActionEntryId: ActionEntryId,
  tapActionEntryId: ActionEntryId,
  tappingTerm: number,
): KeyboardAction {
  return buildAction(keyboardActionId.holdTap, HoldTapAction, (action) => {
    action.holdActionEntryId = holdActionEntryId;
    action.tapActionEntryId = tapActionEntryId;
    action.tappingTerm = tappingTerm;
  });
}

function sequenceAction(
  action1EntryId: ActionEntryId,
  action2EntryId: ActionEntryId,
): KeyboardAction {
  return buildAction(keyboardActionId.sequence, SequenceAction, (action) => {
    action.action1EntryId = action1EntryId;
    action.action2EntryId = action2EntryId;
  });
}

function perLayerAction(
  entryIds: ActionEntryId[],
  layers: number[],
): KeyboardAction {
  if (entryIds.length !== layers.length) {
    throw new Error(
      `perLayerAction requires the same number of entry IDs and layers, got ${entryIds.length} and ${layers.length}.`,
    );
  }
  return buildAction(keyboardActionId.perLayer, PerLayerAction, (action) => {
    const actionEntryIds = action._initActionEntryIds(entryIds.length);
    entryIds.forEach((entryId, index) => actionEntryIds.set(index, entryId));
    const actionLayers = action._initLayers(layers.length);
    layers.forEach((layer, index) => actionLayers.set(index, layer));
  });
}

function buildActionEntry(
  entryId: ActionEntryId,
  action: KeyboardAction,
): KeyboardActionEntry {
  const capnpMessage = new capnp.Message();
  const entry = capnpMessage.initRoot(KeyboardActionEntry);
  entry.entryId = entryId;
  entry.action = action;
  return entry;
}

export function defaultState(): State {
  const keyboardActionEntries = new Map<ActionEntryId, KeyboardActionEntry>();
  const keyboardLayout = new Array<ActionEntryId>();
  let nextEntryId = 0;

  const addEntry = (action: KeyboardAction): ActionEntryId => {
    const entryId = nextEntryId++;
    keyboardActionEntries.set(entryId, buildActionEntry(entryId, action));
    return entryId;
  };

  const toEntry = (action: KeyAction): ActionEntryId => {
    if (action instanceof KeyCode) {
      const actionEntries = encodeKeyCode(
        action,
        "english-us",
        () => nextEntryId++,
      );
      keyboardActionEntries.set(
        actionEntries.rootActionEntry.entryId,
        actionEntries.rootActionEntry,
      );
      for (const childEntry of actionEntries.childActionEntries) {
        keyboardActionEntries.set(childEntry.entryId, childEntry);
      }
      return actionEntries.rootActionEntry.entryId;
    }
    if (action instanceof HoldTap) {
      const holdActionEntryId = toEntry(action.holdAction);
      const tapActionEntryId = toEntry(action.tapAction);
      return addEntry(
        holdTapAction(holdActionEntryId, tapActionEntryId, action.tappingTerm),
      );
    }
    if (action instanceof Sequence) {
      const action1EntryId = toEntry(action.action1);
      const action2EntryId = toEntry(action.action2);
      return addEntry(sequenceAction(action1EntryId, action2EntryId));
    }
    if (action instanceof Layer) {
      return addEntry(layerAction(action.index));
    }
    if (action instanceof Transparent) {
      return addEntry(transparentAction());
    }
    if (action instanceof NoAction) {
      return addEntry(noneAction());
    }
    throw new Error("Invalid action type");
  };

  for (let i = 0; i < numKeys; i++) {
    const perLayerEntryIds = new Array<ActionEntryId>();
    const perLayerLayers = new Array<number>();
    for (let j = 0; j < numLayers; j++) {
      const layerActions = actionsPerLayer[j];
      const action = layerActions?.[i];
      perLayerEntryIds.push(action ? toEntry(action) : addEntry(noneAction()));
      perLayerLayers.push(j);
    }

    keyboardLayout.push(
      addEntry(perLayerAction(perLayerEntryIds, perLayerLayers)),
    );
  }

  return {
    ledArrayHooks: BigInt(
      ledArrayHookId.boot |
        ledArrayHookId.blePairing |
        ledArrayHookId.bleProfile,
    ),
    keyboardActionEntries,
    keyboardLayout,
  };
}
