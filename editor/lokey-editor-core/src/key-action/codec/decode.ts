import { KeyboardAction } from "lokey-editor-odyn/keyboard";
import { KeyAction } from "../action";
import { DecodeContext } from "../contexts";
import { UnknownAction } from "../actions/unknown";

export function decodeAction(
  action: KeyboardAction,
  ctx: DecodeContext,
): KeyAction {
  const cls = ctx.registry.byId(action.actionId);
  if (!cls) {
    return new UnknownAction(
      action.actionId,
      new Uint8Array(action.payload.toArrayBuffer()),
    );
  }
  return cls.decode(new Uint8Array(action.payload.toArrayBuffer()), ctx);
}
