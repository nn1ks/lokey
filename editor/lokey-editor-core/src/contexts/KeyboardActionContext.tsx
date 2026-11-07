import {
  createContext,
  createEffect,
  createMemo,
  createResource,
  createSignal,
  useContext,
} from "solid-js";
import type { Accessor, Component, ParentProps } from "solid-js";
import {
  GetActiveActionEntriesResponse,
  GetLayoutResponse,
  HostLayout,
  KeyboardAction,
  KeyboardActionEntry,
} from "lokey-editor-odyn/keyboard";
import { None } from "../key-action/actions/none";
import type { KeyAction, RootKeyAction } from "../key-action";
import {
  decodeRootKeyAction,
  encodeRootKeyAction,
} from "../key-action/codec/root-key-action";
import type { ActionRegistry } from "../key-action/registry";
import type { Layout } from "../layouts";
import { useConnection } from "./ConnectionContext";
import { useMessageManager } from ".";
import type { MessageManager } from "../protocol/MessageManager";
import { ActionRegistryContextProvider } from "./ActionRegistryContext";
import { buildActionEntry } from "../key-action/codec/build";
import { DecodeContext } from "../key-action/contexts";

const noneAction: KeyAction = new None();

export class KeyboardActionContextValue {
  readonly layerCount: Accessor<number>;
  /** Actions indexed by `[keyIndex][layer]`. */
  readonly actions: Accessor<KeyAction[][]>;
  readonly #keys: Accessor<RootKeyAction[] | undefined>;
  readonly #setKeys: (keys: RootKeyAction[]) => void;
  readonly hostLayout: Accessor<Layout | undefined>;
  readonly #setHostLayout: (hostLayout: Layout) => void;
  readonly #messageManager: MessageManager;
  readonly #registry: ActionRegistry;

  constructor(options: {
    layerCount: Accessor<number>;
    actions: Accessor<KeyAction[][]>;
    keys: Accessor<RootKeyAction[] | undefined>;
    setKeys: (keys: RootKeyAction[]) => void;
    hostLayout: Accessor<Layout | undefined>;
    setHostLayout: (hostLayout: Layout) => void;
    messageManager: MessageManager;
    registry: ActionRegistry;
  }) {
    this.layerCount = options.layerCount;
    this.actions = options.actions;
    this.#keys = options.keys;
    this.#setKeys = options.setKeys;
    this.hostLayout = options.hostLayout;
    this.#setHostLayout = options.setHostLayout;
    this.#messageManager = options.messageManager;
    this.#registry = options.registry;
  }

