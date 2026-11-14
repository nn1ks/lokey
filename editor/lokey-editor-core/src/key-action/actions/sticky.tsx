import { Accordion } from "@ark-ui/solid";
import * as capnp from "capnp-es";
import { Ellipsis } from "lucide-solid";
import { createSignal } from "solid-js";
import type { Accessor, Component, JSX, Setter } from "solid-js";
import { KeyboardActionEntry, StickyAction } from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import { ActionPickerChildRow } from "../../components/keymap/ActionPickerChildRow";
import { ActionPickerDurationRow } from "../../components/keymap/ActionPickerDurationRow";
import { ActionPickerHeader } from "../../components/keymap/ActionPickerHeader";
import { ActionPickerSwitchRow } from "../../components/keymap/ActionPickerSwitchRow";
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

export class Sticky implements KeyAction {
  static readonly actionId = ActionId.sticky;
  readonly actionId = ActionId.sticky;

  readonly action: Accessor<KeyAction>;
  readonly setAction: Setter<KeyAction>;
  readonly timeout: Accessor<number>;
  readonly setTimeout: Setter<number>;
  readonly ignoreModifiers: Accessor<boolean>;
  readonly setIgnoreModifiers: Setter<boolean>;

  constructor(
    public actionEntryId: number | undefined,
    action: KeyAction,
    /** Timeout in milliseconds. */
    timeout: number = 1000,
    ignoreModifiers: boolean = true,
  ) {
    const [localAction, localSetAction] = createSignal(action);
    this.action = localAction;
    this.setAction = localSetAction;
    const [localTimeout, localSetTimeout] = createSignal(timeout);
    this.timeout = localTimeout;
    this.setTimeout = localSetTimeout;
    const [localIgnoreModifiers, localSetIgnoreModifiers] =
      createSignal(ignoreModifiers);
    this.ignoreModifiers = localIgnoreModifiers;
    this.setIgnoreModifiers = localSetIgnoreModifiers;
  }

  encode(ctx: EncodeContext): KeyboardActionEntry {
    const childAction = this.action().encode(ctx);
    ctx.pushActionEntry(childAction);

    const action = buildAction(
      ActionId.sticky,
      StickyAction,
      (stickyAction) => {
        stickyAction.actionEntryId = childAction.entryId;
        stickyAction.timeout = this.timeout();
        stickyAction.ignoreModifiers = this.ignoreModifiers();
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
  ): Sticky {
    const message = new capnp.Message(actionPayload, false);
    const sticky = message.getRoot(StickyAction);
    return new Sticky(
      actionEntryId,
      decodeAction(
        sticky.actionEntryId,
        ctx.resolveEntry(sticky.actionEntryId),
        ctx,
      ),
      sticky.timeout,
      sticky.ignoreModifiers,
    );
  }

  renderButtonContent(level: number): JSX.Element {
    if (level > 0) return <Ellipsis />;

    return (
      <Labelled label="Sticky">
        {this.action().renderButtonContent(level + 1)}
      </Labelled>
    );
  }

  renderTooltip(): JSX.Element {
    return (
      <div class="flex flex-col items-center gap-1">
        <div class="text-neutral-500 text-[0.65rem] font-semibold tracking-[0.08em] uppercase">
          Sticky
        </div>
        <div class="flex min-h-6 items-center justify-center">
          {this.action().renderTooltip()}
        </div>
      </div>
    );
  }

  renderPicker(ctx: PickerContext): JSX.Element {
    return <StickyPicker action={this} ctx={ctx} />;
  }

  remapLayers(ctx: RemapLayersContext): void {
    this.action().remapLayers(ctx);
  }
}

const StickyPicker: Component<{
  action: Sticky;
  ctx: PickerContext;
}> = (props) => {
  return (
    <>
      <ActionPickerHeader
        label="Sticky"
        withAccordionTrigger={true}
        onSelect={() => props.ctx.selectAction()}
      />
      <Accordion.ItemContent class="data-[state=closed]:hidden">
        <div class="flex flex-col gap-2 p-2 pt-0">
          <ActionPickerDurationRow
            label="Timeout"
            value={props.action.timeout()}
            min={0}
            max={10000}
            step={50}
            unit="ms"
            onValueCommit={() => props.ctx.selectAction()}
            onValueChange={(timeout) => props.action.setTimeout(timeout)}
          />
          <ActionPickerSwitchRow
            label="Ignore Modifiers"
            checked={props.action.ignoreModifiers()}
            onCheckedChange={(ignoreModifiers) => {
              props.action.setIgnoreModifiers(ignoreModifiers);
              props.ctx.selectAction();
            }}
          />
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
