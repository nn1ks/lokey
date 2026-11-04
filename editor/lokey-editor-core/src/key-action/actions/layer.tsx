import { Select, createListCollection } from "@ark-ui/solid/select";
import * as capnp from "capnp-es";
import { ChevronsUpDownIcon, Layers2 } from "lucide-solid";
import { For, createMemo } from "solid-js";
import type { Component } from "solid-js";
import { Portal } from "solid-js/web";
import { LayerAction } from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import { ActionPickerHeader } from "../../components/keymap/ActionPickerHeader";
import { useKeyboardActions } from "../../contexts/KeyboardActionContext";
import type { KeyAction } from "../action";
import { buildAction, buildActionEntry } from "../codec/entries";
import { defineAction } from "../definition";
import type { PickerContext } from "../definition";
import { IconSubtitleCombination } from "../render";

export class Layer implements KeyAction {
  readonly actionId = ActionId.layer;

  constructor(readonly index: number) {}
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
    <div class="flex flex-col gap-2">
      <ActionPickerHeader
        label="Layer"
        onSelect={() => props.ctx.onSelect(props.action, true)}
      />
      <Select.Root
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
  renderPicker: (action, ctx) => <LayerPicker action={action} ctx={ctx} />,
});
