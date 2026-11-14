import * as capnp from "capnp-es";
import { Ellipsis } from "lucide-solid";
import { createSignal } from "solid-js";
import type { Accessor, Component, JSX, Setter } from "solid-js";
import { HoldTapAction, KeyboardActionEntry } from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import { ActionPickerHeader } from "../../components/keymap/ActionPickerHeader";
import { ActionPickerChildRow } from "../../components/keymap/ActionPickerChildRow";
import { ActionPickerDurationRow } from "../../components/keymap/ActionPickerDurationRow";
import type { KeyAction } from "../action";
import { buildAction, buildActionEntry } from "../codec/build";
import { Labelled, Split } from "../render";
import { Accordion } from "@ark-ui/solid/accordion";
import {
  EncodeContext,
  DecodeContext,
  PickerContext,
  RemapLayersContext,
} from "../contexts";
import { decodeAction } from "../codec/decode";

export class HoldTap implements KeyAction {
  static readonly actionId = ActionId.holdTap;
  readonly actionId = ActionId.holdTap;

  readonly holdAction: Accessor<KeyAction>;
  readonly setHoldAction: Setter<KeyAction>;
  readonly tapAction: Accessor<KeyAction>;
  readonly setTapAction: Setter<KeyAction>;
  readonly tappingTerm: Accessor<number>;
  readonly setTappingTerm: Setter<number>;

  constructor(
    public actionEntryId: number | undefined,
    holdAction: KeyAction,
    tapAction: KeyAction,
    /** Tapping term in milliseconds. */
    tappingTerm: number = 200,
  ) {
    const [localHoldAction, localSetHoldAction] = createSignal(holdAction);
    this.holdAction = localHoldAction;
    this.setHoldAction = localSetHoldAction;
    const [localTapAction, localSetTapAction] = createSignal(tapAction);
    this.tapAction = localTapAction;
    this.setTapAction = localSetTapAction;
    const [localTappingTerm, localSetTappingTerm] = createSignal(tappingTerm);
    this.tappingTerm = localTappingTerm;
    this.setTappingTerm = localSetTappingTerm;
  }

  encode(ctx: EncodeContext): KeyboardActionEntry {
    const holdActionEntry = this.holdAction().encode(ctx);
    const tapActionEntry = this.tapAction().encode(ctx);

    ctx.pushActionEntry(holdActionEntry);
    ctx.pushActionEntry(tapActionEntry);

    const action = buildAction(
      ActionId.holdTap,
      HoldTapAction,
      (holdTapAction) => {
        holdTapAction.holdActionEntryId = holdActionEntry.entryId;
        holdTapAction.tapActionEntryId = tapActionEntry.entryId;
        holdTapAction.tappingTerm = this.tappingTerm();
      },
    );

    const entryId = this.actionEntryId ?? ctx.nextActionEntryId();
    this.actionEntryId = entryId;

    return buildActionEntry(entryId, action);
  }

  static decode(
    actionEntryId: number | undefined,
    actionPayload: Uint8Array,
    ctx: DecodeContext,
  ): HoldTap {
    const message = new capnp.Message(actionPayload, false);
    const holdTap = message.getRoot(HoldTapAction);
    return new HoldTap(
      actionEntryId,
      decodeAction(
        holdTap.holdActionEntryId,
        ctx.resolveEntry(holdTap.holdActionEntryId),
        ctx,
      ),
      decodeAction(
        holdTap.tapActionEntryId,
        ctx.resolveEntry(holdTap.tapActionEntryId),
        ctx,
      ),
      holdTap.tappingTerm,
    );
  }

  renderButtonContent(level: number): JSX.Element {
    if (level > 0) return <Ellipsis />;

    return (
      <Split
        action1={
          <Labelled label="Hold">
            {this.holdAction().renderButtonContent(level + 1)}
          </Labelled>
        }
        action2={
          <Labelled label="Tap">
            {this.tapAction().renderButtonContent(level + 1)}
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
            {this.holdAction().renderTooltip()}
          </div>
        </div>
        <div class="w-px bg-neutral-600 dark:bg-neutral-400" />
        <div class="flex min-w-12 flex-col items-center gap-1">
          <div class="text-neutral-500 text-[0.65rem] font-semibold tracking-[0.08em] uppercase">
            Tap
          </div>
          <div class="flex min-h-6 items-center justify-center">
            {this.tapAction().renderTooltip()}
          </div>
        </div>
      </div>
    );
  }

  renderPicker(ctx: PickerContext): JSX.Element {
    return <HoldTapPicker action={this} ctx={ctx} />;
  }

  remapLayers(ctx: RemapLayersContext): void {
    this.holdAction().remapLayers(ctx);
    this.tapAction().remapLayers(ctx);
  }
}

const HoldTapPicker: Component<{
  action: HoldTap;
  ctx: PickerContext;
}> = (props) => {
  return (
    <>
      <ActionPickerHeader
        label="Hold Tap"
        withAccordionTrigger={true}
        onSelect={() => {
          props.ctx.selectAction();
          props.ctx.closePicker();
        }}
      />
      <Accordion.ItemContent class="data-[state=closed]:hidden">
        <div class="flex flex-col gap-2 p-2 pt-0">
          <ActionPickerDurationRow
            label="Tapping Term"
            value={props.action.tappingTerm()}
            min={0}
            max={10000}
            step={50}
            unit="ms"
            onValueCommit={() => props.ctx.selectAction()}
            onValueChange={(value) => props.action.setTappingTerm(value)}
          />
          <ActionPickerChildRow
            label="Hold"
            action={props.action.holdAction()}
            onSelect={(action) => {
              action.actionEntryId = props.action.holdAction().actionEntryId;
              props.action.setHoldAction(action);
              props.ctx.selectAction();
              props.ctx.addAction(action);
            }}
          />
          <ActionPickerChildRow
            label="Tap"
            action={props.action.tapAction()}
            onSelect={(action) => {
              action.actionEntryId = props.action.tapAction().actionEntryId;
              props.action.setTapAction(action);
              props.ctx.selectAction();
              props.ctx.addAction(action);
            }}
          />
        </div>
      </Accordion.ItemContent>
    </>
  );
};
