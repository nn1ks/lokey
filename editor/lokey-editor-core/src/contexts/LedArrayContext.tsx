import {
  createContext,
  createEffect,
  createMemo,
  createResource,
  useContext,
} from "solid-js";
import type { Accessor, ParentComponent } from "solid-js";
import { useConnection } from "./ConnectionContext";
import { useMessageManager } from ".";
import * as hookId from "lokey-editor-odyn/led-array/hook-id";
import type { MessageManager } from "../protocol/message-manager";

export type LedArrayHooks = {
  boot: boolean;
  layerChange: boolean;
  blePairing: boolean;
  bleProfile: boolean;
};

export class LedArrayContextValue {
  capabilities: Accessor<LedArrayHooks | undefined>;
  enabled: Accessor<LedArrayHooks | undefined>;
  readonly #messageManager: MessageManager;
  readonly #refetchEnabled: () => unknown;

  constructor(options: {
    capabilities: Accessor<LedArrayHooks | undefined>;
    enabled: Accessor<LedArrayHooks | undefined>;
    messageManager: MessageManager;
    refetchEnabled: () => unknown;
  }) {
    this.capabilities = options.capabilities;
    this.enabled = options.enabled;
    this.#messageManager = options.messageManager;
    this.#refetchEnabled = options.refetchEnabled;
  }

  async setHooks(hooks: LedArrayHooks): Promise<void> {
    await this.#messageManager.setLedArrayHooks(toBitmask(hooks));
    await this.#refetchEnabled();
  }

  async setHook(hook: keyof LedArrayHooks, value: boolean): Promise<void> {
    const current = this.enabled() ?? {
      boot: false,
      layerChange: false,
      blePairing: false,
      bleProfile: false,
    };

    await this.setHooks({ ...current, [hook]: value });
  }
}

const LedArrayContext = createContext<LedArrayContextValue>();

export const LedArrayContextProvider: ParentComponent = (props) => {
  const { connection } = useConnection();
  const messageManager = useMessageManager();

  const [rawCapabilities] = createResource(
    () => (connection().status === "connected" ? messageManager : undefined),
    (manager) => manager.getLedArrayHookCapabilities(),
  );
  const [rawEnabled, { refetch: refetchEnabled }] = createResource(
    () => (connection().status === "connected" ? messageManager : undefined),
    (manager) => manager.getLedArrayHooks(),
  );

  createEffect(() => {
    const error = rawCapabilities.error;
    if (error) {
      // eslint-disable-next-line no-console
      console.error(
        "Unable to retrieve LED array device hook capabilities.",
        error,
      );
    }
  });

  createEffect(() => {
    const error = rawEnabled.error;
    if (error) {
      // eslint-disable-next-line no-console
      console.error("Unable to retrieve LED array device hooks.", error);
    }
  });

  const capabilities = createMemo(() => {
    if (
      rawCapabilities.state !== "ready" &&
      rawCapabilities.state !== "refreshing"
    ) {
      return undefined;
    }

    return toHooks(rawCapabilities()?.hooks);
  });

  const enabled = createMemo(() => {
    if (rawEnabled.state !== "ready" && rawEnabled.state !== "refreshing") {
      return undefined;
    }

    return toHooks(rawEnabled()?.hooks);
  });

  const value = new LedArrayContextValue({
    capabilities,
    enabled,
    messageManager,
    refetchEnabled,
  });

  return (
    <LedArrayContext.Provider value={value}>
      {props.children}
    </LedArrayContext.Provider>
  );
};

export const useLedArray = () => {
  const context = useContext(LedArrayContext);

  if (!context) {
    throw new Error(
      "useLedArray must be used inside a LedArrayContextProvider",
    );
  }

  return context;
};

function toHooks(hooks: bigint | undefined): LedArrayHooks | undefined {
  if (hooks === undefined) {
    return undefined;
  }

  const has = (hook: number) => (hooks & BigInt(hook)) !== 0n;

  return {
    boot: has(hookId.boot),
    layerChange: has(hookId.layerChange),
    blePairing: has(hookId.blePairing),
    bleProfile: has(hookId.bleProfile),
  };
}

function toBitmask(hooks: LedArrayHooks): bigint {
  let mask = 0n;

  if (hooks.boot) {
    mask |= BigInt(hookId.boot);
  }
  if (hooks.layerChange) {
    mask |= BigInt(hookId.layerChange);
  }
  if (hooks.blePairing) {
    mask |= BigInt(hookId.blePairing);
  }
  if (hooks.bleProfile) {
    mask |= BigInt(hookId.bleProfile);
  }

  return mask;
}
