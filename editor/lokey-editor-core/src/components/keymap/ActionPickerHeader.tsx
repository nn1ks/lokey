import { Accordion } from "@ark-ui/solid";
import { ChevronRight } from "lucide-solid";
import type { Component } from "solid-js";
import styles from "./ActionPickerHeader.module.css";

export type ActionPickerHeaderProps = {
  label: string;
  withAccordionTrigger: boolean;
  onSelect: () => void;
};

export const ActionPickerHeader: Component<ActionPickerHeaderProps> = (
  props,
) => {
  const selectButton = (
    <button
      type="button"
      class="px-2 py-0.5 rounded-full hover:cursor-pointer
        bg-neutral-350 hover:bg-neutral-400 dark:bg-neutral-700 dark:hover:bg-neutral-600"
      onClick={() => props.onSelect()}
    >
      Select
    </button>
  );

  if (props.withAccordionTrigger) {
    return (
      <Accordion.ItemTrigger
        class={`${styles.accordionItemTrigger} flex items-center gap-1 p-2 w-full hover:cursor-pointer`}
      >
        <Accordion.ItemIndicator>
          <ChevronRight
            size="20"
            class="-mt-px text-neutral-500 dark:text-neutral-400 transition-rotate duration-50"
          />
        </Accordion.ItemIndicator>
        <div class="w-full text-left text-sm font-medium flex gap-2 items-center justify-between">
          <div>{props.label}</div>
          {selectButton}
        </div>
      </Accordion.ItemTrigger>
    );
  } else {
    return (
      <div class="w-full text-left text-sm font-medium flex gap-2 items-center justify-between p-2">
        <div>{props.label}</div>
        {selectButton}
      </div>
    );
  }
};

export default ActionPickerHeader;
