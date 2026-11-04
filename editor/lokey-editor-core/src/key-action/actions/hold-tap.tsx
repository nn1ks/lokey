import { NumberInput } from "@ark-ui/solid/number-input";
import * as capnp from "capnp-es";
import { Ellipsis } from "lucide-solid";
import { createSignal } from "solid-js";
import type { Component } from "solid-js";
import { HoldTapAction } from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import { ActionPickerHeader } from "../../components/keymap/ActionPickerHeader";
import { ActionPickerChildRow } from "../../components/keymap/ActionPickerChildRow";
import type { KeyAction } from "../action";
import { buildAction, buildActionEntry } from "../codec/entries";
import { defineAction } from "../definition";
import type { PickerContext } from "../definition";
import { Labelled, Split } from "../render";

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
    <div class="flex flex-col gap-2">
      <ActionPickerHeader
        label="Hold Tap"
        onSelect={() => props.ctx.onSelect(currentAction())}
      />
      <div class="flex items-center justify-between rounded-sm px-3 h-9 text-left text-sm bg-neutral-50 dark:bg-neutral-900">
        <span>Tapping Term</span>
        <NumberInput.Root
          value={String(currentAction().tappingTerm)}
          min={0}
          max={10000}
          step={50}
          onValueChange={(details) => {
            if (!Number.isNaN(details.valueAsNumber)) {
              setCurrentAction(
                withHoldTapTappingTerm(currentAction(), details.valueAsNumber),
              );
            }
          }}
          class="flex items-center gap-1"
        >
          <NumberInput.Control class="flex items-center h-6">
            <NumberInput.DecrementTrigger
              class="flex w-6 h-full cursor-pointer items-center justify-center rounded-l-full border-r
                bg-neutral-200 text-neutral-700 hover:bg-neutral-300 border-neutral-300
                dark:bg-neutral-800 dark:text-neutral-300 dark:hover:bg-neutral-700 dark:border-neutral-700"
            >
              −
            </NumberInput.DecrementTrigger>
            <NumberInput.Input
              class="w-14 h-full px-2 py-0.5 text-right text-sm
              bg-neutral-200 dark:bg-neutral-800"
            />
            <NumberInput.IncrementTrigger
              class="flex w-6 h-full cursor-pointer items-center justify-center rounded-r-full border-l
                bg-neutral-200 text-neutral-700 hover:bg-neutral-300 border-neutral-300
                dark:bg-neutral-800 dark:text-neutral-300 dark:hover:bg-neutral-700 dark:border-neutral-700"
            >
              +
            </NumberInput.IncrementTrigger>
          </NumberInput.Control>
          <span class="text-neutral-500 dark:text-neutral-400">ms</span>
        </NumberInput.Root>
      </div>
      <ActionPickerChildRow
        label="Hold"
        action={currentAction().holdAction}
        onSelect={(action) =>
          setCurrentAction(
            withHoldTapChildren(currentAction(), [
              action,
              currentAction().tapAction,
            ]),
          )
        }
      />
      <ActionPickerChildRow
        label="Tap"
        action={currentAction().tapAction}
        onSelect={(action) =>
          setCurrentAction(
            withHoldTapChildren(currentAction(), [
              currentAction().holdAction,
              action,
            ]),
          )
        }
      />
    </div>
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
