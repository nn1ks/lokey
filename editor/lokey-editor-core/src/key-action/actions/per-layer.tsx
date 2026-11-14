import * as capnp from "capnp-es";
import {
  KeyboardActionEntry,
  PerLayerAction,
} from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import { JSX } from "solid-js/jsx-runtime";
import { KeyAction } from "../action";
import {
  DecodeContext,
  EncodeContext,
  PickerContext,
  RemapLayersContext,
} from "../contexts";
import { createStore, SetStoreFunction } from "solid-js/store";
import { buildAction, buildActionEntry } from "../codec/build";
import { decodeAction } from "../codec/decode";
import { Labelled } from "../render";
import { Ellipsis } from "lucide-solid";
import { For } from "solid-js";

export class PerLayer implements KeyAction {
  static readonly actionId = ActionId.perLayer;
  readonly actionId = ActionId.perLayer;

  readonly actions: [number, KeyAction][];
  readonly setActions: SetStoreFunction<[number, KeyAction][]>;

  constructor(
    public actionEntryId: number | undefined,
    actions: KeyAction[],
    layers: number[],
  ) {
    if (actions.length !== layers.length) {
      throw new Error("actions and layers must have the same length");
    }
    const value: [number, KeyAction][] = [];
    for (let i = 0; i < actions.length; i++) {
      value.push([layers[i]!, actions[i]!]);
    }
    const [actionsStore, setActionsStore] = createStore(value);
    this.actions = actionsStore;
    this.setActions = setActionsStore;
  }

  encode(ctx: EncodeContext): KeyboardActionEntry {
    const encodedActionEntries = this.actions.map(([_, action]) =>
      action.encode(ctx),
    );
    const actionEntryIds = encodedActionEntries.map((entry) => entry.entryId);
    encodedActionEntries.forEach((action) => ctx.pushActionEntry(action));
    const layers = this.actions.map(([layer, _]) => layer);

    const action = buildAction(
      ActionId.perLayer,
      PerLayerAction,
      (perLayerAction) => {
        perLayerAction._initActionEntryIds(actionEntryIds.length);
        actionEntryIds.forEach((actionEntryId, index) =>
          perLayerAction.actionEntryIds.set(index, actionEntryId),
        );
        perLayerAction._initLayers(actionEntryIds.length);
        layers.forEach((layer, index) =>
          perLayerAction.layers.set(index, layer),
        );
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
  ): PerLayer {
    const message = new capnp.Message(actionPayload, false);
    const perLayer = message.getRoot(PerLayerAction);
    const childActions = perLayer.actionEntryIds.map((actionEntryId) =>
      decodeAction(actionEntryId, ctx.resolveEntry(actionEntryId), ctx),
    );
    return new PerLayer(actionEntryId, childActions, perLayer.layers);
  }

  renderButtonContent(_level: number): JSX.Element {
    return (
      <Labelled label="Per Layer">
        <Ellipsis />
      </Labelled>
    );
  }

  renderTooltip(): JSX.Element {
    const items = this.actions.map(([layer, action]) => (
      <div class="flex min-w-12 flex-col items-center gap-1">
        <div class="text-neutral-500 text-[0.65rem] font-semibold tracking-[0.08em] uppercase">
          {layer}
        </div>
        <div class="flex min-h-6 items-center justify-center">
          {action.renderTooltip()}
        </div>
      </div>
    ));
    return (
      <div class="flex items-stretch gap-2">
        <For each={items}>
          {(item, index) => {
            if (index() !== 0) {
              return (
                <>
                  <div class="w-px bg-neutral-600 dark:bg-neutral-400" />
                  {item}
                </>
              );
            } else {
              return item;
            }
          }}
        </For>
      </div>
    );
  }

  renderPicker(_ctx: PickerContext): JSX.Element {
    throw new Error(
      "PerLayer action does not implement the renderPicker method",
    );
  }

  remapLayers(ctx: RemapLayersContext): void {
    this.actions.forEach(([_, action]) => action.remapLayers(ctx));
  }

  getActionOfLayer(layer: number): KeyAction | undefined {
    return this.actions.find(([currentLayer, _]) => currentLayer == layer)?.[1];
  }

  setActionOfLayer(layer: number, action: KeyAction): void {
    const currentActionIndex = this.actions.findIndex(
      ([currentLayer, _]) => currentLayer == layer,
    );
    if (currentActionIndex === -1) {
      throw new Error(`PerLayer action does not have layer ${layer}`);
    }
    this.setActions(currentActionIndex, [layer, action]);
  }
}
