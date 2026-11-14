import { createContext, useContext } from "solid-js";
import { KeymapState } from "../key-action/keymap-state";

export const KeymapStateContext = createContext<KeymapState>();

export const useKeymapState = (): KeymapState => {
  const keymapState = useContext(KeymapStateContext);
  if (!keymapState) {
    throw new Error(
      "useKeymapState must be used inside an KeymapStateContext.Provider",
    );
  }
  return keymapState;
};
