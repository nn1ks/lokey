import { Tooltip } from "@ark-ui/solid";
import { ChevronLeft, ChevronRight, Plus } from "lucide-solid";
import { For } from "solid-js";
import type { Component, JSX } from "solid-js";
import LayerSelectorButton from "./LayerSelectorButton";
import StyledTooltipContent from "../StyledTooltipContent";

export type LayerSelectorBarProps = {
  /** Number of layers displayed by the selector. */
  layerCount: number;

  /** Index of the currently selected layer. */
  selected?: number;

  /** Called with the selected layer index when moving it to the left. */
  onMoveLeft?: (index: number) => void;

  /** Called with the selected layer index when moving it to the right. */
  onMoveRight?: (index: number) => void;

  /** Called when a new layer should be created. */
  onCreate?: () => void;

  /** Called when a layer is selected. */
  onSelect?: (index: number) => void;

  /** Called when a layer is removed. */
  onRemove?: (index: number) => void;
};

type LayerSelectorActionButtonProps = {
  label: string;
  tooltip: string;
  icon: JSX.Element;
  disabled?: boolean;
  onClick?: JSX.EventHandler<HTMLButtonElement, MouseEvent>;
};

const LayerSelectorActionButton: Component<LayerSelectorActionButtonProps> = (
  props,
) => (
  <Tooltip.Root positioning={{ placement: "bottom" }}>
    <Tooltip.Trigger
      class="
        inline-flex flex-none size-8 items-center justify-center rounded-full border-0 p-0
        cursor-pointer
        bg-neutral-300 text-neutral-700
        dark:bg-neutral-800 dark:text-neutral-300
        enabled:hover:bg-neutral-400 enabled:hover:text-neutral-800
        dark:enabled:hover:bg-neutral-700 dark:enabled:hover:text-neutral-200
        disabled:cursor-not-allowed disabled:opacity-[0.45]
        focus-visible:outline-2 focus-visible:outline-focused
        focus-visible:outline-offset-2
      "
      type="button"
      aria-label={props.label}
      disabled={props.disabled}
      onClick={props.onClick}
    >
      {props.icon}
    </Tooltip.Trigger>
    <Tooltip.Positioner>
      <StyledTooltipContent>{props.tooltip}</StyledTooltipContent>
    </Tooltip.Positioner>
  </Tooltip.Root>
);

export const LayerSelectorBar: Component<LayerSelectorBarProps> = (props) => {
  const layers = () =>
    Array.from({ length: props.layerCount }, (_, index) => index);
  const hasSelectedLayer = () => props.selected !== undefined;

  return (
    <div class="flex w-full min-w-0 items-center gap-1.5 overflow-x-scroll px-3 py-2">
      <LayerSelectorActionButton
        label="Move selected layer left"
        tooltip="Move layer up"
        icon={<ChevronLeft size={18} strokeWidth={2} />}
        disabled={!hasSelectedLayer()}
        onClick={() => {
          if (props.selected !== undefined) props.onMoveLeft?.(props.selected);
        }}
      />

      <LayerSelectorActionButton
        label="Move selected layer right"
        tooltip="Move layer down"
        icon={<ChevronRight size={18} strokeWidth={2} />}
        disabled={!hasSelectedLayer()}
        onClick={() => {
          if (props.selected !== undefined) props.onMoveRight?.(props.selected);
        }}
      />

      <div
        class="mx-0.5 h-6 w-px flex-none bg-neutral-300 dark:bg-neutral-800"
        role="separator"
        aria-orientation="vertical"
      />

      <div
        class="flex flex-none items-center gap-1.5 overflow-x-scroll"
        role="list"
        aria-label="Layers"
      >
        <For each={layers()}>
          {(index) => (
            <LayerSelectorButton
              layerIndex={index}
              selected={props.selected}
              onSelect={props.onSelect}
              onRemove={props.onRemove}
            />
          )}
        </For>

        <LayerSelectorActionButton
          label="Add new layer"
          tooltip="Add new layer"
          icon={<Plus size={18} strokeWidth={2} />}
          onClick={() => props.onCreate?.()}
        />
      </div>
    </div>
  );
};

export default LayerSelectorBar;
