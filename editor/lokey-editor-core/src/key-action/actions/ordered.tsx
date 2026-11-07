import { Accordion } from "@ark-ui/solid";
import * as capnp from "capnp-es";
import { Ellipsis } from "lucide-solid";
import { createSignal } from "solid-js";
import type { Component, JSX } from "solid-js";
import { KeyboardAction, OrderedAction } from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import { ActionPickerChildRow } from "../../components/keymap/ActionPickerChildRow";
import { ActionPickerHeader } from "../../components/keymap/ActionPickerHeader";
import type { KeyAction } from "../action";
import { buildAction } from "../codec/build";
import {
  DecodeContext,
  EncodeContext,
  PickerContext,
  RemapLayersContext,
} from "../contexts";
import { Split } from "../render";
import { KeyCode, keyCodesForLayout, reverseKeyCodes } from "./key-code";
import { decodeAction } from "../codec/decode";

export class Ordered<
  A1 extends KeyAction = KeyAction,
  A2 extends KeyAction = KeyAction,
> implements KeyAction {
  static readonly actionId = ActionId.ordered;
  readonly actionId = ActionId.ordered;

  constructor(
    readonly action1: A1,
    readonly action2: A2,
  ) {}

  encode(ctx: EncodeContext): KeyboardAction {
    const action1 = this.action1.encode(ctx);
    const action2 = this.action2.encode(ctx);

    const action1EntryId = ctx.pushAction(action1);
    const action2EntryId = ctx.pushAction(action2);

    return buildAction(ActionId.ordered, OrderedAction, (orderedAction) => {
      orderedAction.action1EntryId = action1EntryId;
      orderedAction.action2EntryId = action2EntryId;
    });
  }

  static decode(payload: Uint8Array, ctx: DecodeContext): Ordered | KeyCode {
    const message = new capnp.Message(payload, false);
    const orderedAction = message.getRoot(OrderedAction);
    const action1 = decodeAction(
      ctx.resolveEntry(orderedAction.action1EntryId),
      ctx,
    );
    const action2 = decodeAction(
      ctx.resolveEntry(orderedAction.action2EntryId),
      ctx,
    );

    // A sequence that encodes a single character (e.g. German "=" as
    // Shift+0) collapses back into that key code.
    if (action1 instanceof KeyCode && action2 instanceof KeyCode) {
      const encoding1 = keyCodesForLayout(ctx.hostLayout).get(action1);
      const encoding2 = keyCodesForLayout(ctx.hostLayout).get(action2);
      if (encoding1?.kind === "single" && encoding2?.kind === "single") {
        const keyCode = reverseKeyCodes(ctx.hostLayout).sequence.get(
          `${encoding1.rawKeyCode.code}:${encoding2.rawKeyCode.code}`,
        );
        if (keyCode) return keyCode;
      }
    }

    return new Ordered(action1, action2);
  }

  renderButtonContent(level: number): JSX.Element {
    if (level > 0) return <Ellipsis />;

    return (
      <Split
        action1={this.action1.renderButtonContent(level + 1)}
        action2={this.action2.renderButtonContent(level + 1)}
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
            {this.action1.renderTooltip()}
          </div>
        </div>
        <div class="w-px bg-neutral-600 dark:bg-neutral-400" />
        <div class="flex min-w-12 flex-col items-center gap-1">
          <div class="text-neutral-500 text-[0.65rem] font-semibold tracking-[0.08em] uppercase">
            2
          </div>
          <div class="flex min-h-6 items-center justify-center">
            {this.action2.renderTooltip()}
          </div>
        </div>
      </div>
    );
  }

  renderPicker(ctx: PickerContext): JSX.Element {
    return <OrderedPicker action={this} ctx={ctx} />;
  }

  remapLayers(ctx: RemapLayersContext): boolean {
    const action1Changed = this.action1.remapLayers(ctx);
    const action2Changed = this.action2.remapLayers(ctx);
    return action1Changed || action2Changed;
  }
}

const withOrderedChildren = (action: Ordered, children: KeyAction[]): Ordered =>
  new Ordered(children[0]!, children[1]!);

const OrderedPicker: Component<{
  action: Ordered;
  ctx: PickerContext;
}> = (props) => {
  const [currentAction, setCurrentAction] = createSignal(props.action);

  return (
    <>
      <ActionPickerHeader
        label="Ordered"
        withAccordionTrigger={true}
        onSelect={() => props.ctx.onSelect(currentAction(), true)}
      />
      <Accordion.ItemContent class="data-[state=closed]:hidden">
        <div class="flex flex-col gap-2 p-2 pt-0">
          <ActionPickerChildRow
            label="1"
            action={currentAction().action1}
            onSelect={(action) => {
              const children = [
                currentAction().action1,
                currentAction().action2,
              ];
              children[0] = action;
              setCurrentAction(withOrderedChildren(currentAction(), children));
              props.ctx.onSelect(currentAction(), false);
            }}
          />
          <ActionPickerChildRow
            label="2"
            action={currentAction().action2}
            onSelect={(action) => {
              const children = [
                currentAction().action1,
                currentAction().action2,
              ];
              children[1] = action;
              setCurrentAction(withOrderedChildren(currentAction(), children));
              props.ctx.onSelect(currentAction(), false);
            }}
          />
        </div>
      </Accordion.ItemContent>
    </>
  );
};
