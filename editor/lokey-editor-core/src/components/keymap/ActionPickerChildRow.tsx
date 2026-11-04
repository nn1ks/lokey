import { Ellipsis } from "lucide-solid";
import { Show, createSignal } from "solid-js";
import type { Component } from "solid-js";
import type { KeyAction } from "../../key-action";
import { NoAction } from "../../key-action/actions/none";
import KeyActionPicker from "./KeyActionPicker";

export type ActionPickerChildRowProps = {
  label: string;
  action: KeyAction;
  onSelect: (action: KeyAction) => void;
};

export const ActionPickerChildRow: Component<ActionPickerChildRowProps> = (
  props,
) => {
  const [pickerOpen, setPickerOpen] = createSignal(
    !(props.action instanceof NoAction),
  );

  return (
    <div class="flex flex-col gap-2">
      <button
        type="button"
        class="flex items-center justify-between rounded-sm px-3 h-9 text-left text-sm
          hover:cursor-pointer
          bg-neutral-50 hover:bg-white dark:bg-neutral-900 dark:hover:bg-neutral-950
          transition-colors duration-50"
        onClick={() => setPickerOpen((open) => !open)}
      >
        <span>{props.label}</span>
        <div class="text-neutral-500 dark:text-neutral-400">
          <Show
            when={props.action instanceof NoAction}
            fallback={<Ellipsis size="20" class="stroke-1.5" />}
          >
            <span>None</span>
          </Show>
        </div>
      </button>
      <Show when={pickerOpen()}>
        <div class="rounded-sm p-2 bg-neutral-50 dark:bg-neutral-900">
          <KeyActionPicker
            onSelect={(action) => {
              props.onSelect(action);
              setPickerOpen(false);
            }}
            selectedAction={props.action}
          />
        </div>
      </Show>
    </div>
  );
};

export default ActionPickerChildRow;
