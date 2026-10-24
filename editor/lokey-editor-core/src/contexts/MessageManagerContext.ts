import { createContext, useContext } from "solid-js";
import type { MessageManager } from "../protocol/MessageManager";

export const MessageManagerContext = createContext<MessageManager>();

export const useMessageManager = () => {
  const manager = useContext(MessageManagerContext);

  if (!manager) {
    throw new Error(
      "useMessageManager must be used inside a MessageManagerContext.Provider",
    );
  }

  return manager;
};