  async setHostLayout(hostLayout: Layout): Promise<void> {
    this.#setHostLayout(hostLayout);
    await this.#messageManager.setHostLayout(
      convertToRawHostLayout(hostLayout),
    );
    const current = this.#keys();
    if (current) {
      await this.#persist(current);
    }
  }

  async setAction(
    keyIndex: number,
    layer: number,
    action: KeyAction,
  ): Promise<void> {
    const current = this.#requireKeys();
    const layerCount = this.layerCount();

    if (keyIndex < 0 || keyIndex >= current.length) {
      throw new Error(
        `Key index ${keyIndex} is out of range, expected 0..${current.length - 1}.`,
      );
    }
    if (layer < 0 || layer >= layerCount) {
      throw new Error(
        `Layer ${layer} is out of range, expected 0..${layerCount - 1}.`,
      );
    }

    const next = current.map((key, index) => {
      if (index !== keyIndex) {
        return key;
      }

      const actions =
        key.kind === "single"
          ? Array.from({ length: layerCount }, () => key.action)
          : [...key.actions];
      while (actions.length < layerCount) {
        actions.push(noneAction);
      }
      actions[layer] = action;

      return { kind: "perLayer", actions } satisfies RootKeyAction;
    });

    this.#setKeys(next);
    await this.#persist(next);
  }

  async addLayer(): Promise<void> {
    const current = this.#requireKeys();
    const layerCount = this.layerCount();

    const next = current.map((key): RootKeyAction => {
      if (key.kind === "single") {
        return {
          kind: "perLayer",
          actions: [
            ...Array.from({ length: layerCount }, () => key.action),
            noneAction,
          ],
        };
      }

      const actions = [...key.actions];
      while (actions.length < layerCount) {
        actions.push(noneAction);
      }
      actions.push(noneAction);

      return { kind: "perLayer", actions };
    });

    this.#setKeys(next);
    await this.#persist(next);
  }

  async deleteLayer(layer: number): Promise<void> {
    const current = this.#requireKeys();
    const layerCount = this.layerCount();

    if (layer < 0 || layer >= layerCount) {
      throw new Error(
        `Layer ${layer} is out of range, expected 0..${layerCount - 1}.`,
      );
    }
    if (layerCount <= 1) {
      throw new Error("Cannot delete the only remaining layer.");
    }

    const next = current.map((key): RootKeyAction => {
      if (key.kind === "single") {
        return key;
      }

      const actions = key.actions.filter((_, index) => index !== layer);
      return { kind: "perLayer", actions };
    });

    this.#setKeys(next);
    await this.#persist(next);
  }

  async moveLayer(from: number, to: number): Promise<void> {
    const current = this.#requireKeys();
    const layerCount = this.layerCount();

    if (from < 0 || from >= layerCount) {
      throw new Error(
        `Layer ${from} is out of range, expected 0..${layerCount - 1}.`,
      );
    }
    if (to < 0 || to >= layerCount) {
      throw new Error(
        `Layer ${to} is out of range, expected 0..${layerCount - 1}.`,
      );
    }
    if (from === to) {
      return;
    }

    const actionsChanged = current
      .map((key): boolean => {
        if (key.kind === "single") {
          return remapLayer(key.action, from, to);
        }

        const actions = [...key.actions];
        while (actions.length < layerCount) {
          actions.push(noneAction);
        }

        const [moved] = actions.splice(from, 1);
        actions.splice(to, 0, moved ?? noneAction);

        actions.forEach((action) => remapLayer(action, from, to));

        key.actions = actions;

        return true;
      })
      .some((v) => v);

    if (actionsChanged) {
      const keys = this.#keys();
      if (keys) {
        this.#setKeys([...keys]);
        await this.#persist(keys);
      }
    }
  }

  #requireKeys(): RootKeyAction[] {
    const keys = this.#keys();
    if (!keys) {
      throw new Error("The keyboard actions have not been loaded yet.");
    }
    return keys;
  }

  async #persist(keys: RootKeyAction[]): Promise<void> {
    const layout = this.hostLayout();
    if (layout === undefined) {
      console.error("Host layout is undefined.");
      return;
    }

    const entries: KeyboardActionEntry[] = [];
    let nextEntryId = 0;

    const pushAction = (action: KeyboardAction) => {
      const entryId = nextEntryId;
      nextEntryId++;
      const actionEntry = buildActionEntry(entryId, action);
      entries.push(actionEntry);
      return entryId;
    };

    const keyEntryIds = keys.map((key) =>
      encodeRootKeyAction(key, {
        hostLayout: layout,
        pushAction,
      }),
    );

    await this.#messageManager.addActionEntries(entries);
    await this.#messageManager.updateLayout(0, keyEntryIds);
  }
}

const KeyboardActionContext = createContext<KeyboardActionContextValue>();

export type KeyboardActionContextProviderProps = ParentProps<{
  registry: ActionRegistry;
}>;

export const KeyboardActionContextProvider: Component<
  KeyboardActionContextProviderProps
