import { Toast, Toaster as ArkToaster } from "@ark-ui/solid";
import {
  Check,
  CircleAlert,
  Info,
  Loader,
  TriangleAlert,
  X,
} from "lucide-solid";
import type { ToasterProps as ArkToasterProps } from "@ark-ui/solid";
import type { Component } from "solid-js";
import type { ToastType } from "@ark-ui/solid/toast";

export type ToasterProps = Omit<ArkToasterProps, "children">;

export const Toaster: Component<ToasterProps> = (props) => {
  const getIcon = (type: ToastType | undefined) => {
    switch (type) {
      case undefined:
      case "info":
        return (
          <Info
            size="20"
            class="stroke-neutral-600 dark:stroke-neutral-400"
          ></Info>
        );
      case "success":
        return (
          <Check
            size="20"
            class="stroke-green-600 dark:stroke-green-400"
          ></Check>
        );
      case "error":
        return (
          <CircleAlert
            size="20"
            class="stroke-red-600 dark:stroke-red-400"
          ></CircleAlert>
        );
      case "warning":
        return (
          <TriangleAlert
            size="20"
            class="stroke-orange-600 dark:stroke-orange-400"
          ></TriangleAlert>
        );
      case "loading":
        return (
          <Loader
            size="20"
            class="stroke-neutral-600 dark:stroke-neutral-400"
          ></Loader>
        );
    }
  };

  return (
    <ArkToaster {...props}>
      {(toast) => (
        <Toast.Root
          class="
            flex justify-between gap-2 px-2.5 py-2 rounded-xl min-w-70 max-w-80
            bg-neutral-300 dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300
            drop-shadow-sm
            translate-x-(--x) translate-y-(--y) opacity-(--opacity)
            will-change-[translate,opacity] transition-[translate,opacity]
          "
        >
          <div class="flex gap-2 items-center">
            <div>{getIcon(toast().type)}</div>
            <div>
              <Toast.Title class="text-md font-medium">
                {toast().title}
              </Toast.Title>
              <Toast.Description class="text-neutral-500 dark:text-neutral-400">
                {toast().description}
              </Toast.Description>
            </div>
          </div>
          <div class="py-1">
            <Toast.CloseTrigger class="flex hover:cursor-pointer">
              <X size="20" class="stroke-neutral-500 dark:stroke-neutral-400" />
            </Toast.CloseTrigger>
          </div>
        </Toast.Root>
      )}
    </ArkToaster>
  );
};

export default Toaster;
