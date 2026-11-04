import { Dialog } from "@ark-ui/solid";
import { Portal } from "solid-js/web";
import { createSignal } from "solid-js";
import type { Component } from "solid-js";
import type { KeyAction } from "../../key-action";
import KeyActionPicker from "./KeyActionPicker";
import { X } from "lucide-solid";

export type KeyActionPickerDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSelect: (action: KeyAction) => void;
  selectedAction?: KeyAction;
};

export const KeyActionPickerDialog: Component<KeyActionPickerDialogProps> = (
  props,
) => {
  const [scrolled, setScrolled] = createSignal(false);

  return (
    <Dialog.Root
      open={props.open}
      onOpenChange={(details) => props.onOpenChange(details.open)}
    >
      <Portal>
        <Dialog.Backdrop class="fixed inset-0 z-1000 bg-black/40" />
        <Dialog.Positioner class="fixed inset-0 z-1000 flex items-center justify-center sm:p-3">
          <Dialog.Content
            class="
              flex h-full sm:max-h-[min(90vh,48rem)] sm:min-h-[min(90vh,48rem)] w-full max-w-2xl flex-col
              overflow-hidden border shadow-xl sm:rounded-2xl
              border-neutral-50 bg-neutral-50
              dark:border-neutral-800 dark:bg-neutral-900
            "
          >
            <div
              class={`flex items-center justify-between gap-4 border-b px-4 pt-3 pb-2
                transition duration-100 z-2000
                ${
                  scrolled()
                    ? "border-neutral-200 dark:border-neutral-800 shadow-md"
                    : "border-transparent shadow-none"
                }`}
            >
              <Dialog.Title class="text-lg font-semibold">
                Select action
              </Dialog.Title>
              <button
                type="button"
                class="
                  flex size-8 cursor-pointer items-center justify-center rounded-lg
                  text-neutral-600 transition-colors duration-50
                  hover:bg-neutral-200 hover:text-neutral-900
                  dark:text-neutral-400 dark:hover:bg-neutral-800
                  dark:hover:text-neutral-100
                "
                aria-label="Close"
                onClick={() => props.onOpenChange(false)}
              >
                <X size="22" />
              </button>
            </div>
            <div
              class="min-h-0 flex-1 overflow-y-auto px-4 pt-1 pb-4"
              onScroll={(event) =>
                setScrolled(event.currentTarget.scrollTop > 0)
              }
            >
              <KeyActionPicker
                onSelect={(action) => {
                  props.onSelect(action);
                  props.onOpenChange(false);
                }}
                selectedAction={props.selectedAction}
              />
            </div>
          </Dialog.Content>
        </Dialog.Positioner>
      </Portal>
    </Dialog.Root>
  );
};

export default KeyActionPickerDialog;
