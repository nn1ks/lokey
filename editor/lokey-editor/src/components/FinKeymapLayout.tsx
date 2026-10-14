import { For, useContext } from "solid-js";
import type { Component } from "solid-js";
import { FinKeyActionsContext } from "../contexts/FinKeyActionsContext";
import { KeyButton } from "lokey-editor-core/components/keymap";
import styles from "./FinKeymapLayout.module.css";

export const FinKeymapLayout: Component = () => {
  const finKeyActions = useContext(FinKeyActionsContext);
  const selectedLayer = 0;
  const keyActions = () => finKeyActions?.[selectedLayer]?.[1] ?? [];

  return (
    <div
      class="
        relative box-border flex flex-auto h-min min-w-0
        w-[calc(100%+2*var(--page-padding-x))] -mx-(--page-padding-x)
        min-[1155px]:w-full min-[1155px]:mx-0
        items-center justify-start overflow-x-auto rounded-none
        py-(--layout-container-padding) mt-3
        bg-neutral-150
        shadow-[inset_0_1px_0_var(--color-neutral-300),inset_0_-1px_0_var(--color-neutral-300)]
        dark:bg-neutral-900
        dark:shadow-[inset_0_1px_0_var(--color-neutral-800),inset_0_-1px_0_var(--color-neutral-800)]
        min-[1155px]:rounded-2xl min-[1155px]:px-[15px]
        min-[1155px]:shadow-none dark:min-[1155px]:shadow-none
        [--layout-container-padding:20px]
      "
    >
      <picture class="block h-auto w-[1100px] min-w-[1100px] pointer-events-none select-none">
        <source
          srcset="/device-fin-dark.svg"
          media="(prefers-color-scheme: dark)"
        />
        <img src="/device-fin-light.svg" alt="Fin keyboard layout" />
      </picture>
      <div class={styles.key_button_container}>
        <For each={keyActions()}>
          {(action) => <KeyButton keyAction={action} />}
        </For>
      </div>
    </div>
  );
};

export default FinKeymapLayout;
