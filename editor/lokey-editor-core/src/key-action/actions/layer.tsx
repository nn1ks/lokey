import { Select, createListCollection } from "@ark-ui/solid/select";
import * as capnp from "capnp-es";
import { ChevronsUpDownIcon, Layers2 } from "lucide-solid";
import { For, createMemo, createSignal } from "solid-js";
import type { Accessor, Component, JSX, Setter } from "solid-js";
import { Portal } from "solid-js/web";
import { KeyboardActionEntry, LayerAction } from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import { ActionPickerHeader } from "../../components/keymap/ActionPickerHeader";
import type { KeyAction } from "../action";
import { buildAction, buildActionEntry } from "../codec/build";
import {
  DecodeContext,
  EncodeContext,
  PickerContext,
  RemapLayersContext,
} from "../contexts";
import { IconSubtitleCombination } from "../render";
import { useKeymapState } from "../../contexts";

export class Layer implements KeyAction {
  static readonly actionId = ActionId.layer;
  readonly actionId = ActionId.layer;

  readonly index: Accessor<number>;
  readonly setIndex: Setter<number>;

  constructor(
    public actionEntryId: number | undefined,
    index: number,
  ) {
    const [localIndex, localSetIndex] = createSignal(index);
    this.index = localIndex;
    this.setIndex = localSetIndex;
  }

  encode(ctx: EncodeContext): KeyboardActionEntry {
    const entryId = this.actionEntryId ?? ctx.nextActionEntryId();
    this.actionEntryId = entryId;
    const action = buildAction(ActionId.layer, LayerAction, (layerAction) => {
      layerAction.layer = this.index();
    });
    return buildActionEntry(entryId, action);
  }

  static decode(
    actionEntryId: number | undefined,
    actionPayload: Uint8Array,
    _ctx: DecodeContext,
  ): Layer {
    const message = new capnp.Message(actionPayload, false);
    return new Layer(actionEntryId, message.getRoot(LayerAction).layer);
  }

  renderButtonContent(_level: number): JSX.Element {
    const layerName = `Layer ${this.index() + 1}`;
    return <IconSubtitleCombination icon={<Layers2 />} subtitle={layerName} />;
  }

  renderTooltip(): JSX.Element {
    return <>Layer {this.index() + 1}</>;
  }

  renderPicker(ctx: PickerContext): JSX.Element {
    return <LayerPicker action={this} ctx={ctx} />;
  }

  remapLayers(ctx: RemapLayersContext): void {
    const oldIndex = this.index();
    this.setIndex(ctx.convertLayer(oldIndex));
  }
}

const LayerPicker: Component<{
  action: Layer;
  ctx: PickerContext;
}> = (props) => {
  const { layerCount } = useKeymapState();
  const collection = createMemo(() =>
    createListCollection({
      items: Array.from({ length: layerCount() }, (_, index) => ({
        value: String(index),
        label: `Layer ${index + 1}`,
      })),
    }),
  );

  return (
    <div class="flex flex-col">
      <ActionPickerHeader
        label="Layer"
        withAccordionTrigger={false}
        onSelect={() => props.ctx.selectAction()}
      />
      <Select.Root
        class="p-2 pt-0"
        collection={collection()}
        value={[String(props.action.index())]}
        onSelect={(details) => {
          const value = details.value[0];
          if (value !== undefined) {
            props.action.setIndex(Number(value));
            props.ctx.selectAction();
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
