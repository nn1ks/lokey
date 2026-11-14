import * as capnp from "capnp-es";
import { CircleChevronDown } from "lucide-solid";
import {
  KeyboardActionEntry,
  TransparentAction,
} from "lokey-editor-odyn/keyboard";
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

export class Transparent implements KeyAction {
  static readonly actionId = ActionId.transparent;
  readonly actionId = ActionId.transparent;

  constructor(public actionEntryId: number | undefined) {}

  encode(ctx: EncodeContext): KeyboardActionEntry {
    const action = buildAction(
      ActionId.transparent,
      TransparentAction,
      () => {},
    );

    const entryId = this.actionEntryId ?? ctx.nextActionEntryId();
    this.actionEntryId = entryId;

    return buildActionEntry(entryId, action);
  }

  static decode(
    actionEntryId: number | undefined,
    actionPayload: Uint8Array,
    _ctx: DecodeContext,
  ): Transparent {
    const message = new capnp.Message(actionPayload, false);
    message.getRoot(TransparentAction);
    return new Transparent(actionEntryId);
  }

  renderButtonContent(_level: number): JSX.Element {
    return <CircleChevronDown />;
  }

  renderTooltip(): JSX.Element {
    return <>Transparent</>;
  }

  renderPicker(_ctx: PickerContext): JSX.Element {
    return this.renderButtonContent(0);
  }

  remapLayers(_ctx: RemapLayersContext): void {}
}
