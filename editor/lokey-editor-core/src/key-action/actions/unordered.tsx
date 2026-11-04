import * as capnp from "capnp-es";
import { Ellipsis } from "lucide-solid";
import { createSignal } from "solid-js";
import type { Component } from "solid-js";
import { UnorderedAction } from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import { ActionPickerHeader } from "../../components/keymap/ActionPickerHeader";
import { ActionPickerChildRow } from "../../components/keymap/ActionPickerChildRow";
import type { KeyAction } from "../action";
import { buildAction, buildActionEntry } from "../codec/entries";
import { defineAction } from "../definition";
import type { PickerContext } from "../definition";
import { Split } from "../render";

export class Unordered<
  A1 extends KeyAction = KeyAction,
  A2 extends KeyAction = KeyAction,
> implements KeyAction {
  readonly actionId = ActionId.unordered;

  constructor(
    readonly action1: A1,
    readonly action2: A2,
  ) {}
}

const withUnorderedChildren = (
  action: Unordered,
  children: KeyAction[],
): Unordered => new Unordered(children[0]!, children[1]!);

const UnorderedPicker: Component<{
  action: Unordered;
  ctx: PickerContext;
}> = (props) => {
  const [currentAction, setCurrentAction] = createSignal(props.action);

  return (
    <div class="flex flex-col gap-2">
      <ActionPickerHeader
        label="Unordered"
        onSelect={() => props.ctx.onSelect(currentAction(), true)}
      />
      <ActionPickerChildRow
        label="1"
        action={currentAction().action1}
        onSelect={(action) => {
          const children = [currentAction().action1, currentAction().action2];
          children[0] = action;
          setCurrentAction(withUnorderedChildren(currentAction(), children));
          props.ctx.onSelect(currentAction(), false);
        }}
      />
      <ActionPickerChildRow
        label="2"
        action={currentAction().action2}
        onSelect={(action) => {
          const children = [currentAction().action1, currentAction().action2];
          children[1] = action;
          setCurrentAction(withUnorderedChildren(currentAction(), children));
          props.ctx.onSelect(currentAction(), false);
        }}
      />
    </div>
  );
};

export const unorderedActionDefinition = defineAction<Unordered>({
  actionId: ActionId.unordered,
  label: "Unordered",
  encode: (action, ctx) => {
    const action1EntryId = ctx.encodeChild(action.action1);
    const action2EntryId = ctx.encodeChild(action.action2);
    return {
      rootActionEntry: buildActionEntry(
        ctx.allocateEntryId(),
        buildAction(ActionId.unordered, UnorderedAction, (orderedAction) => {
          orderedAction.action1EntryId = action1EntryId;
          orderedAction.action2EntryId = action2EntryId;
        }),
      ),
      childActionEntries: [],
    };
  },
  decode: (payload, ctx) => {
    const message = new capnp.Message(payload, false);
    const orderedAction = message.getRoot(UnorderedAction);
    const action1 = ctx.decodeChild(orderedAction.action1EntryId);
    const action2 = ctx.decodeChild(orderedAction.action2EntryId);
    return new Unordered(action1, action2);
  },
  render: (action, level, ctx) => {
    if (level > 0) return <Ellipsis />;

    return (
      <Split
        action1={ctx.renderAction(action.action1, level + 1)}
        action2={ctx.renderAction(action.action2, level + 1)}
      />
    );
  },
  renderTooltip: (action, ctx) => (
    <div class="flex items-stretch gap-2">
      <div class="flex min-w-12 flex-col items-center gap-1">
        <div class="text-neutral-500 text-[0.65rem] font-semibold tracking-[0.08em] uppercase">
          1
        </div>
        <div class="flex min-h-6 items-center justify-center">
          {ctx.renderTooltip(action.action1)}
        </div>
      </div>
      <div class="w-px bg-neutral-600 dark:bg-neutral-400" />
      <div class="flex min-w-12 flex-col items-center gap-1">
        <div class="text-neutral-500 text-[0.65rem] font-semibold tracking-[0.08em] uppercase">
          2
        </div>
        <div class="flex min-h-6 items-center justify-center">
          {ctx.renderTooltip(action.action2)}
        </div>
      </div>
    </div>
  ),
  remap: (action, ctx) =>
    new Unordered(
      ctx.remapAction(action.action1),
      ctx.remapAction(action.action2),
    ),
  children: (action) => [action.action1, action.action2],
  renderPicker: (action, ctx) => <UnorderedPicker action={action} ctx={ctx} />,
  withChildren: withUnorderedChildren,
});
