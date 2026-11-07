import { KeyboardAction } from "lokey-editor-odyn/keyboard";
import type { JSX } from "solid-js";
import type { KeyAction } from "../action";
import { buildRawAction } from "../codec/build";
import type {
  EncodeContext,
  PickerContext,
  RemapLayersContext,
} from "../contexts";
import { CircleQuestionMark } from "lucide-solid";

/**
 * An action whose definition is not registered in the editor. The raw payload
 * is preserved so the action can be written back unchanged.
 */
export class UnknownAction implements KeyAction {
  constructor(
    readonly actionId: number,
    readonly payload: Uint8Array,
  ) {}

  encode(_ctx: EncodeContext): KeyboardAction {
    return buildRawAction(this.actionId, this.payload);
  }

  renderButtonContent(_level: number): JSX.Element {
    return <CircleQuestionMark />;
  }

  renderTooltip(): JSX.Element {
    return <>Unsupported action ({this.actionId})</>;
  }

  renderPicker(_ctx: PickerContext): JSX.Element {
    return this.renderButtonContent(0);
  }

  remapLayers(_ctx: RemapLayersContext): boolean {
    return false;
  }
}
