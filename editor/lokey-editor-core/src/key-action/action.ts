import { JSX } from "solid-js";
import {
  EncodeContext,
  PickerContext,
  DecodeContext,
  RemapLayersContext,
} from "./contexts";
import { KeyboardAction } from "lokey-editor-odyn/keyboard";

export interface KeyAction {
  readonly actionId: number;

  encode(ctx: EncodeContext): KeyboardAction;

  renderButtonContent(level: number): JSX.Element;
  renderTooltip(): JSX.Element;
  renderPicker(ctx: PickerContext): JSX.Element;

  remapLayers(ctx: RemapLayersContext): boolean;
}

export interface KeyActionClass<A extends KeyAction = KeyAction> {
  readonly actionId: number;

  decode(paylaod: Uint8Array, ctx: DecodeContext): KeyAction;
}

/**
 * The actions of a single key: either one action shared across all layers, or
 * one action per layer.
 */
export type RootKeyAction =
  | { kind: "single"; action: KeyAction }
  | { kind: "perLayer"; actions: KeyAction[] };
