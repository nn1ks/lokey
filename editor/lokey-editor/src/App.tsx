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
  DeviceCapabilitiesProvider,
  MessageManagerContext,
  ThemeProvider,
  ToastProvider,
  useConnection,
  useDeviceCapabilities,
} from "lokey-editor-core/contexts";
import { MessageManager } from "lokey-editor-core/protocol";
import { FinPage } from "./components/FinPage";
import { Portal } from "solid-js/web";
import { createToaster } from "@ark-ui/solid";

export const toaster = createToaster({
  placement: "bottom-end",
  overlap: false,
  gap: 10,
});

const AppContent: Component = () => {
  const { connection } = useConnection();

  return (
    <>
      <Switch>
        <Match when={connection().status === "unsupported"}>
          <Header title={<DefaultHeaderTitle />} class="max-w-250" />
          <StartPage>
            <ErrorMessage>
              Web Serial API is not supported in this browser.
            </ErrorMessage>
            <DemoButton />
          </StartPage>
        </Match>
        <Match when={connection().status === "disconnected"}>
          <Header title={<DefaultHeaderTitle />} class="max-w-250" />
          <StartPage>
            <SelectDeviceButton />
            <DemoButton />
          </StartPage>
        </Match>
        <Match
          when={connection().status === "connected" && connection().isDemo}
        >
          <MessageManagerContext.Provider value={MessageManager.fromDemo()}>
            <DeviceCapabilitiesProvider>
              <DeviceContent />
            </DeviceCapabilitiesProvider>
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
            <DeviceCapabilitiesProvider>
              <DeviceContent />
            </DeviceCapabilitiesProvider>
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
  const { device } = useDeviceCapabilities();

  return (
    <Switch>
      <Match when={device().status === "waiting-for-connection"}>
        <Header title={<DefaultHeaderTitle />} class="max-w-250" />
        <StartPage>
          <LoadingIndicator label="Waiting for device connection..." />
        </StartPage>
      </Match>
      <Match when={device().status === "waiting-for-response"}>
        <Header title={<DefaultHeaderTitle />} class="max-w-250" />
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
            <Header title={<DefaultHeaderTitle />} class="max-w-250" />
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
    <ThemeProvider>
      <ToastProvider toaster={toaster}>
        <ConnectionProvider>
          <AppContent />
        </ConnectionProvider>
      </ToastProvider>
    </ThemeProvider>
  );
};

export default App;
