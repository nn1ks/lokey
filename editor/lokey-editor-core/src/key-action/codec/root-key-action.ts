import * as capnp from "capnp-es";
import {
  KeyboardActionEntry,
  PerLayerAction,
} from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import { KeyAction, RootKeyAction } from "../action";
import { DecodeContext, EncodeContext } from "../contexts";
import assertNever from "assert-never";
import { buildPerLayerAction } from "./build";
import { None } from "../actions/none";
import { decodeAction } from "./decode";

export function encodeRootKeyAction(
  key: RootKeyAction,
  ctx: EncodeContext,
): number {
  if (key.kind == "single") {
    return ctx.pushAction(key.action.encode(ctx));
  }

  if (key.kind == "perLayer") {
    const actionEntryIds = key.actions.map((action) =>
      ctx.pushAction(action.encode(ctx)),
    );
    const layers = key.actions.map((_, index) => index);
    return ctx.pushAction(buildPerLayerAction(actionEntryIds, layers));
  }

  assertNever(key);
}

export function decodeRootKeyAction(
  entry: KeyboardActionEntry,
  ctx: DecodeContext,
): RootKeyAction {
  if (entry.action.actionId !== ActionId.perLayer) {
    return {
      kind: "single",
      action: decodeAction(entry.action, ctx),
    };
  }

  const message = new capnp.Message(
    entry.action.payload.toArrayBuffer(),
    false,
  );
  const perLayer = message.getRoot(PerLayerAction);
  const actionEntryIds = perLayer.actionEntryIds;
  const layers = perLayer.layers;

  const actions: KeyAction[] = [];
  for (let i = 0; i < actionEntryIds.length; i++) {
    const layer = layers.get(i);
    const childEntry = decodeAction(
      ctx.resolveEntry(actionEntryIds.get(i)),
      ctx,
    );
    actions[layer] = childEntry;
  }

  for (let layer = 0; layer < actions.length; layer++) {
    actions[layer] ??= new None();
  }

  return { kind: "perLayer", actions };
}
