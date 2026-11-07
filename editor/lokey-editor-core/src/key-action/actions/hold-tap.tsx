import * as capnp from "capnp-es";
import { Ellipsis } from "lucide-solid";
import { createSignal } from "solid-js";
import type { Component, JSX } from "solid-js";
import { HoldTapAction, KeyboardAction } from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import { ActionPickerHeader } from "../../components/keymap/ActionPickerHeader";
import { ActionPickerChildRow } from "../../components/keymap/ActionPickerChildRow";
import { ActionPickerDurationRow } from "../../components/keymap/ActionPickerDurationRow";
import type { KeyAction } from "../action";
import { buildAction } from "../codec/build";
import { Labelled, Split } from "../render";
import { Accordion } from "@ark-ui/solid/accordion";
import {
  EncodeContext,
  DecodeContext,
  PickerContext,
  RemapLayersContext,
} from "../contexts";
import { decodeAction } from "../codec/decode";

export class HoldTap<
  H extends KeyAction = KeyAction,
  T extends KeyAction = KeyAction,
> implements KeyAction {
  static readonly actionId = ActionId.holdTap;
  readonly actionId = ActionId.holdTap;

  constructor(
    readonly holdAction: H,
    readonly tapAction: T,
    /** Tapping term in milliseconds. */
    readonly tappingTerm: number = 200,
  ) {}

  encode(ctx: EncodeContext): KeyboardAction {
    const holdAction = this.holdAction.encode(ctx);
    const tapAction = this.tapAction.encode(ctx);

    const holdActionEntryId = ctx.pushAction(holdAction);
    const tapActionEntryId = ctx.pushAction(tapAction);

    return buildAction(ActionId.holdTap, HoldTapAction, (holdTapAction) => {
      holdTapAction.holdActionEntryId = holdActionEntryId;
      holdTapAction.tapActionEntryId = tapActionEntryId;
      holdTapAction.tappingTerm = this.tappingTerm;
    });
  }

  static decode(payload: Uint8Array, ctx: DecodeContext): HoldTap {
    const message = new capnp.Message(payload, false);
    const holdTap = message.getRoot(HoldTapAction);
    return new HoldTap(
      decodeAction(ctx.resolveEntry(holdTap.holdActionEntryId), ctx),
      decodeAction(ctx.resolveEntry(holdTap.tapActionEntryId), ctx),
      holdTap.tappingTerm,
    );
  }

  renderButtonContent(level: number): JSX.Element {
    if (level > 0) return <Ellipsis />;

    return (
      <Split
        action1={
          <Labelled label="Hold">
            {this.holdAction.renderButtonContent(level + 1)}
          </Labelled>
        }
        action2={
          <Labelled label="Tap">
            {this.tapAction.renderButtonContent(level + 1)}
          </Labelled>
        }
      />
    );
  }

  renderTooltip(): JSX.Element {
    return (
      <div class="flex items-stretch gap-2">
        <div class="flex min-w-12 flex-col items-center gap-1">
          <div class="text-neutral-500 text-[0.65rem] font-semibold tracking-[0.08em] uppercase">
            Hold
          </div>
          <div class="flex min-h-6 items-center justify-center">
            {this.holdAction.renderTooltip()}
          </div>
        </div>
        <div class="w-px bg-neutral-600 dark:bg-neutral-400" />
        <div class="flex min-w-12 flex-col items-center gap-1">
          <div class="text-neutral-500 text-[0.65rem] font-semibold tracking-[0.08em] uppercase">
            Tap
          </div>
          <div class="flex min-h-6 items-center justify-center">
            {this.tapAction.renderTooltip()}
          </div>
        </div>
      </div>
    );
  }

  renderPicker(ctx: PickerContext): JSX.Element {
    return <HoldTapPicker action={this} ctx={ctx} />;
  }

  remapLayers(ctx: RemapLayersContext): boolean {
    const holdActionChanged = this.holdAction.remapLayers(ctx);
    const tapActionChanged = this.tapAction.remapLayers(ctx);
    return holdActionChanged || tapActionChanged;
  }
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
