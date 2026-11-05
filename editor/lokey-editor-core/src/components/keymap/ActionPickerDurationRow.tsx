import { NumberInput } from "@ark-ui/solid";
import { Component } from "solid-js";

export type ActionPickerDurationRowProps = {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  unit: string;
  onValueCommit: () => void;
  onValueChange: (value: number) => void;
};

export const ActionPickerDurationRow: Component<
  ActionPickerDurationRowProps
> = (props) => {
  return (
    <div class="flex items-center justify-between rounded-sm px-3 h-9 text-left text-sm bg-neutral-50 dark:bg-neutral-900">
      <span>{props.label}</span>
      <NumberInput.Root
        value={String(props.value)}
        min={props.min}
        max={props.max}
        step={props.step}
        onValueCommit={props.onValueCommit}
        onValueChange={(details) => {
          if (!Number.isNaN(details.valueAsNumber)) {
            props.onValueChange(details.valueAsNumber);
          }
        }}
        class="flex items-center gap-1"
      >
        <NumberInput.Control class="flex items-center h-6">
          <NumberInput.DecrementTrigger
            class="flex w-6 h-full cursor-pointer items-center justify-center rounded-l-full border-r
              bg-neutral-200 text-neutral-700 hover:bg-neutral-300 border-neutral-300
              dark:bg-neutral-800 dark:text-neutral-300 dark:hover:bg-neutral-700 dark:border-neutral-700"
            onClick={props.onValueCommit}
          >
            −
          </NumberInput.DecrementTrigger>
          <NumberInput.Input
            class="w-14 h-full px-2 py-0.5 text-right text-sm
              bg-neutral-200 dark:bg-neutral-800"
          />
          <NumberInput.IncrementTrigger
            class="flex w-6 h-full cursor-pointer items-center justify-center rounded-r-full border-l
              bg-neutral-200 text-neutral-700 hover:bg-neutral-300 border-neutral-300
              dark:bg-neutral-800 dark:text-neutral-300 dark:hover:bg-neutral-700 dark:border-neutral-700"
            onClick={props.onValueCommit}
          >
            +
          </NumberInput.IncrementTrigger>
        </NumberInput.Control>
        <span class="text-neutral-500 dark:text-neutral-400">{props.unit}</span>
      </NumberInput.Root>
    </div>
  );
};
