import type { KeyAction } from "../action";
import type { DecodeContext, EncodeContext } from "../definition";
import type { ActionRegistry } from "../registry";
import { UnknownAction } from "../unknown";
import type { ActionEntries } from "./entries";
import { buildActionEntry, buildRawAction } from "./entries";

export function encodeAction(
  action: KeyAction,
  registry: ActionRegistry,
  ctx: EncodeContext,
): ActionEntries {
  if (action instanceof UnknownAction) {
    return {
      rootActionEntry: buildActionEntry(
        ctx.allocateEntryId(),
        buildRawAction(action.actionId, action.payload),
      ),
      childActionEntries: [],
    };
  }

  const definition = registry.byId(action.actionId);
  if (!definition) {
    throw new Error(
      `No action definition is registered for action ID ${action.actionId}.`,
    );
  }
  return definition.encode(action, ctx);
}

export function decodeAction(
  actionId: number,
  payload: Uint8Array,
  registry: ActionRegistry,
  ctx: DecodeContext,
): KeyAction {
  const definition = registry.byId(actionId);
  if (!definition) {
    return new UnknownAction(actionId, payload);
  }
  return definition.decode(payload, ctx);
}
