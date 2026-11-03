import * as capnp from "capnp-es";
import { Layers2 } from "lucide-solid";
import { LayerAction } from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import type { KeyAction } from "../action";
import { buildAction, buildActionEntry } from "../codec/entries";
import { defineAction } from "../definition";
import { IconSubtitleCombination } from "../render";

export class Layer implements KeyAction {
  readonly actionId = ActionId.layer;

  constructor(readonly index: number) {}
}

export const layerActionDefinition = defineAction<Layer>({
  actionId: ActionId.layer,
  label: "Layer",
  encode: (action, ctx) => ({
    rootActionEntry: buildActionEntry(
      ctx.allocateEntryId(),
      buildAction(ActionId.layer, LayerAction, (layerAction) => {
        layerAction.layer = action.index;
      }),
    ),
    childActionEntries: [],
  }),
  decode: (payload) => {
    const message = new capnp.Message(payload, false);
    return new Layer(message.getRoot(LayerAction).layer);
  },
  render: (action) => {
    const layerName = `Layer ${action.index + 1}`;
    return <IconSubtitleCombination icon={<Layers2 />} subtitle={layerName} />;
  },
  renderTooltip: (action) => <>Layer {action.index + 1}</>,
  remap: (action, ctx) => {
    const index = ctx.remapLayer(action.index);
    return index === action.index ? action : new Layer(index);
  },
  children: () => [],
});
