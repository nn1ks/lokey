import { Accordion } from "@ark-ui/solid";
import * as capnp from "capnp-es";
import { Ellipsis } from "lucide-solid";
import { createSignal } from "solid-js";
import type { Accessor, Component, JSX, Setter } from "solid-js";
import { KeyboardActionEntry, ToggleAction } from "lokey-editor-odyn/keyboard";
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
import { Labelled } from "../render";
import { decodeAction } from "../codec/decode";

export class Toggle implements KeyAction {
  static readonly actionId = ActionId.toggle;
  readonly actionId = ActionId.toggle;

  readonly action: Accessor<KeyAction>;
  readonly setAction: Setter<KeyAction>;

  constructor(
    public actionEntryId: number | undefined,
    action: KeyAction,
  ) {
    const [localAction, localSetAction] = createSignal(action);
    this.action = localAction;
    this.setAction = localSetAction;
  }

  encode(ctx: EncodeContext): KeyboardActionEntry {
    const childAction = this.action().encode(ctx);
    ctx.pushActionEntry(childAction);

    const action = buildAction(
      ActionId.toggle,
      ToggleAction,
      (toggleAction) => {
        toggleAction.actionEntryId = childAction.entryId;
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
  ): Toggle {
    const message = new capnp.Message(actionPayload, false);
    const toggle = message.getRoot(ToggleAction);
    return new Toggle(
      actionEntryId,
      decodeAction(
        toggle.actionEntryId,
        ctx.resolveEntry(toggle.actionEntryId),
        ctx,
      ),
    );
  }

  renderButtonContent(level: number): JSX.Element {
    if (level > 0) return <Ellipsis />;

    return (
      <Labelled label="Toggle">
        {this.action().renderButtonContent(level + 1)}
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
          {this.action().renderTooltip()}
        </div>
      </div>
    );
  }

  renderPicker(ctx: PickerContext): JSX.Element {
    return <TogglePicker action={this} ctx={ctx} />;
  }

  remapLayers(ctx: RemapLayersContext): void {
    this.action().remapLayers(ctx);
  }
}

const withToggleChildren = (action: Toggle, children: KeyAction[]): Toggle =>
  new Toggle(action.actionEntryId, children[0]!);

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
        onSelect={() => props.ctx.selectAction()}
      />
      <Accordion.ItemContent class="data-[state=closed]:hidden">
        <div class="flex flex-col gap-2 p-2 pt-0">
          <ActionPickerChildRow
            label="Action"
            action={props.action.action()}
            onSelect={(action) => {
              action.actionEntryId = props.action.action().actionEntryId;
              props.action.setAction(action);
              props.ctx.selectAction();
              props.ctx.addAction(action);
            }}
          />
        </div>
      </Accordion.ItemContent>
    </>
  );
};
