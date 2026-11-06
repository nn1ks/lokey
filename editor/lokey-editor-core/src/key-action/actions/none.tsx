import * as capnp from "capnp-es";
import { CircleOff } from "lucide-solid";
import { KeyboardAction, NoneAction } from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import type { KeyAction } from "../action";
import { buildAction, buildActionEntry } from "../codec/entries";
import { defineAction } from "../definition";

export class None implements KeyAction {
  readonly actionId = ActionId.none;

  encode(): KeyboardAction {
    return buildAction(ActionId.none, NoneAction, () => {});
  }

  static decode(payload: Uint8Array): None {
    const message = new capnp.Message(payload, false);
    message.getRoot(NoneAction);
    return new None();
  }
}

export const noneActionDefinition = defineAction<None>({
  actionId: ActionId.none,
  label: "None",
  encode: (action, ctx) => ({
    rootActionEntry: buildActionEntry(ctx.allocateEntryId(), action.encode()),
    childActionEntries: [],
  }),
  decode: (payload) => None.decode(payload),
  render: () => <CircleOff />,
  renderTooltip: () => <>None</>,
  remap: (action) => action,
  children: () => [],
});
