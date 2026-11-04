import * as capnp from "capnp-es";
import { Ellipsis } from "lucide-solid";
import { createSignal } from "solid-js";
import type { Component } from "solid-js";
import { OrderedAction } from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import { ActionPickerHeader } from "../../components/keymap/ActionPickerHeader";
import { ActionPickerChildRow } from "../../components/keymap/ActionPickerChildRow";
import type { KeyAction } from "../action";
import { buildAction, buildActionEntry } from "../codec/entries";
import { defineAction } from "../definition";
import type { PickerContext } from "../definition";
import { Labelled, Split } from "../render";
import { KeyCode, keyCodesForLayout, reverseKeyCodes } from "./key-code";

export class Ordered<
  A1 extends KeyAction = KeyAction,
  A2 extends KeyAction = KeyAction,
> implements KeyAction {
  readonly actionId = ActionId.ordered;

  constructor(
    readonly action1: A1,
    readonly action2: A2,
  ) {}
}

const withOrderedChildren = (action: Ordered, children: KeyAction[]): Ordered =>
  new Ordered(children[0]!, children[1]!);

const OrderedPicker: Component<{
  action: Ordered;
  ctx: PickerContext;
}> = (props) => {
  const [currentAction, setCurrentAction] = createSignal(props.action);

  return (
    <div class="flex flex-col gap-2">
      <ActionPickerHeader
        label="Ordered"
        onSelect={() => props.ctx.onSelect(currentAction())}
      />
      <ActionPickerChildRow
        label="1"
        action={currentAction().action1}
        onSelect={(action) => {
          const children = [currentAction().action1, currentAction().action2];
          children[0] = action;
          setCurrentAction(withOrderedChildren(currentAction(), children));
        }}
      />
      <ActionPickerChildRow
        label="2"
        action={currentAction().action2}
        onSelect={(action) => {
          const children = [currentAction().action1, currentAction().action2];
          children[1] = action;
          setCurrentAction(withOrderedChildren(currentAction(), children));
        }}
      />
    </div>
  );
};

export const orderedActionDefinition = defineAction<Ordered>({
  actionId: ActionId.ordered,
  label: "Ordered",
  encode: (action, ctx) => {
    const action1EntryId = ctx.encodeChild(action.action1);
    const action2EntryId = ctx.encodeChild(action.action2);

    return {
      rootActionEntry: buildActionEntry(
        ctx.allocateEntryId(),
        buildAction(ActionId.ordered, OrderedAction, (orderedAction) => {
          orderedAction.action1EntryId = action1EntryId;
          orderedAction.action2EntryId = action2EntryId;
        }),
      ),
      childActionEntries: [],
    };
  },
  decode: (payload, ctx) => {
    const message = new capnp.Message(payload, false);
    const orderedAction = message.getRoot(OrderedAction);
    const action1 = ctx.decodeChild(orderedAction.action1EntryId);
    const action2 = ctx.decodeChild(orderedAction.action2EntryId);

    // A sequence that encodes a single character (e.g. German "=" as
    // Shift+0) collapses back into that key code.
    if (action1 instanceof KeyCode && action2 instanceof KeyCode) {
      const encoding1 = keyCodesForLayout(ctx.layout).get(action1);
      const encoding2 = keyCodesForLayout(ctx.layout).get(action2);
      if (encoding1?.kind === "simple" && encoding2?.kind === "simple") {
        const keyCode = reverseKeyCodes(ctx.layout).sequence.get(
          `${encoding1.keyCode}:${encoding2.keyCode}`,
        );
        if (keyCode) return keyCode;
      }
    }

    return new Ordered(action1, action2);
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
    new Ordered(
      ctx.remapAction(action.action1),
      ctx.remapAction(action.action2),
    ),
  children: (action) => [action.action1, action.action2],
  renderPicker: (action, ctx) => <OrderedPicker action={action} ctx={ctx} />,
  withChildren: withOrderedChildren,
});
