import * as capnp from "capnp-es";
import type {
  KeyboardAction,
  KeyboardActionEntry,
} from "lokey-editor-odyn/keyboard";
import { PerLayerAction } from "lokey-editor-odyn/keyboard";
import type { Layout } from "../../layouts";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import type { KeyAction, RootKeyAction } from "../action";
import type { ActionRegistry } from "../registry";
import { decodeAction, encodeAction } from "./action";
import { buildActionEntry, buildPerLayerAction } from "./entries";

export interface EncodeKeyStateContext {
  readonly registry: ActionRegistry;
  readonly layout: Layout;
  allocateEntryId(): number;
  pushEntry(entry: KeyboardActionEntry): void;
}

export function encodeKeyState(
  key: RootKeyAction,
  ctx: EncodeKeyStateContext,
): number {
  const encode = (action: KeyAction): number => {
    const entries = encodeAction(action, ctx.registry, {
      layout: ctx.layout,
      encodeChild: encode,
      allocateEntryId: ctx.allocateEntryId,
    });
    ctx.pushEntry(entries.rootActionEntry);
    for (const childEntry of entries.childActionEntries) {
      ctx.pushEntry(childEntry);
    }
    return entries.rootActionEntry.entryId;
  };

  if (key.kind === "single") {
    return encode(key.action);
  }

  const actionEntryIds = key.actions.map((action) => encode(action));
  const layers = key.actions.map((_, index) => index);
  const entryId = ctx.allocateEntryId();
  ctx.pushEntry(
    buildActionEntry(entryId, buildPerLayerAction(actionEntryIds, layers)),
  );
  return entryId;
}

export function decodeKeyState(
  entry: KeyboardActionEntry,
  resolveEntry: (entryId: number) => KeyboardActionEntry | undefined,
  registry: ActionRegistry,
  layout: Layout,
  noneAction: KeyAction,
): RootKeyAction {
  const action = entry.action;

  if (action.actionId !== ActionId.perLayer) {
    return {
      kind: "single",
      action: decodeActionEntry(
        entry,
        resolveEntry,
        registry,
        layout,
        noneAction,
      ),
    };
  }

  const message = new capnp.Message(payloadBytes(action), false);
  const perLayer = message.getRoot(PerLayerAction);
  const actionEntryIds = perLayer.actionEntryIds;
  const layers = perLayer.layers;

  const actions: KeyAction[] = [];
  for (let i = 0; i < actionEntryIds.length; i++) {
    const layer = layers.get(i);
    const childEntry = resolveEntry(actionEntryIds.get(i));
    actions[layer] = childEntry
      ? decodeActionEntry(
          childEntry,
          resolveEntry,
          registry,
          layout,
          noneAction,
        )
      : noneAction;
  }

  for (let layer = 0; layer < actions.length; layer++) {
    actions[layer] ??= noneAction;
  }

  return { kind: "perLayer", actions };
}

export function decodeActionEntry(
  entry: KeyboardActionEntry,
  resolveEntry: (entryId: number) => KeyboardActionEntry | undefined,
  registry: ActionRegistry,
  layout: Layout,
  noneAction: KeyAction,
): KeyAction {
  const action = entry.action;
  return decodeAction(action.actionId, payloadBytes(action), registry, {
    layout,
    decodeChild: (entryId) => {
      const childEntry = resolveEntry(entryId);
      return childEntry
        ? decodeActionEntry(
            childEntry,
            resolveEntry,
            registry,
            layout,
            noneAction,
          )
        : noneAction;
    },
  });
}

function payloadBytes(action: KeyboardAction): Uint8Array {
  return action._hasPayload()
    ? new Uint8Array(action.payload.toArrayBuffer())
    : new Uint8Array();
}
