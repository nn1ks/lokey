import { Dialog } from "@ark-ui/solid";
import { Portal } from "solid-js/web";
import { Show } from "solid-js";
import type { Component, JSX } from "solid-js";
import Button from "./Button";

export type ConfirmDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: JSX.Element;
  description?: JSX.Element;
  confirmLabel?: string;
  cancelLabel?: string;
  onConfirm: () => void;
};

export const ConfirmDialog: Component<ConfirmDialogProps> = (props) => {
  return (
    <Dialog.Root
      open={props.open}
      onOpenChange={(details) => props.onOpenChange(details.open)}
      role="alertdialog"
    >
      <Portal>
        <Dialog.Backdrop class="fixed inset-0 z-1000 bg-black/40" />
        <Dialog.Positioner class="fixed inset-0 z-1000 flex items-center justify-center p-4">
          <Dialog.Content
            class="
              w-full max-w-sm rounded-2xl border p-5 shadow-xl
              border-neutral-50 bg-neutral-50
              dark:border-neutral-800 dark:bg-neutral-900
            "
          >
            <Dialog.Title class="text-lg font-semibold">
              {props.title}
            </Dialog.Title>
            <Show when={props.description}>
              <Dialog.Description class="mt-2 text-md text-neutral-600 dark:text-neutral-400">
                {props.description}
              </Dialog.Description>
            </Show>
            <div class="mt-5 flex justify-end gap-2">
              <Button onClick={() => props.onOpenChange(false)}>
                {props.cancelLabel ?? "Cancel"}
              </Button>
              <Button
                class="
                  bg-red-700! text-white! hover:bg-red-800!
                  dark:bg-red-800! dark:text-white! dark:hover:bg-red-900!
                "
                onClick={() => {
                  props.onConfirm();
                  props.onOpenChange(false);
                }}
              >
                {props.confirmLabel ?? "Confirm"}
              </Button>
            </div>
          </Dialog.Content>
        </Dialog.Positioner>
      </Portal>
    </Dialog.Root>
  );
};

export default ConfirmDialog;
