import * as capnp from "capnp-es";
import { CircleOff } from "lucide-solid";
import { KeyboardActionEntry, NoneAction } from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import type { JSX } from "solid-js";
import type { KeyAction } from "../action";
import { buildAction, buildActionEntry } from "../codec/build";
import {
  DecodeContext,
  EncodeContext,
  PickerContext,
  RemapLayersContext,
} from "../contexts";

export class None implements KeyAction {
  static readonly actionId = ActionId.none;
  readonly actionId = ActionId.none;

  constructor(public actionEntryId: number | undefined) {}

  encode(ctx: EncodeContext): KeyboardActionEntry {
    const entryId = this.actionEntryId ?? ctx.nextActionEntryId();
    this.actionEntryId = entryId;
    return buildActionEntry(
      entryId,
      buildAction(ActionId.none, NoneAction, () => {}),
    );
  }

  static decode(
    actionEntryId: number | undefined,
    actionPayload: Uint8Array,
    _ctx: DecodeContext,
  ): None {
    const message = new capnp.Message(actionPayload, false);
    message.getRoot(NoneAction);
    return new None(actionEntryId);
  }

  renderButtonContent(_level: number): JSX.Element {
    return <CircleOff />;
  }

  renderTooltip(): JSX.Element {
    return <>None</>;
  }

  renderPicker(_ctx: PickerContext): JSX.Element {
    return this.renderButtonContent(0);
  }

  remapLayers(_ctx: RemapLayersContext): void {}
}
