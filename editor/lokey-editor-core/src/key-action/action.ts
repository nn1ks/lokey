import { JSX } from "solid-js";
import {
  EncodeContext,
  PickerContext,
  DecodeContext,
  RemapLayersContext,
} from "./contexts";
import { KeyboardActionEntry } from "lokey-editor-odyn/keyboard";

export interface KeyAction {
  readonly actionId: number;
  actionEntryId: number | undefined;

  encode(ctx: EncodeContext): KeyboardActionEntry;

  renderButtonContent(level: number): JSX.Element;
  renderTooltip(): JSX.Element;
  renderPicker(ctx: PickerContext): JSX.Element;

  remapLayers(ctx: RemapLayersContext): void;
}

export interface KeyActionClass<A extends KeyAction = KeyAction> {
  readonly actionId: number;

  decode(
    actionEntryId: number | undefined,
    actionPaylaod: Uint8Array,
    ctx: DecodeContext,
  ): KeyAction;
}
