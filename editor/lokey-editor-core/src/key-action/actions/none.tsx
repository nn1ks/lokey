import { CircleOff } from "lucide-solid";
import { NoneAction } from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import type { KeyAction } from "../action";
import { buildAction, buildActionEntry } from "../codec/entries";
import { defineAction } from "../definition";

export class None implements KeyAction {
  readonly actionId = ActionId.none;
}

export const noneActionDefinition = defineAction<None>({
  actionId: ActionId.none,
  label: "None",
  encode: (_action, ctx) => ({
    rootActionEntry: buildActionEntry(
      ctx.allocateEntryId(),
      buildAction(ActionId.none, NoneAction, () => {}),
    ),
    childActionEntries: [],
  }),
  decode: () => new None(),
  render: () => <CircleOff />,
  renderTooltip: () => <>None</>,
  remap: (action) => action,
  children: () => [],
});
