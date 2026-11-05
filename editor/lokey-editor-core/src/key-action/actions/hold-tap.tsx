import * as capnp from "capnp-es";
import { ChevronRight, Ellipsis } from "lucide-solid";
import { createSignal } from "solid-js";
import type { Component } from "solid-js";
import { HoldTapAction } from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import { ActionPickerHeader } from "../../components/keymap/ActionPickerHeader";
import { ActionPickerChildRow } from "../../components/keymap/ActionPickerChildRow";
import { ActionPickerDurationRow } from "../../components/keymap/ActionPickerDurationRow";
import type { KeyAction } from "../action";
import { buildAction, buildActionEntry } from "../codec/entries";
import { defineAction } from "../definition";
import type { PickerContext } from "../definition";
import { Labelled, Split } from "../render";
import { Accordion } from "@ark-ui/solid/accordion";

export class HoldTap<
  H extends KeyAction = KeyAction,
  T extends KeyAction = KeyAction,
> implements KeyAction {
  readonly actionId = ActionId.holdTap;

  constructor(
    readonly holdAction: H,
    readonly tapAction: T,
    /** Tapping term in milliseconds. */
    readonly tappingTerm: number = 200,
  ) {}
}

const withHoldTapChildren = (action: HoldTap, children: KeyAction[]): HoldTap =>
  new HoldTap(children[0]!, children[1]!, action.tappingTerm);

const withHoldTapTappingTerm = (
  action: HoldTap,
  tappingTerm: number,
): HoldTap => new HoldTap(action.holdAction, action.tapAction, tappingTerm);

const HoldTapPicker: Component<{
  action: HoldTap;
  ctx: PickerContext;
}> = (props) => {
  const [currentAction, setCurrentAction] = createSignal(props.action);

  return (
    <>
      <ActionPickerHeader
        label="Hold Tap"
        withAccordionTrigger={true}
        onSelect={() => props.ctx.onSelect(currentAction(), true)}
      />
      <Accordion.ItemContent class="data-[state=closed]:hidden">
        <div class="flex flex-col gap-2 p-2 pt-0">
          <ActionPickerDurationRow
            label="Tapping Term"
            value={currentAction().tappingTerm}
            min={0}
            max={10000}
            step={50}
            unit="ms"
            onValueCommit={() => props.ctx.onSelect(currentAction(), false)}
            onValueChange={(value) => {
              setCurrentAction(withHoldTapTappingTerm(currentAction(), value));
            }}
          />
          <ActionPickerChildRow
            label="Hold"
            action={currentAction().holdAction}
            onSelect={(action) => {
              setCurrentAction(
                withHoldTapChildren(currentAction(), [
                  action,
                  currentAction().tapAction,
                ]),
              );
              props.ctx.onSelect(currentAction(), false);
            }}
          />
          <ActionPickerChildRow
            label="Tap"
            action={currentAction().tapAction}
            onSelect={(action) => {
              setCurrentAction(
                withHoldTapChildren(currentAction(), [
                  currentAction().holdAction,
                  action,
                ]),
              );
              props.ctx.onSelect(currentAction(), false);
            }}
          />
        </div>
      </Accordion.ItemContent>
    </>
  );
};

export const holdTapActionDefinition = defineAction<HoldTap>({
  actionId: ActionId.holdTap,
  label: "Hold Tap",
  encode: (action, ctx) => {
    const holdActionEntryId = ctx.encodeChild(action.holdAction);
    const tapActionEntryId = ctx.encodeChild(action.tapAction);

    return {
      rootActionEntry: buildActionEntry(
        ctx.allocateEntryId(),
        buildAction(ActionId.holdTap, HoldTapAction, (holdTapAction) => {
          holdTapAction.holdActionEntryId = holdActionEntryId;
          holdTapAction.tapActionEntryId = tapActionEntryId;
          holdTapAction.tappingTerm = action.tappingTerm;
        }),
      ),
      childActionEntries: [],
    };
  },
  decode: (payload, ctx) => {
    const message = new capnp.Message(payload, false);
    const holdTap = message.getRoot(HoldTapAction);
    return new HoldTap(
      ctx.decodeChild(holdTap.holdActionEntryId),
      ctx.decodeChild(holdTap.tapActionEntryId),
      holdTap.tappingTerm,
    );
  },
  render: (action, level, ctx) => {
    if (level > 0) return <Ellipsis />;

    return (
      <Split
        action1={
          <Labelled label="Hold">
            {ctx.renderAction(action.holdAction, level + 1)}
          </Labelled>
        }
        action2={
          <Labelled label="Tap">
            {ctx.renderAction(action.tapAction, level + 1)}
          </Labelled>
        }
      />
    );
  },
  renderTooltip: (action, ctx) => (
    <div class="flex items-stretch gap-2">
      <div class="flex min-w-12 flex-col items-center gap-1">
        <div class="text-neutral-500 text-[0.65rem] font-semibold tracking-[0.08em] uppercase">
          Hold
        </div>
        <div class="flex min-h-6 items-center justify-center">
          {ctx.renderTooltip(action.holdAction)}
        </div>
      </div>
      <div class="w-px bg-neutral-600 dark:bg-neutral-400" />
      <div class="flex min-w-12 flex-col items-center gap-1">
        <div class="text-neutral-500 text-[0.65rem] font-semibold tracking-[0.08em] uppercase">
          Tap
        </div>
        <div class="flex min-h-6 items-center justify-center">
          {ctx.renderTooltip(action.tapAction)}
        </div>
      </div>
    </div>
  ),
  remap: (action, ctx) =>
    new HoldTap(
      ctx.remapAction(action.holdAction),
      ctx.remapAction(action.tapAction),
      action.tappingTerm,
    ),
  children: (action) => [action.holdAction, action.tapAction],
  renderPicker: (action, ctx) => <HoldTapPicker action={action} ctx={ctx} />,
  withChildren: withHoldTapChildren,
});
