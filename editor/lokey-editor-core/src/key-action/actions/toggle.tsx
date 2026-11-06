import { Accordion } from "@ark-ui/solid";
import * as capnp from "capnp-es";
import { Ellipsis } from "lucide-solid";
import { createSignal } from "solid-js";
import type { Component } from "solid-js";
import { ToggleAction } from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import { ActionPickerChildRow } from "../../components/keymap/ActionPickerChildRow";
import { ActionPickerHeader } from "../../components/keymap/ActionPickerHeader";
import type { KeyAction } from "../action";
import { buildAction, buildActionEntry } from "../codec/entries";
import { defineAction } from "../definition";
import type { PickerContext } from "../definition";
import { Labelled } from "../render";

export class Toggle<A extends KeyAction = KeyAction> implements KeyAction {
  readonly actionId = ActionId.toggle;

  constructor(readonly action: A) {}
}

const withToggleChildren = (action: Toggle, children: KeyAction[]): Toggle =>
  new Toggle(children[0]!);

const TogglePicker: Component<{
  action: Toggle;
  ctx: PickerContext;
}> = (props) => {
  const [currentAction, setCurrentAction] = createSignal(props.action);

  return (
    <>
      <ActionPickerHeader
        label="Toggle"
        withAccordionTrigger={true}
        onSelect={() => props.ctx.onSelect(currentAction(), true)}
      />
      <Accordion.ItemContent class="data-[state=closed]:hidden">
        <div class="flex flex-col gap-2 p-2 pt-0">
          <ActionPickerChildRow
            label="Action"
            action={currentAction().action}
            onSelect={(action) => {
              const next = new Toggle(action);
              setCurrentAction(next);
              props.ctx.onSelect(next, false);
            }}
          />
        </div>
      </Accordion.ItemContent>
    </>
  );
};

export const toggleActionDefinition = defineAction<Toggle>({
  actionId: ActionId.toggle,
  label: "Toggle",
  encode: (action, ctx) => {
    const actionEntryId = ctx.encodeChild(action.action);

    return {
      rootActionEntry: buildActionEntry(
        ctx.allocateEntryId(),
        buildAction(ActionId.toggle, ToggleAction, (toggleAction) => {
          toggleAction.actionEntryId = actionEntryId;
        }),
      ),
      childActionEntries: [],
    };
  },
  decode: (payload, ctx) => {
    const message = new capnp.Message(payload, false);
    const toggle = message.getRoot(ToggleAction);
    return new Toggle(ctx.decodeChild(toggle.actionEntryId));
  },
  render: (action, level, ctx) => {
    if (level > 0) return <Ellipsis />;

    return (
      <Labelled label="Toggle">
        {ctx.renderAction(action.action, level + 1)}
      </Labelled>
    );
  },
  renderTooltip: (action, ctx) => (
    <div class="flex flex-col items-center gap-1">
      <div class="text-neutral-500 text-[0.65rem] font-semibold tracking-[0.08em] uppercase">
        Toggle
      </div>
      <div class="flex min-h-6 items-center justify-center">
        {ctx.renderTooltip(action.action)}
      </div>
    </div>
  ),
  remap: (action, ctx) => new Toggle(ctx.remapAction(action.action)),
  children: (action) => [action.action],
  renderPicker: (action, ctx) => <TogglePicker action={action} ctx={ctx} />,
  withChildren: withToggleChildren,
});