> = (props) => {
  const { connection } = useConnection();
  const messageManager = useMessageManager();
  const registry = props.registry;

  const [rawHostLayout] = createResource(
    () => (connection().status === "connected" ? messageManager : undefined),
    (manager) => manager.getHostLayout(),
  );

  const [hostLayout, setHostLayout] = createSignal<Layout>();

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

  const [keys, setKeys] = createSignal<RootKeyAction[]>();

  createEffect(() => {
    if (
      rawHostLayout.state !== "ready" &&
      rawHostLayout.state !== "refreshing"
    ) {
      return;
    }
    let hostLayout = convertFromRawHostLayout(rawHostLayout().hostLayout);
    if (hostLayout === undefined) {
      console.warn(`Unknown host layout "${hostLayout}".`);
      hostLayout = "english-us";
    }
    setHostLayout(hostLayout);
  });

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

    setKeys(
      resolveKeys(
        rawLayout(),
        rawEntries(),
        rawHostLayout().hostLayout,
        registry,
        noneAction,
      ),
    );
  });

  const layerCount = createMemo(() => {
    const current = keys();
    if (!current) {
      return 0;
    }

    let count = 1;
    for (const key of current) {
      if (key.kind === "perLayer") {
        count = Math.max(count, key.actions.length);
      }
    }
    return count;
  });

  const actions = createMemo(() => {
    const current = keys();
    if (!current) {
      return [];
    }

    const count = layerCount();
    return current.map((key): KeyAction[] => {
      if (key.kind === "single") {
        return Array.from({ length: count }, () => key.action);
      }
      return Array.from(
        { length: count },
        (_, layer) => key.actions[layer] ?? noneAction,
      );
    });
  });

  const value = new KeyboardActionContextValue({
    layerCount,
    actions,
    keys,
    setKeys,
    hostLayout,
    setHostLayout,
    messageManager,
    registry,
  });

  return (
    <ActionRegistryContextProvider registry={registry}>
      <KeyboardActionContext.Provider value={value}>
        {props.children}
      </KeyboardActionContext.Provider>
    </ActionRegistryContextProvider>
  );
};

export const useKeyboardActions = () => {
  const context = useContext(KeyboardActionContext);

  if (!context) {
    throw new Error(
      "useKeyboardActions must be used inside a KeyboardActionContextProvider",
    );
  }

  return context;
};

function convertFromRawHostLayout(
  rawHostLayout: HostLayout,
): Layout | undefined {
  switch (rawHostLayout) {
    case HostLayout.ENGLISH_US:
      return "english-us";
    case HostLayout.GERMAN:
      return "german";
    default:
      return undefined;
  }
}

function convertToRawHostLayout(hostLayout: Layout): HostLayout {
  switch (hostLayout) {
    case "english-us":
      return HostLayout.ENGLISH_US;
    case "german":
      return HostLayout.GERMAN;
  }
}

function resolveKeys(
  layout: GetLayoutResponse,
  entriesResponse: GetActiveActionEntriesResponse,
  hostLayout: HostLayout,
  registry: ActionRegistry,
  noneAction: KeyAction,
): RootKeyAction[] {
  const entries = new Map<number, KeyboardActionEntry>();
  const entryList = entriesResponse.entries;
  for (let i = 0; i < entryList.length; i++) {
    const entry = entryList.get(i);
    entries.set(entry.entryId, entry);
  }

  let actualHostLayout = convertFromRawHostLayout(hostLayout);
  if (actualHostLayout === undefined) {
    console.warn(`Unknown host layout "${hostLayout}".`);
    actualHostLayout = "english-us";
  }

  const resolveEntry = (entryId: number) => entries.get(entryId);

  const ctx: DecodeContext = {
    hostLayout: actualHostLayout,
    registry,
    resolveEntry: (actionEntryId) => entries.get(actionEntryId)?.action!, // TODO: remove "!"
  };

  const actionEntryIds = layout.actionEntryIds;
  const keys: RootKeyAction[] = [];
  for (let i = 0; i < actionEntryIds.length; i++) {
    const entry = resolveEntry(actionEntryIds.get(i));
    // TODO: show error instead of falling back to "None" action
    keys.push(
      entry
        ? decodeRootKeyAction(entry, ctx)
        : { kind: "single", action: noneAction },
    );
  }
  return keys;
}

/**
 * Maps a layer index through a move operation: the moved layer takes the target
 * index and the layers in between shift by one.
 */
function remapLayerIndex(index: number, from: number, to: number): number {
  if (index === from) {
    return to;
  }

  if (from < to) {
    return index > from && index <= to ? index - 1 : index;
  }

  return index >= to && index < from ? index + 1 : index;
}

/** Rewrites the layer indices referenced by an action after a layer move. */
function remapLayer(action: KeyAction, from: number, to: number): boolean {
  return action.remapLayers({
    remapLayer: (index) => remapLayerIndex(index, from, to),
  });
}
