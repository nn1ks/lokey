import {
  createContext,
  createEffect,
  createMemo,
  createResource,
  createSignal,
  untrack,
  useContext,
} from "solid-js";
import type { Accessor, Component, ParentProps } from "solid-js";
import * as capnp from "capnp-es";
import {
  GetActiveActionEntriesResponse,
  GetLayoutResponse,
  HoldTapAction,
  KeyboardAction as KeyboardActionStruct,
  KeyboardActionEntry,
  LayerAction,
  NoneAction,
  PerLayerAction,
  SequenceAction,
  TransparentAction,
} from "lokey-editor-odyn/keyboard";
import * as keyboardActionId from "lokey-editor-odyn/keyboard/action-id";
import {
  HoldTap,
  KeyCode,
  Layer,
  NoAction,
  Sequence,
  Transparent,
} from "../key-action";
import type { KeyAction, RootKeyAction } from "../key-action";
import { decodeKeyState, encodeKeyCode } from "../key-action/convert";
import type { Layout } from "../key-action/convert";
import { useConnection } from "./ConnectionContext";
import { useMessageManager } from ".";
import type { MessageManager } from "../protocol/MessageManager";

const noneAction: KeyAction = new NoAction();

export class KeyboardActionContextValue {
  readonly layerCount: Accessor<number>;
  /** Actions indexed by `[keyIndex][layer]`. */
  readonly actions: Accessor<KeyAction[][]>;
  readonly #keys: Accessor<RootKeyAction[] | undefined>;
  readonly #setKeys: (keys: RootKeyAction[]) => void;
  readonly #messageManager: MessageManager;
  /** The host layout used to translate key codes. */
  readonly layout: Accessor<Layout>;
  readonly #setLayout: (layout: Layout) => void;

  constructor(options: {
    layerCount: Accessor<number>;
    actions: Accessor<KeyAction[][]>;
    keys: Accessor<RootKeyAction[] | undefined>;
    setKeys: (keys: RootKeyAction[]) => void;
    messageManager: MessageManager;
    layout: Accessor<Layout>;
    setLayout: (layout: Layout) => void;
  }) {
    this.layerCount = options.layerCount;
    this.actions = options.actions;
    this.#keys = options.keys;
    this.#setKeys = options.setKeys;
    this.#messageManager = options.messageManager;
    this.layout = options.layout;
    this.#setLayout = options.setLayout;
  }

  /**
   * Changes the host layout. The keymap is re-encoded with the new layout so
   * the same characters keep being produced, then the layout is applied.
   */
  async setLayout(layout: Layout): Promise<void> {
    const current = this.#keys();
    if (current) {
      await this.#persist(current, layout);
    }
    this.#setLayout(layout);
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

  #requireKeys(): RootKeyAction[] {
    const keys = this.#keys();
    if (!keys) {
      throw new Error("The keyboard actions have not been loaded yet.");
    }
    return keys;
  }

  async #persist(
    keys: RootKeyAction[],
    layout: Layout = this.layout(),
  ): Promise<void> {
    const entries: KeyboardActionEntry[] = [];
    let nextEntryId = 0;
    const allocate = () => nextEntryId++;

    const encode = (action: KeyAction): number => {
      if (action instanceof KeyCode) {
        const actionEntries = encodeKeyCode(action, layout, allocate);
        entries.push(
          actionEntries.rootActionEntry,
          ...actionEntries.childActionEntries,
        );
        return actionEntries.rootActionEntry.entryId;
      }

      const entryId = allocate();
      entries.push(buildActionEntry(entryId, encodeAction(action, encode)));
      return entryId;
    };

    const keyEntryIds = keys.map((key) => {
      if (key.kind === "single") {
        return encode(key.action);
      }

      const actionEntryIds = key.actions.map((action) => encode(action));
      const layers = key.actions.map((_, index) => index);
      const entryId = allocate();
      entries.push(
        buildActionEntry(entryId, buildPerLayerAction(actionEntryIds, layers)),
      );
      return entryId;
    });

    await this.#messageManager.addActionEntries(entries);
    await this.#messageManager.updateLayout(0, keyEntryIds);
  }
}

const KeyboardActionContext = createContext<KeyboardActionContextValue>();

export type KeyboardActionContextProviderProps = ParentProps<{
  layout?: Layout;
}>;

export const KeyboardActionContextProvider: Component<
  KeyboardActionContextProviderProps
