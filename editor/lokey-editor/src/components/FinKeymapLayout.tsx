import { Index, Show, createMemo, createSignal } from "solid-js";
import type { Component } from "solid-js";
import {
  KeyActionPickerDialog,
  KeyButton,
} from "lokey-editor-core/components/keymap";
import {
  useKeymapState,
  useSelectedLayer,
  useTheme,
} from "lokey-editor-core/contexts";
import styles from "./FinKeymapLayout.module.css";

export const FinKeymapLayout: Component = () => {
  const keymapState = useKeymapState();
  const selectedLayer = useSelectedLayer();
  const { theme } = useTheme();

  const [selectedKey, setSelectedKey] = createSignal<{
    keyIndex: number;
    layer: number;
  }>();

  const selectedAction = () => {
    const key = selectedKey();
    return key ? keymapState.actionAt(key.layer, key.keyIndex) : undefined;
  };

  const keyIndices = createMemo(() =>
    Array.from(
      { length: keymapState.rootActionEntryIds.length },
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
        <Show when={keymapState.rootActionEntryIds.length > 0}>
          <Index each={keyIndices()}>
            {(keyIndex) => {
              const action = createMemo(() => {
                const layerCount = keymapState.layerCount();
                // `selectedLayer` can briefly be out of range while a layer is
                // being removed, so clamp it to the last valid layer.
                const layer = Math.min(selectedLayer(), layerCount - 1);
                return keymapState.actionAt(layer, keyIndex());
              });

              return (
                <Show when={action()} keyed>
                  {(action) => {
                    return (
                      <KeyButton
                        keyAction={action}
                        onClick={() =>
                          setSelectedKey({
                            keyIndex: keyIndex(),
                            layer: selectedLayer(),
                          })
                        }
                      />
                    );
                  }}
                </Show>
              );
            }}
          </Index>
        </Show>
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
            void keymapState.pushActionAt(action, key.layer, key.keyIndex);
          }
        }}
        selectedAction={selectedAction()}
      />
    </div>
  );
};

export default FinKeymapLayout;
