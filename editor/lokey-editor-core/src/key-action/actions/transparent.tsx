import * as capnp from "capnp-es";
import { CircleChevronDown } from "lucide-solid";
import { KeyboardAction, TransparentAction } from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import type { JSX } from "solid-js";
import type { KeyAction } from "../action";
import { buildAction } from "../codec/build";
import {
  DecodeContext,
  EncodeContext,
  PickerContext,
  RemapLayersContext,
} from "../contexts";

export class Transparent implements KeyAction {
  static readonly actionId = ActionId.transparent;
  readonly actionId = ActionId.transparent;

  encode(_ctx: EncodeContext): KeyboardAction {
    return buildAction(ActionId.transparent, TransparentAction, () => {});
  }

  static decode(payload: Uint8Array, _ctx: DecodeContext): Transparent {
    const message = new capnp.Message(payload, false);
    message.getRoot(TransparentAction);
    return new Transparent();
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

  remapLayers(_ctx: RemapLayersContext): boolean {
    return false;
  }
}
