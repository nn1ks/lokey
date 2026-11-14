import {
  KeyboardAction,
  KeyboardActionEntry,
} from "lokey-editor-odyn/keyboard";
import { KeyAction } from "../action";
import { DecodeContext } from "../contexts";
import { UnknownAction } from "../actions/unknown";

export function decodeActionEntry(
  actionEntry: KeyboardActionEntry,
  ctx: DecodeContext,
): KeyAction {
  return decodeAction(actionEntry.entryId, actionEntry.action, ctx);
}

export function decodeAction(
  actionEntryId: number | undefined,
  action: KeyboardAction,
  ctx: DecodeContext,
): KeyAction {
  const cls = ctx.registry.byId(action.actionId);
  if (!cls) {
    return new UnknownAction(
      undefined,
      action.actionId,
      new Uint8Array(action.payload.toArrayBuffer()),
    );
  }
  return cls.decode(
    actionEntryId,
    new Uint8Array(action.payload.toArrayBuffer()),
    ctx,
  );
}
