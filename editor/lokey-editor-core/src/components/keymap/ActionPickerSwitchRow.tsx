import { Switch } from "@ark-ui/solid/switch";
import type { Component } from "solid-js";

export type ActionPickerSwitchRowProps = {
  label: string;
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
};

export const ActionPickerSwitchRow: Component<ActionPickerSwitchRowProps> = (
  props,
) => (
  <Switch.Root
    checked={props.checked}
    onCheckedChange={(details) => props.onCheckedChange(details.checked)}
    class="flex items-center justify-between rounded-sm px-3 h-9 text-left text-sm cursor-pointer
      bg-neutral-50 dark:bg-neutral-900 hover:bg-white dark:hover:bg-neutral-950"
  >
    <Switch.Label>{props.label}</Switch.Label>
    <Switch.Control
      class="
        relative inline-flex h-5 w-9 items-center rounded-full p-0.5
        transition-colors duration-100
        bg-neutral-300 data-[state=checked]:bg-neutral-900
        dark:bg-neutral-700 dark:data-[state=checked]:bg-neutral-100
      "
    >
      <Switch.Thumb
        class="
          inline-block size-4 rounded-full bg-white shadow-sm
          transition-transform duration-100
          data-[state=checked]:translate-x-4
          data-[state=checked]:bg-neutral-100
          dark:data-[state=checked]:bg-neutral-900
        "
      />
    </Switch.Control>
    <Switch.HiddenInput />
  </Switch.Root>
);

export default ActionPickerSwitchRow;