> = (props) => {
  const { connection } = useConnection();
  const messageManager = useMessageManager();
  const [layout, setLayout] = createSignal<Layout>(
    props.layout ?? "english-us",
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
    if (rawLayout.state !== "ready" && rawLayout.state !== "refreshing") {
      return;
    }
    if (rawEntries.state !== "ready" && rawEntries.state !== "refreshing") {
      return;
    }

    setKeys(resolveKeys(rawLayout(), rawEntries(), untrack(layout)));
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
    setKeys: (next) => setKeys(next),
    messageManager,
    layout,
    setLayout,
  });

  return (
    <KeyboardActionContext.Provider value={value}>
      {props.children}
    </KeyboardActionContext.Provider>
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

function resolveKeys(
  layout: GetLayoutResponse,
  entriesResponse: GetActiveActionEntriesResponse,
  keyCodeLayout: Layout,
): RootKeyAction[] {
  const entries = new Map<number, KeyboardActionEntry>();
  const entryList = entriesResponse.entries;
  for (let i = 0; i < entryList.length; i++) {
    const entry = entryList.get(i);
    entries.set(entry.entryId, entry);
  }

  const resolveEntry = (entryId: number) => entries.get(entryId);

  const actionEntryIds = layout.actionEntryIds;
  const keys: RootKeyAction[] = [];
  for (let i = 0; i < actionEntryIds.length; i++) {
    const entry = resolveEntry(actionEntryIds.get(i));
    keys.push(
      entry
        ? decodeKeyState(entry, resolveEntry, keyCodeLayout)
        : { kind: "single", action: noneAction },
    );
  }
  return keys;
}

function encodeAction(
  action: KeyAction,
  encodeChild: (action: KeyAction) => number,
): KeyboardActionStruct {
  if (action instanceof NoAction) {
    return buildAction(keyboardActionId.none, NoneAction, () => {});
  }

  if (action instanceof Transparent) {
    return buildAction(
      keyboardActionId.transparent,
      TransparentAction,
      () => {},
    );
  }

  if (action instanceof Layer) {
    return buildAction(keyboardActionId.layer, LayerAction, (layerAction) => {
      layerAction.layer = action.index;
    });
  }

  if (action instanceof HoldTap) {
    const holdActionEntryId = encodeChild(action.holdAction);
    const tapActionEntryId = encodeChild(action.tapAction);

    return buildAction(
      keyboardActionId.holdTap,
      HoldTapAction,
      (holdTapAction) => {
        holdTapAction.holdActionEntryId = holdActionEntryId;
        holdTapAction.tapActionEntryId = tapActionEntryId;
        holdTapAction.tappingTerm = action.tappingTerm;
      },
    );
  }

  if (action instanceof Sequence) {
    const action1EntryId = encodeChild(action.action1);
    const action2EntryId = encodeChild(action.action2);

    return buildAction(
      keyboardActionId.sequence,
      SequenceAction,
      (sequenceAction) => {
        sequenceAction.action1EntryId = action1EntryId;
        sequenceAction.action2EntryId = action2EntryId;
      },
    );
  }

  throw new Error("Unsupported keyboard action.");
}

function buildAction<T extends capnp.Struct>(
  actionId: number,
  actionType: capnp.StructCtor<T>,
  setAction: (action: T) => void,
): KeyboardActionStruct {
  const capnpMessage = new capnp.Message();
  const actionWrapper = capnpMessage.initRoot(KeyboardActionStruct);
  actionWrapper.actionId = actionId;

  const capnpMessageInner = new capnp.Message();
  const action = capnpMessageInner.initRoot(actionType);
  setAction(action);
  const serializedAction = capnpMessageInner.toArrayBuffer();
  actionWrapper
    ._initPayload(serializedAction.byteLength)
    .copyBuffer(serializedAction);

  return actionWrapper;
}

function buildPerLayerAction(
  actionEntryIds: number[],
  layers: number[],
): KeyboardActionStruct {
  if (actionEntryIds.length !== layers.length) {
    throw new Error(
      `A per-layer action requires the same number of entry IDs and layers, got ${actionEntryIds.length} and ${layers.length}.`,
    );
  }

  return buildAction(keyboardActionId.perLayer, PerLayerAction, (action) => {
    const ids = action._initActionEntryIds(actionEntryIds.length);
    actionEntryIds.forEach((entryId, index) => ids.set(index, entryId));
    const layerList = action._initLayers(layers.length);
    layers.forEach((layer, index) => layerList.set(index, layer));
  });
}

function buildActionEntry(
  entryId: number,
  action: KeyboardActionStruct,
): KeyboardActionEntry {
  const capnpMessage = new capnp.Message();
  const entry = capnpMessage.initRoot(KeyboardActionEntry);
  entry.entryId = entryId;
  entry.action = action;
  return entry;
}
