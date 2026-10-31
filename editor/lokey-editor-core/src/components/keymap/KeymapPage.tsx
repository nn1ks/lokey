import { Keyboard } from "lucide-solid";
import { createEffect, createSignal } from "solid-js";
import type { Component, JSX } from "solid-js";
import LayerSelectorBar from "./LayerSelectorBar";
import Options from "./Options";
import PageHeading from "../PageHeading";
import PoweredByLokey from "../PoweredByLokey";
import WithoutPagePadding from "../WithoutPagePadding";
import {
  KeyboardActionContextProvider,
  SelectedLayerContext,
  useKeyboardActions,
} from "../../contexts";

export type KeymapPageProps = {
  children: JSX.Element;
  class?: string;
};

const KeymapContent: Component<KeymapPageProps> = (props) => {
  const keyboardActions = useKeyboardActions();
  const [selectedLayer, setSelectedLayer] = createSignal(0);

  createEffect(() => {
    const layerCount = keyboardActions.layerCount();
    if (layerCount > 0 && selectedLayer() >= layerCount) {
      setSelectedLayer(layerCount - 1);
    }
  });

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
            onRemove={
              keyboardActions.layerCount() > 1
                ? (layer) => void keyboardActions.deleteLayer(layer)
                : undefined
            }
          />
        </WithoutPagePadding>
        {props.children}
        <Options />
        <PoweredByLokey />
      </main>
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
