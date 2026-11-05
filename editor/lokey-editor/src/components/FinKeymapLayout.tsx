import { Index, Show, createMemo, createSignal } from "solid-js";
import type { Component } from "solid-js";
import {
  KeyActionPickerDialog,
  KeyButton,
} from "lokey-editor-core/components/keymap";
import {
  useKeyboardActions,
  useSelectedLayer,
  useTheme,
} from "lokey-editor-core/contexts";
import styles from "./FinKeymapLayout.module.css";

export const FinKeymapLayout: Component = () => {
  const keyboardActions = useKeyboardActions();
  const selectedLayer = useSelectedLayer();
  const { theme } = useTheme();

  const [selectedKey, setSelectedKey] = createSignal<{
    keyIndex: number;
    layer: number;
  }>();

  const selectedAction = () => {
    const key = selectedKey();
    return key
      ? keyboardActions.actions()[key.keyIndex]?.[key.layer]
      : undefined;
  };

  const keyIndices = createMemo(() =>
    Array.from(
      { length: keyboardActions.actions().length },
      (_, index) => index,
    ),
  );

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
      <img
        class="block h-auto w-[1100px] min-w-[1100px] pointer-events-none select-none"
        src={
          theme() === "dark" ? "/device-fin-dark.svg" : "/device-fin-light.svg"
        }
        alt="Fin keyboard layout"
      />
      <div class={styles.key_button_container}>
        <Index each={keyIndices()}>
          {(keyIndex) => {
            const action = createMemo(() => {
              const layer = selectedLayer();
              return keyboardActions.actions()[keyIndex()]?.[layer];
            });

            return (
              <Show when={action()}>
                {(action) => (
                  <KeyButton
                    keyAction={action()}
                    onClick={() =>
                      setSelectedKey({
                        keyIndex: keyIndex(),
                        layer: selectedLayer(),
                      })
                    }
                  />
                )}
              </Show>
            );
          }}
        </Index>
      </div>
      <KeyActionPickerDialog
        open={selectedKey() !== undefined}
        onOpenChange={(open) => {
          if (!open) {
            setSelectedKey(undefined);
          }
        }}
        onSelect={(action) => {
          const key = selectedKey();
          if (key) {
            void keyboardActions.setAction(key.keyIndex, key.layer, action);
          }
        }}
        selectedAction={selectedAction()}
      />
    </div>
  );
};

export default FinKeymapLayout;
