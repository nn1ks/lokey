import type { Component } from "solid-js";

export type ActionPickerHeaderProps = {
  label: string;
  onSelect: () => void;
};

export const ActionPickerHeader: Component<ActionPickerHeaderProps> = (
  props,
) => (
  <div class="text-left text-sm font-medium flex gap-2 pb-1 items-center justify-between">
    <div>{props.label}</div>
    <button
      type="button"
      class="px-2 py-0.5 rounded-full hover:cursor-pointer
        bg-neutral-350 hover:bg-neutral-400 dark:bg-neutral-700 dark:hover:bg-neutral-600"
      onClick={() => props.onSelect()}
    >
      Select
    </button>
  </div>
);

export default ActionPickerHeader;
