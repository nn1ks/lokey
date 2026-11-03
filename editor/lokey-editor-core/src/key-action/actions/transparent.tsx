import { CircleChevronDown } from "lucide-solid";
import { TransparentAction } from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import type { KeyAction } from "../action";
import { buildAction, buildActionEntry } from "../codec/entries";
import { defineAction } from "../definition";

export class Transparent implements KeyAction {
  readonly actionId = ActionId.transparent;
}

export const transparentActionDefinition = defineAction<Transparent>({
  actionId: ActionId.transparent,
  label: "Transparent",
  encode: (_action, ctx) => ({
    rootActionEntry: buildActionEntry(
      ctx.allocateEntryId(),
      buildAction(ActionId.transparent, TransparentAction, () => {}),
    ),
    childActionEntries: [],
  }),
  decode: () => new Transparent(),
  render: () => <CircleChevronDown />,
  renderTooltip: () => <>Transparent</>,
  remap: (action) => action,
  children: () => [],
});
