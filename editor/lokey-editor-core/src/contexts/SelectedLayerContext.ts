import { createContext, useContext } from "solid-js";
import type { Accessor } from "solid-js";

/** Index of the currently selected layer. */
export const SelectedLayerContext = createContext<Accessor<number>>();

export const useSelectedLayer = () => {
  const selectedLayer = useContext(SelectedLayerContext);

  if (!selectedLayer) {
    throw new Error(
      "useSelectedLayer must be used inside a SelectedLayerContext.Provider",
    );
  }

  return selectedLayer;
};
