import { Keyboard } from "lucide-solid";
import { Show, createEffect, createSignal } from "solid-js";
import type { Component, JSX } from "solid-js";
import { Dynamic, Portal } from "solid-js/web";
import LayerSelectorBar from "./LayerSelectorBar";
import Options from "./Options";
import styles from "./KeymapPage.module.css";
import PageHeading from "../PageHeading";
import PoweredByLokey from "../PoweredByLokey";
import WithoutPagePadding from "../WithoutPagePadding";
import {
  KeyboardActionContextProvider,
  SelectedLayerContext,
  useKeyboardActions,
  useToast,
} from "../../contexts";

export type KeymapPageProps = {
  children: JSX.Element;
  class?: string;
  /** View that is rendered instead of the page when printing. */
  printView?: Component;
};

const KeymapContent: Component<KeymapPageProps> = (props) => {
  const keyboardActions = useKeyboardActions();
  const [selectedLayer, setSelectedLayer] = createSignal(0);

  const toast = useToast();

  createEffect(() => {
    const layerCount = keyboardActions.layerCount();
    if (layerCount > 0 && selectedLayer() >= layerCount) {
      setSelectedLayer(layerCount - 1);
    }
  });

  const removeLayer = (layer: number) => {
    if (keyboardActions.layerCount() > 1) {
      void keyboardActions.deleteLayer(layer);
      toast.createToast({
        title: `Removed Layer ${layer + 1}`,
        type: "info",
      });
    } else {
      toast.createToast({
        title: `Failed to remove Layer ${layer + 1}`,
        description: "At least one layer is required.",
        type: "error",
      });
    }
  };

  return (
    <SelectedLayerContext.Provider value={selectedLayer}>
      <main class={`w-full px-(--page-padding-x) mb-8 ${props.class ?? ""}`}>
        <PageHeading
          icon={<Keyboard />}
          title="Keymap"
          description="Configure key bindings and layers"
          actions={<></>}
        />
        <WithoutPagePadding>
          <LayerSelectorBar
            layerCount={keyboardActions.layerCount()}
            selected={selectedLayer()}
            onSelect={setSelectedLayer}
            onCreate={() => {
              const newLayer = keyboardActions.layerCount();
              void keyboardActions.addLayer();
              setSelectedLayer(newLayer);
            }}
            onRemove={removeLayer}
          />
        </WithoutPagePadding>
        {props.children}
        <Options onPrint={props.printView ? () => window.print() : undefined} />
        <PoweredByLokey />
      </main>

      <Show when={props.printView}>
        {(printView) => (
          <Portal>
            <div data-print-root class={styles.printRoot}>
              <Dynamic component={printView()} />
            </div>
          </Portal>
        )}
      </Show>
    </SelectedLayerContext.Provider>
  );
};

export const KeymapPage: Component<KeymapPageProps> = (props) => {
  return (
    <KeyboardActionContextProvider>
      <KeymapContent {...props} />
    </KeyboardActionContextProvider>
  );
};

export default KeymapPage;
