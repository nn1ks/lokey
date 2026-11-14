import { Accordion } from "@ark-ui/solid";
import * as capnp from "capnp-es";
import { Ellipsis } from "lucide-solid";
import { createSignal } from "solid-js";
import type { Accessor, Component, JSX, Setter } from "solid-js";
import { KeyboardActionEntry, OrderedAction } from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import { ActionPickerChildRow } from "../../components/keymap/ActionPickerChildRow";
import { ActionPickerHeader } from "../../components/keymap/ActionPickerHeader";
import type { KeyAction } from "../action";
import { buildAction, buildActionEntry } from "../codec/build";
import {
  DecodeContext,
  EncodeContext,
  PickerContext,
  RemapLayersContext,
} from "../contexts";
import { Split } from "../render";
import { KeyCode } from "./key-code";
import { decodeAction } from "../codec/decode";
import { keyCodeValues, reverseKeyCodeValues } from "../key-code-value";

export class Ordered implements KeyAction {
  static readonly actionId = ActionId.ordered;
  readonly actionId = ActionId.ordered;

  readonly action1: Accessor<KeyAction>;
  readonly setAction1: Setter<KeyAction>;
  readonly action2: Accessor<KeyAction>;
  readonly setAction2: Setter<KeyAction>;

  constructor(
    public actionEntryId: number | undefined,
    action1: KeyAction,
    action2: KeyAction,
  ) {
    const [localAction1, localSetAction1] = createSignal(action1);
    this.action1 = localAction1;
    this.setAction1 = localSetAction1;
    const [localAction2, localSetAction2] = createSignal(action2);
    this.action2 = localAction2;
    this.setAction2 = localSetAction2;
  }

  encode(ctx: EncodeContext): KeyboardActionEntry {
    const action1Entry = this.action1().encode(ctx);
    const action2Entry = this.action2().encode(ctx);

    ctx.pushActionEntry(action1Entry);
    ctx.pushActionEntry(action2Entry);

    const action = buildAction(
      ActionId.ordered,
      OrderedAction,
      (orderedAction) => {
        orderedAction.action1EntryId = action1Entry.entryId;
        orderedAction.action2EntryId = action2Entry.entryId;
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
  ): Ordered | KeyCode {
    const message = new capnp.Message(actionPayload, false);
    const orderedAction = message.getRoot(OrderedAction);
    const action1 = decodeAction(
      orderedAction.action1EntryId,
      ctx.resolveEntry(orderedAction.action1EntryId),
      ctx,
    );
    const action2 = decodeAction(
      orderedAction.action2EntryId,
      ctx.resolveEntry(orderedAction.action2EntryId),
      ctx,
    );

    // A sequence that encodes a single character (e.g. German "=" as
    // Shift+0) collapses back into that key code.
    if (action1 instanceof KeyCode && action2 instanceof KeyCode) {
      const encoding1 = keyCodeValues(ctx.hostLayout).get(action1.value);
      const encoding2 = keyCodeValues(ctx.hostLayout).get(action2.value);
      if (encoding1?.kind === "single" && encoding2?.kind === "single") {
        const keyCodeValue = reverseKeyCodeValues(ctx.hostLayout).get(
          encoding1.rawKeyCode.code,
          encoding2.rawKeyCode.code,
        );
        if (keyCodeValue) return new KeyCode(actionEntryId, keyCodeValue);
      }
    }

    return new Ordered(actionEntryId, action1, action2);
  }

  renderButtonContent(level: number): JSX.Element {
    if (level > 0) return <Ellipsis />;

    return (
      <Split
        action1={this.action1().renderButtonContent(level + 1)}
        action2={this.action2().renderButtonContent(level + 1)}
      />
    );
  }

  renderTooltip(): JSX.Element {
    return (
      <div class="flex items-stretch gap-2">
        <div class="flex min-w-12 flex-col items-center gap-1">
          <div class="text-neutral-500 text-[0.65rem] font-semibold tracking-[0.08em] uppercase">
            1
          </div>
          <div class="flex min-h-6 items-center justify-center">
            {this.action1().renderTooltip()}
          </div>
        </div>
        <div class="w-px bg-neutral-600 dark:bg-neutral-400" />
        <div class="flex min-w-12 flex-col items-center gap-1">
          <div class="text-neutral-500 text-[0.65rem] font-semibold tracking-[0.08em] uppercase">
            2
          </div>
          <div class="flex min-h-6 items-center justify-center">
            {this.action2().renderTooltip()}
          </div>
        </div>
      </div>
    );
  }

  renderPicker(ctx: PickerContext): JSX.Element {
    return <OrderedPicker action={this} ctx={ctx} />;
  }

  remapLayers(ctx: RemapLayersContext): void {
    this.action1().remapLayers(ctx);
    this.action2().remapLayers(ctx);
  }
}

const OrderedPicker: Component<{
  action: Ordered;
  ctx: PickerContext;
}> = (props) => {
  return (
    <>
      <ActionPickerHeader
        label="Ordered"
        withAccordionTrigger={true}
        onSelect={() => props.ctx.selectAction()}
      />
      <Accordion.ItemContent class="data-[state=closed]:hidden">
        <div class="flex flex-col gap-2 p-2 pt-0">
          <ActionPickerChildRow
            label="1"
            action={props.action.action1()}
            onSelect={(action) => {
              action.actionEntryId = props.action.action1().actionEntryId;
              props.action.setAction1(action);
              props.ctx.selectAction();
              props.ctx.addAction(action);
            }}
          />
          <ActionPickerChildRow
            label="2"
            action={props.action.action2()}
            onSelect={(action) => {
              action.actionEntryId = props.action.action2().actionEntryId;
              props.action.setAction2(action);
              props.ctx.selectAction();
              props.ctx.addAction(action);
            }}
          />
        </div>
      </Accordion.ItemContent>
    </>
  );
};
