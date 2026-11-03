import { For } from "solid-js";
import type { Component } from "solid-js";
import type { KeyAction } from "lokey-editor-core";
import { ActionRenderer } from "lokey-editor-core/components/keymap";
import { useKeyboardActions } from "lokey-editor-core/contexts";
import styles from "./FinKeymapLayout.module.css";

const PrintKey: Component<{ keyAction: KeyAction }> = (props) => {
  return (
    <>
      <div
        class="
          absolute flex items-center justify-center
          w-[64px] h-[60px] rounded-lg border
          bg-white text-neutral-900 border-neutral-300
          text-[16px] font-medium
          [&_svg]:size-[1.4em] [&_svg]:stroke-[1.8]
        "
      >
        <ActionRenderer action={props.keyAction} level={0} />
      </div>
      {/* `KeyButton` renders two elements (the trigger and the tooltip
          positioner), so the print cells need a matching placeholder to keep
          the `nth-child` position selectors aligned. */}
      <div></div>
    </>
  );
};

export const FinKeymapLayoutPrint: Component = () => {
  const keyboardActions = useKeyboardActions();

  const layers = () =>
    Array.from({ length: keyboardActions.layerCount() }, (_, index) => index);

  const keyActions = (layer: number) =>
    keyboardActions
      .actions()
      .map((layers) => layers[layer])
      .filter((action) => action !== undefined);

  return (
    <div class="flex flex-col gap-8 bg-white p-4">
      <For each={layers()}>
        {(layer) => (
          <section>
            <h2 class="mb-2 text-base font-semibold text-neutral-900">
              Layer {layer + 1}
            </h2>
            <div
              class="
                relative box-border flex h-min w-[1100px]
                items-center justify-start
                py-(--layout-container-padding)
                [--layout-container-padding:20px]
              "
            >
              <img
                class="block h-auto w-[1100px] min-w-[1100px] pointer-events-none select-none"
                src="/device-fin-light.svg"
                alt="Fin keyboard layout"
              />
              <div class={styles.key_button_container}>
                <For each={keyActions(layer)}>
                  {(action) => <PrintKey keyAction={action} />}
                </For>
              </div>
            </div>
          </section>
        )}
      </For>
    </div>
  );
};

export default FinKeymapLayoutPrint;
