import {
  Component,
  createEffect,
  createResource,
  createSignal,
  JSX,
  Show,
} from "solid-js";
import { KeymapState } from "../../key-action/keymap-state";
import {
  useActionRegistry,
  useConnection,
  useMessageManager,
} from "../../contexts";
import { ActionRegistry, Layout } from "../../key-action";

export type KeymapStateInitializerProps = {
  fallback: JSX.Element;
  children: (keymapState: KeymapState) => JSX.Element;
};

export const KeymapStateInitializer: Component<KeymapStateInitializerProps> = (
  props,
) => {
  const { connection } = useConnection();
  const messageManager = useMessageManager();
  const registry = useActionRegistry();

  const [rawHostLayout] = createResource(
    () => (connection().status === "connected" ? messageManager : undefined),
    (manager) => manager.getHostLayout(),
  );

  const [rawLayout] = createResource(
    () => (connection().status === "connected" ? messageManager : undefined),
    (manager) => manager.getLayout(),
  );
  const [rawEntries] = createResource(
    () => (connection().status === "connected" ? messageManager : undefined),
    (manager) => manager.getActiveActionEntries(),
  );

  createEffect(() => {
    const error = rawHostLayout.error;
    if (error) {
      // eslint-disable-next-line no-console
      console.error("Unable to retrieve the host layout.", error);
    }
  });

  createEffect(() => {
    const error = rawLayout.error;
    if (error) {
      // eslint-disable-next-line no-console
      console.error("Unable to retrieve the keyboard layout.", error);
    }
  });

  createEffect(() => {
    const error = rawEntries.error;
    if (error) {
      // eslint-disable-next-line no-console
      console.error("Unable to retrieve the keyboard action entries.", error);
    }
  });

  const [state, setState] = createSignal<KeymapState>();

  createEffect(() => {
    if (
      rawHostLayout.state !== "ready" &&
      rawHostLayout.state !== "refreshing"
    ) {
      return;
    }
    if (rawLayout.state !== "ready" && rawLayout.state !== "refreshing") {
      return;
    }
    if (rawEntries.state !== "ready" && rawEntries.state !== "refreshing") {
      return;
    }

    setState(
      KeymapState.fromResponses(
        messageManager,
        registry,
        rawHostLayout().hostLayout,
        rawEntries(),
        rawLayout(),
      ),
    );
  });

  return (
    <Show when={state()} fallback={props.fallback}>
      {(state) => props.children(state())}
    </Show>
  );
};
