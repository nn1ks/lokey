import { CircleOff } from "lucide-solid";
import { NoneAction } from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import type { KeyAction } from "../action";
import { buildAction, buildActionEntry } from "../codec/entries";
import { defineAction } from "../definition";

export class NoAction implements KeyAction {
  readonly actionId = ActionId.none;
}

export const noActionDefinition = defineAction<NoAction>({
  actionId: ActionId.none,
  label: "None",
  encode: (_action, ctx) => ({
    rootActionEntry: buildActionEntry(
      ctx.allocateEntryId(),
      buildAction(ActionId.none, NoneAction, () => {}),
    ),
    childActionEntries: [],
  }),
  decode: () => new NoAction(),
  render: () => <CircleOff />,
  renderTooltip: () => <>None</>,
  remap: (action) => action,
  children: () => [],
});
