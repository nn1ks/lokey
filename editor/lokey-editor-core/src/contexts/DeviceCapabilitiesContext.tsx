import {
  createContext,
  createEffect,
  createResource,
  useContext,
} from "solid-js";
import type { Accessor, ParentComponent } from "solid-js";
import type { DeviceType } from "lokey-editor-odyn/core";
import { useConnection } from "./ConnectionContext";
import { useMessageManager } from ".";

export type DeviceCapabilitiesState =
  | { status: "waiting-for-connection" }
  | { status: "waiting-for-response" }
  | {
      status: "retrieved";
      deviceId: number;
      deviceName: string;
      deviceType: DeviceType;
    };

export type DeviceCapabilitiesContextValue = {
  device: Accessor<DeviceCapabilitiesState>;
};

export const DeviceCapabilitiesContext =
  createContext<DeviceCapabilitiesContextValue>();

export const DeviceCapabilitiesProvider: ParentComponent = (props) => {
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
      console.error("Unable to retrieve the device capabilities.", error);
    }
  });

  const device: Accessor<DeviceCapabilitiesState> = () => {
    if (connection().status !== "connected") {
      return { status: "waiting-for-connection" };
    }

    if (capabilities.state === "ready") {
      return {
        status: "retrieved",
        deviceId: capabilities().deviceId,
        deviceName: capabilities().deviceName,
        deviceType: capabilities().deviceType,
      };
    }

    return { status: "waiting-for-response" };
  };

  return (
    <DeviceCapabilitiesContext.Provider value={{ device }}>
      {props.children}
    </DeviceCapabilitiesContext.Provider>
  );
};

export const useDeviceCapabilities = () => {
  const context = useContext(DeviceCapabilitiesContext);

  if (!context) {
    throw new Error(
      "useDeviceCapabilities must be used inside a DeviceCapabilitiesProvider",
    );
  }

  return context;
};
