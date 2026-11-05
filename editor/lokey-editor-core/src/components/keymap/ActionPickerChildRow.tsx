import { ChevronRight, Ellipsis } from "lucide-solid";
import { Show, createSignal } from "solid-js";
import type { Component } from "solid-js";
import type { KeyAction } from "../../key-action";
import { None } from "../../key-action/actions/none";
import KeyActionPicker from "./KeyActionPicker";
import { Accordion } from "@ark-ui/solid";
import styles from "./ActionPickerChildRow.module.css";

export type ActionPickerChildRowProps = {
  label: string;
  action: KeyAction;
  onSelect: (action: KeyAction) => void;
};

export const ActionPickerChildRow: Component<ActionPickerChildRowProps> = (
  props,
) => {
  const itemValue = "default-item";

  const [rootValue, setRootValue] = createSignal<string[]>([]);

  if (props.action instanceof None) {
    setRootValue([]);
  } else {
    setRootValue([itemValue]);
  }

  return (
    <Accordion.Root
      class={styles.accordion}
      collapsible
      value={rootValue()}
      onValueChange={(details) => setRootValue(details.value)}
    >
      <Accordion.Item class="flex flex-col" value={itemValue}>
        <Accordion.ItemTrigger
          class="flex items-center justify-between rounded-sm
            pl-2 pr-3 h-9 text-left text-sm
            hover:cursor-pointer
            bg-neutral-50 hover:bg-white dark:bg-neutral-900 dark:hover:bg-neutral-950
            transition-colors duration-50"
        >
          <div class="flex items-center gap-1">
            <Accordion.ItemIndicator>
              <ChevronRight
                size="20"
                class="-mt-px text-neutral-500 dark:text-neutral-400 transition-rotate duration-50"
              />
            </Accordion.ItemIndicator>
            <span>{props.label}</span>
          </div>
          <div class="text-neutral-500 dark:text-neutral-400">
            <Show
              when={props.action instanceof None}
              fallback={<Ellipsis size="20" class="stroke-1.5" />}
            >
              <span>None</span>
            </Show>
          </div>
        </Accordion.ItemTrigger>
        <Accordion.ItemContent>
          <Show when={rootValue().includes(itemValue)}>
            <div class="rounded-b-sm p-2 border-t bg-neutral-50 border-neutral-300 dark:bg-neutral-900 dark:border-neutral-800">
              <KeyActionPicker
                onSelect={(action) => props.onSelect(action)}
                selectedAction={props.action}
              />
            </div>
          </Show>
        </Accordion.ItemContent>
      </Accordion.Item>
    </Accordion.Root>
  );
};

export default ActionPickerChildRow;
