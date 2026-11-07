import * as capnp from "capnp-es";
import { CircleOff } from "lucide-solid";
import { KeyboardAction, NoneAction } from "lokey-editor-odyn/keyboard";
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

export class None implements KeyAction {
  static readonly actionId = ActionId.none;
  readonly actionId = ActionId.none;

  encode(_ctx: EncodeContext): KeyboardAction {
    return buildAction(ActionId.none, NoneAction, () => {});
  }

  static decode(payload: Uint8Array, _ctx: DecodeContext): None {
    const message = new capnp.Message(payload, false);
    message.getRoot(NoneAction);
    return new None();
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

  remapLayers(_ctx: RemapLayersContext): boolean {
    return false;
  }
}
