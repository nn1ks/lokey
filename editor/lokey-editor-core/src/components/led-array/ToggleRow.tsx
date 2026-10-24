import { Switch } from "@ark-ui/solid";
import type { Component } from "solid-js";

export type ToggleRowProps = {
  label: string;
  description: string;
  enabled: boolean;
  onCheckedChange?: (checked: boolean) => void;
};

export const ToggleRow: Component<ToggleRowProps> = (props) => {
  return (
    <Switch.Root
      checked={props.enabled}
      onCheckedChange={(details) => props.onCheckedChange?.(details.checked)}
      class="
        flex items-center justify-between gap-3 py-3 px-4 cursor-pointer select-none
        bg-neutral-150 dark:bg-neutral-900
      "
    >
      <Switch.Label class="flex min-w-0 flex-1 flex-col">
        <div>{props.label}</div>
        <div class="text-neutral-500">{props.description}</div>
      </Switch.Label>
      <Switch.Control
        class="
          inline-flex h-7 w-12 shrink-0 items-center rounded-full pl-1
          transition-[background-color,box-shadow] duration-150 ease-[ease]
          bg-neutral-300 dark:bg-neutral-700
          data-[state=checked]:bg-neutral-700 dark:data-[state=checked]:bg-neutral-300
          data-focus-visible:outline-2 data-focus-visible:outline-focused
          data-focus-visible:outline-offset-2
        "
      >
        <Switch.Thumb
          class="
            flex size-5 items-center justify-center rounded-full
            data-[state=checked]:translate-x-5
            transition-transform duration-100 ease-[ease]
            bg-neutral-50 dark:bg-neutral-950
          "
        />
      </Switch.Control>
      <Switch.HiddenInput />
    </Switch.Root>
  );
};

export default ToggleRow;
