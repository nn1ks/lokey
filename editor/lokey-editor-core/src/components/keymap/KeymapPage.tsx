import { Keyboard } from "lucide-solid";
import { Show, createEffect, createSignal } from "solid-js";
import type { Component, JSX } from "solid-js";
import { Dynamic, Portal } from "solid-js/web";
import LayerSelectorBar from "./LayerSelectorBar";
import Options from "./Options";
import styles from "./KeymapPage.module.css";
import ConfirmDialog from "../ConfirmDialog";
import PageHeading from "../PageHeading";
import PoweredByLokey from "../PoweredByLokey";
import WithoutPagePadding from "../WithoutPagePadding";
import {
  ActionRegistryContextProvider,
  KeymapStateContext,
  SelectedLayerContext,
  useKeymapState,
  useToast,
} from "../../contexts";
import { KeymapStateInitializer } from "./KeymapStateInitializer";
import { LoadingIndicator, StartPage } from "../start";
import { ActionRegistry } from "../../key-action";

export type KeymapPageProps = {
  children: JSX.Element;
  actionRegistry: ActionRegistry;
  class?: string;
  /** View that is rendered instead of the page when printing. */
  printView?: Component;
};

const KeymapContent: Component<KeymapPageProps> = (props) => {
  const keymapState = useKeymapState();
  const [selectedLayer, setSelectedLayer] = createSignal(0);
  const [pendingRemoval, setPendingRemoval] = createSignal<number>();

  const toast = useToast();

  createEffect(() => {
    const layerCount = keymapState.layerCount();
    if (layerCount > 0 && selectedLayer() >= layerCount) {
      setSelectedLayer(layerCount - 1);
    }
  });

  const removeLayer = (layer: number) => {
    if (keymapState.layerCount() <= 1) {
      toast.createToast({
        title: `Failed to remove Layer ${layer + 1}`,
        description: "At least one layer is required.",
        type: "error",
      });
      return;
    }

    setPendingRemoval(layer);
  };

  const confirmRemoveLayer = () => {
    const layer = pendingRemoval();
    if (layer === undefined) {
      return;
    }

    void keymapState.removeLayer(layer);
    toast.createToast({
      title: `Removed Layer ${layer + 1}`,
      type: "info",
    });
    setPendingRemoval(undefined);
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
            layerCount={keymapState.layerCount()}
            selected={selectedLayer()}
            onSelect={setSelectedLayer}
            onMoveLeft={(layer) => {
              if (layer <= 0) return;
              void keymapState.moveLayer(layer, layer - 1);
              setSelectedLayer(layer - 1);
            }}
            onMoveRight={(layer) => {
              if (layer >= keymapState.layerCount() - 1) return;
              void keymapState.moveLayer(layer, layer + 1);
              setSelectedLayer(layer + 1);
            }}
            onCreate={() => {
              const newLayer = keymapState.layerCount();
              void keymapState.addLayer();
              setSelectedLayer(newLayer);
            }}
            onRemove={removeLayer}
          />
        </WithoutPagePadding>
        {props.children}
        <Options onPrint={props.printView ? () => window.print() : undefined} />
        <PoweredByLokey />
      </main>

      <ConfirmDialog
        open={pendingRemoval() !== undefined}
        onOpenChange={(open) => {
          if (!open) {
            setPendingRemoval(undefined);
          }
        }}
        title={`Remove Layer ${(pendingRemoval() ?? 0) + 1}?`}
        description="This removes the layer and all its key actions."
        confirmLabel="Remove"
        onConfirm={confirmRemoveLayer}
      />

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
    <ActionRegistryContextProvider actionRegistry={props.actionRegistry}>
      <KeymapStateInitializer
        fallback={
          <div class="flex w-full max-w-76 flex-col items-center gap-4 py-20">
            <LoadingIndicator label="Retrieving keymap..." />
          </div>
        }
      >
        {(state) => (
          <KeymapStateContext.Provider value={state}>
            <KeymapContent {...props} />
          </KeymapStateContext.Provider>
        )}
      </KeymapStateInitializer>
    </ActionRegistryContextProvider>
  );
};

export default KeymapPage;
