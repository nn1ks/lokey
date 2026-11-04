import { createContext, useContext } from "solid-js";
import type { Component, ParentProps } from "solid-js";
import type { ActionRegistry } from "../key-action/registry";

const ActionRegistryContext = createContext<ActionRegistry>();

export type ActionRegistryContextProviderProps = ParentProps<{
  registry: ActionRegistry;
}>;

export const ActionRegistryContextProvider: Component<
  ActionRegistryContextProviderProps
> = (props) => (
  <ActionRegistryContext.Provider value={props.registry}>
    {props.children}
  </ActionRegistryContext.Provider>
);

export const useActionRegistry = (): ActionRegistry => {
  const registry = useContext(ActionRegistryContext);
  if (!registry) {
    throw new Error(
      "useActionRegistry must be used inside an ActionRegistryContextProvider",
    );
  }
  return registry;
};
