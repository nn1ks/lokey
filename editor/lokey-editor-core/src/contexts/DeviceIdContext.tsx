import {
  createContext,
  createEffect,
  createResource,
  useContext,
} from "solid-js";
import type { Accessor, ParentComponent } from "solid-js";
import { useConnection } from "./ConnectionContext";
import { useMessageManager } from ".";

export type DeviceIdState =
  | { status: "waiting-for-connection" }
  | { status: "waiting-for-response" }
  | { status: "retrieved"; deviceId: number };

export type DeviceIdContextValue = {
  device: Accessor<DeviceIdState>;
};

export const DeviceIdContext = createContext<DeviceIdContextValue>();

export const DeviceIdProvider: ParentComponent = (props) => {
  const { connection } = useConnection();
  const messageManager = useMessageManager();

  const [capabilities] = createResource(
    () => (connection().status === "connected" ? messageManager : undefined),
    (manager) => manager.getDeviceCapabilities(),
  );

  createEffect(() => {
    const error = capabilities.error;
    if (error) {
      // eslint-disable-next-line no-console
      console.error("Unable to retrieve the device ID.", error);
    }
  });

  const device: Accessor<DeviceIdState> = () => {
    if (connection().status !== "connected") {
      return { status: "waiting-for-connection" };
    }

    if (capabilities.state === "ready") {
      return {
        status: "retrieved",
        deviceId: capabilities().deviceId,
      };
    }

    return { status: "waiting-for-response" };
  };

  return (
    <DeviceIdContext.Provider value={{ device }}>
      {props.children}
    </DeviceIdContext.Provider>
  );
};

export const useDeviceId = () => {
  const context = useContext(DeviceIdContext);

  if (!context) {
    throw new Error("useDeviceId must be used inside a DeviceIdProvider");
  }

  return context;
};
