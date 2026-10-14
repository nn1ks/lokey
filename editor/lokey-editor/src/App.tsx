import { Match, Switch } from "solid-js";
import type { Component } from "solid-js";
import {
  DefaultHeaderTitle,
  Header,
  Toaster,
} from "lokey-editor-core/components";
import {
  DemoButton,
  ErrorMessage,
  SelectDeviceButton,
  StartPage,
  LoadingIndicator,
} from "lokey-editor-core/components/start";
import {
  ConnectionProvider,
  DeviceIdProvider,
  MessageManagerContext,
  useConnection,
  useDeviceId,
} from "lokey-editor-core/contexts";
import { MessageManager } from "lokey-editor-core/protocol";
import { FinPage } from "./components/FinPage";
import { Portal } from "solid-js/web";
import { createToaster } from "@ark-ui/solid";

export const toaster = createToaster({ placement: "bottom-end" });

const AppContent: Component = () => {
  const { connection } = useConnection();

  return (
    <>
      <Header title={<DefaultHeaderTitle />} />
      <Switch>
        <Match when={connection().status === "unsupported"}>
          <StartPage>
            <ErrorMessage>
              Web Serial API is not supported in this browser.
            </ErrorMessage>
            <DemoButton />
          </StartPage>
        </Match>
        <Match when={connection().status === "disconnected"}>
          <StartPage>
            <SelectDeviceButton />
            <DemoButton />
          </StartPage>
        </Match>
        <Match
          when={connection().status === "connected" && connection().isDemo}
        >
          <MessageManagerContext.Provider value={MessageManager.fromDemo()}>
            <DeviceIdProvider>
              <DeviceContent />
            </DeviceIdProvider>
          </MessageManagerContext.Provider>
        </Match>
        <Match
          when={connection().status === "connected" && !connection().isDemo}
        >
          <MessageManagerContext.Provider
            value={MessageManager.fromWebSerial(
              connection().writer,
              connection().reader,
            )}
          >
            <DeviceIdProvider>
              <DeviceContent />
            </DeviceIdProvider>
          </MessageManagerContext.Provider>
        </Match>
      </Switch>
      <Portal>
        <Toaster toaster={toaster} />
      </Portal>
    </>
  );
};

const DeviceContent: Component = () => {
  const { device } = useDeviceId();

  return (
    <Switch>
      <Match when={device().status === "waiting-for-connection"}>
        <StartPage>
          <LoadingIndicator label="Waiting for device connection..." />
        </StartPage>
      </Match>
      <Match when={device().status === "waiting-for-response"}>
        <StartPage>
          <LoadingIndicator label="Retrieving Device ID..." />
        </StartPage>
      </Match>
      <Match when={device().status === "retrieved"}>
        <Switch>
          <Match when={device().deviceId === 0x1000_0001}>
            <FinPage />
          </Match>
          <Match when={true}>
            <StartPage>
              <ErrorMessage>
                Device with ID 0x{device().deviceId.toString(16).toUpperCase()}{" "}
                is not supported.
              </ErrorMessage>
            </StartPage>
          </Match>
        </Switch>
      </Match>
    </Switch>
  );
};

const App: Component = () => {
  return (
    <ConnectionProvider>
      <AppContent />
    </ConnectionProvider>
  );
};

export default App;
