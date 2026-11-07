import { Accordion } from "@ark-ui/solid";
import * as capnp from "capnp-es";
import { Ellipsis } from "lucide-solid";
import { createSignal } from "solid-js";
import type { Component, JSX } from "solid-js";
import { KeyboardAction, ToggleAction } from "lokey-editor-odyn/keyboard";
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
import { Labelled } from "../render";
import { decodeAction } from "../codec/decode";

export class Toggle<A extends KeyAction = KeyAction> implements KeyAction {
  static readonly actionId = ActionId.toggle;
  readonly actionId = ActionId.toggle;

  constructor(readonly action: A) {}

  encode(ctx: EncodeContext): KeyboardAction {
    const action = this.action.encode(ctx);
    const actionEntryId = ctx.pushAction(action);

    return buildAction(ActionId.toggle, ToggleAction, (toggleAction) => {
      toggleAction.actionEntryId = actionEntryId;
    });
  }

  static decode(payload: Uint8Array, ctx: DecodeContext): Toggle {
    const message = new capnp.Message(payload, false);
    const toggle = message.getRoot(ToggleAction);
    return new Toggle(
      decodeAction(ctx.resolveEntry(toggle.actionEntryId), ctx),
    );
  }

  renderButtonContent(level: number): JSX.Element {
    if (level > 0) return <Ellipsis />;

    return (
      <Labelled label="Toggle">
        {this.action.renderButtonContent(level + 1)}
      </Labelled>
    );
  }

  renderTooltip(): JSX.Element {
    return (
      <div class="flex flex-col items-center gap-1">
        <div class="text-neutral-500 text-[0.65rem] font-semibold tracking-[0.08em] uppercase">
          Toggle
        </div>
        <div class="flex min-h-6 items-center justify-center">
          {this.action.renderTooltip()}
        </div>
      </div>
    );
  }

  renderPicker(ctx: PickerContext): JSX.Element {
    return <TogglePicker action={this} ctx={ctx} />;
  }

  remapLayers(ctx: RemapLayersContext): boolean {
    return this.action.remapLayers(ctx);
  }
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
