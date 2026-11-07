import { Select, createListCollection } from "@ark-ui/solid/select";
import * as capnp from "capnp-es";
import { ChevronsUpDownIcon, Layers2 } from "lucide-solid";
import { For, createMemo } from "solid-js";
import type { Component, JSX } from "solid-js";
import { Portal } from "solid-js/web";
import { KeyboardAction, LayerAction } from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import { ActionPickerHeader } from "../../components/keymap/ActionPickerHeader";
import { useKeyboardActions } from "../../contexts/KeyboardActionContext";
import type { KeyAction } from "../action";
import { buildAction } from "../codec/build";
import {
  DecodeContext,
  EncodeContext,
  PickerContext,
  RemapLayersContext,
} from "../contexts";
import { IconSubtitleCombination } from "../render";

export class Layer implements KeyAction {
  static readonly actionId = ActionId.layer;
  readonly actionId = ActionId.layer;

  constructor(public index: number) {}

  encode(_ctx: EncodeContext): KeyboardAction {
    return buildAction(ActionId.layer, LayerAction, (layerAction) => {
      layerAction.layer = this.index;
    });
  }

  static decode(payload: Uint8Array, _ctx: DecodeContext): Layer {
    const message = new capnp.Message(payload, false);
    return new Layer(message.getRoot(LayerAction).layer);
  }

  renderButtonContent(_level: number): JSX.Element {
    const layerName = `Layer ${this.index + 1}`;
    return <IconSubtitleCombination icon={<Layers2 />} subtitle={layerName} />;
  }

  renderTooltip(): JSX.Element {
    return <>Layer {this.index + 1}</>;
  }

  renderPicker(ctx: PickerContext): JSX.Element {
    return <LayerPicker action={this} ctx={ctx} />;
  }

  remapLayers(ctx: RemapLayersContext): boolean {
    const oldIndex = this.index;
    this.index = ctx.remapLayer(this.index);
    return oldIndex !== this.index;
  }
}

const LayerPicker: Component<{
  action: Layer;
  ctx: PickerContext;
}> = (props) => {
  const keyboardActions = useKeyboardActions();
  const collection = createMemo(() =>
    createListCollection({
      items: Array.from(
        { length: keyboardActions.layerCount() },
        (_, index) => ({
          value: String(index),
          label: `Layer ${index + 1}`,
        }),
      ),
    }),
  );

  return (
    <div class="flex flex-col">
      <ActionPickerHeader
        label="Layer"
        withAccordionTrigger={false}
        onSelect={() => props.ctx.onSelect(props.action, true)}
      />
      <Select.Root
        class="p-2 pt-0"
        collection={collection()}
        value={[String(props.action.index)]}
        onSelect={(details) => {
          const value = details.value[0];
          if (value !== undefined) {
            props.ctx.onSelect(new Layer(Number(value)), false);
          }
        }}
      >
        <Select.Control>
          <Select.Trigger
            class="
              flex w-full cursor-pointer items-center justify-between gap-2
              rounded-sm px-3 h-9 text-left text-sm
              text-neutral-900 bg-neutral-50 hover:bg-white
              dark:text-neutral-100 dark:bg-neutral-900 dark:hover:bg-neutral-950
            "
          >
            <Select.ValueText placeholder="Select layer" />
            <Select.Indicator class="flex justify-center [&_svg]:size-4">
              <ChevronsUpDownIcon />
            </Select.Indicator>
          </Select.Trigger>
        </Select.Control>
        <Portal>
          <Select.Positioner>
            <Select.Content
              class="
                z-1000 min-w-(--reference-width) overflow-hidden rounded-md border
                shadow-md
                border-neutral-300 bg-neutral-50 text-neutral-900
                dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-100
              "
            >
              <For each={collection().items}>
                {(item) => (
                  <Select.Item
                    class="
                      m-1 flex cursor-pointer items-center justify-between gap-4
                      rounded-sm p-2
                      data-highlighted:bg-neutral-200
                      dark:data-highlighted:bg-neutral-800
                    "
                    item={item}
                  >
                    <Select.ItemText>{item.label}</Select.ItemText>
                    <Select.ItemIndicator>✓</Select.ItemIndicator>
                  </Select.Item>
                )}
              </For>
            </Select.Content>
          </Select.Positioner>
        </Portal>
      </Select.Root>
    </div>
  );
};
