import { createContext, useContext } from "solid-js";
import type { Component, ParentProps } from "solid-js";
import type { CreateToasterReturn, ToastOptions } from "@ark-ui/solid/toast";

export type { ToastOptions } from "@ark-ui/solid/toast";

export type ToastContextValue = {
  /** Creates a toast message and returns its id. */
  createToast: (options: ToastOptions) => string;
};

export const ToastContext = createContext<ToastContextValue>();

export const useToast = () => {
  const context = useContext(ToastContext);

  if (!context) {
    throw new Error("useToast must be used inside a ToastContext.Provider");
  }

  return context;
};

export type ToastProviderProps = ParentProps<{
  toaster: CreateToasterReturn;
}>;

export const ToastProvider: Component<ToastProviderProps> = (props) => {
  const value: ToastContextValue = {
    createToast: (options) => props.toaster.create(options),
  };

  return (
    <ToastContext.Provider value={value}>{props.children}</ToastContext.Provider>
  );
};
