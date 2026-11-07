import * as capnp from "capnp-es";
import {
  KeyboardAction,
  KeyboardActionEntry,
  PerLayerAction,
} from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";

export function buildAction<T extends capnp.Struct>(
  actionId: number,
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

/** Builds an action from an already serialized payload. */
export function buildRawAction(
  actionId: number,
  payload: Uint8Array,
): KeyboardAction {
  const capnpMessage = new capnp.Message();
  const actionWrapper = capnpMessage.initRoot(KeyboardAction);
  actionWrapper.actionId = actionId;
  actionWrapper._initPayload(payload.byteLength).copyBuffer(payload);
  return actionWrapper;
}

export function buildActionEntry(
  entryId: number,
  action: KeyboardAction,
): KeyboardActionEntry {
  const capnpMessage = new capnp.Message();
  const entry = capnpMessage.initRoot(KeyboardActionEntry);
  entry.entryId = entryId;
  entry.action = action;
  return entry;
}

export function buildPerLayerAction(
  actionEntryIds: number[],
  layers: number[],
): KeyboardAction {
  if (actionEntryIds.length !== layers.length) {
    throw new Error(
      `A per-layer action requires the same number of entry IDs and layers, got ${actionEntryIds.length} and ${layers.length}.`,
    );
  }

  return buildAction(ActionId.perLayer, PerLayerAction, (action) => {
    const ids = action._initActionEntryIds(actionEntryIds.length);
    actionEntryIds.forEach((entryId, index) => ids.set(index, entryId));
    const layerList = action._initLayers(layers.length);
    layers.forEach((layer, index) => layerList.set(index, layer));
  });
}
