import { Accordion } from "@ark-ui/solid";
import * as capnp from "capnp-es";
import { Ellipsis } from "lucide-solid";
import { createSignal } from "solid-js";
import type { Component, JSX } from "solid-js";
import { KeyboardAction, UnorderedAction } from "lokey-editor-odyn/keyboard";
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
import { decodeAction } from "../codec/decode";

export class Unordered<
  A1 extends KeyAction = KeyAction,
  A2 extends KeyAction = KeyAction,
> implements KeyAction {
  static readonly actionId = ActionId.unordered;
  readonly actionId = ActionId.unordered;

  constructor(
    readonly action1: A1,
    readonly action2: A2,
  ) {}

  encode(ctx: EncodeContext): KeyboardAction {
    const action1 = this.action1.encode(ctx);
    const action2 = this.action2.encode(ctx);

    const action1EntryId = ctx.pushAction(action1);
    const action2EntryId = ctx.pushAction(action2);

    return buildAction(
      ActionId.unordered,
      UnorderedAction,
      (unorderedAction) => {
        unorderedAction.action1EntryId = action1EntryId;
        unorderedAction.action2EntryId = action2EntryId;
      },
    );
  }

  static decode(payload: Uint8Array, ctx: DecodeContext): Unordered {
    const message = new capnp.Message(payload, false);
    const unorderedAction = message.getRoot(UnorderedAction);
    return new Unordered(
      decodeAction(ctx.resolveEntry(unorderedAction.action1EntryId), ctx),
      decodeAction(ctx.resolveEntry(unorderedAction.action2EntryId), ctx),
    );
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
    return <UnorderedPicker action={this} ctx={ctx} />;
  }

  remapLayers(ctx: RemapLayersContext): boolean {
    const action1Changed = this.action1.remapLayers(ctx);
    const action2Changed = this.action2.remapLayers(ctx);
    return action1Changed || action2Changed;
  }
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
    <>
      <ActionPickerHeader
        label="Unordered"
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
              setCurrentAction(
                withUnorderedChildren(currentAction(), children),
              );
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
              setCurrentAction(
                withUnorderedChildren(currentAction(), children),
              );
              props.ctx.onSelect(currentAction(), false);
            }}
          />
        </div>
      </Accordion.ItemContent>
    </>
  );
};
