import * as capnp from "capnp-es";
import {
  GetActiveActionEntriesResponse,
  GetLayoutResponse,
  HostLayout,
  KeyboardActionEntry,
  PerLayerAction,
} from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import { KeyAction } from "./action";
import { RemapLayersContext } from "./contexts";
import { createStore, SetStoreFunction } from "solid-js/store";
import { Accessor, batch, createSignal, Setter } from "solid-js";
import { Layout } from "../layouts";
import { None } from "./actions/none";
import { PerLayer } from "./actions/per-layer";
import { ActionRegistry } from "./registry";
import { decodeActionEntry } from "./codec/decode";
import { MessageManager } from "../protocol";

type ActionEntryId = number;

export class KeymapState {
  readonly #messageManager: MessageManager;
  readonly #registry: ActionRegistry;

  readonly hostLayout: Accessor<Layout>;
  readonly #setHostLayout: Setter<Layout>;

  readonly layerCount: Accessor<number>;
  readonly #setLayerCount: Setter<number>;

  readonly rootActionEntryIds: ActionEntryId[]; // must be entry IDs of PerLayer actions
  readonly #setRootActionEntryIds: SetStoreFunction<ActionEntryId[]>;

  readonly encodedActionEntries: Record<ActionEntryId, KeyboardActionEntry>;
  readonly #setEncodedActionEntries: SetStoreFunction<
    Record<ActionEntryId, KeyboardActionEntry>
  >;

  readonly keyActions: Record<ActionEntryId, KeyAction>;
  readonly #setKeyActions: SetStoreFunction<Record<ActionEntryId, KeyAction>>;

  #nextEntryId: number;

  constructor(
    messageManager: MessageManager,
    registry: ActionRegistry,
    hostLayout: Layout,
    layerCount: number,
    rootActionEntryIds: number[],
    encodedActionEntries: Record<ActionEntryId, KeyboardActionEntry>,
    keyActions: Record<ActionEntryId, KeyAction>,
  ) {
    this.#messageManager = messageManager;
    this.#registry = registry;

    const [localHostLayout, localSetHostLayout] = createSignal(hostLayout);
    this.hostLayout = localHostLayout;
    this.#setHostLayout = localSetHostLayout;

    const [localLayerCount, localSetLayerCount] = createSignal(layerCount);
    this.layerCount = localLayerCount;
    this.#setLayerCount = localSetLayerCount;

    const [localRootActionEntryIds, localSetRootActionEntryIds] =
      createStore(rootActionEntryIds);
    this.rootActionEntryIds = localRootActionEntryIds;
    this.#setRootActionEntryIds = localSetRootActionEntryIds;

    const [localEncodedActionEntries, localSetEncodedActionEntries] =
      createStore(encodedActionEntries);
    this.encodedActionEntries = localEncodedActionEntries;
    this.#setEncodedActionEntries = localSetEncodedActionEntries;

    const [localKeyActions, localSetKeyActions] = createStore(keyActions);
    this.keyActions = localKeyActions;
    this.#setKeyActions = localSetKeyActions;

    if (Object.entries(this.encodedActionEntries).length === 0) {
      this.#nextEntryId = 0;
    } else {
      this.#nextEntryId =
        Math.max(
          ...Object.keys(this.encodedActionEntries).map((x) => Number(x)),
        ) + 1;
    }
  }

  static fromResponses(
    messageManager: MessageManager,
    registry: ActionRegistry,
    hostLayout: HostLayout,
    activeActionEntriesResponses: GetActiveActionEntriesResponse[],
    layoutResponses: GetLayoutResponse[],
  ): KeymapState {
    const hostLayoutValue = convertFromRawHostLayout(hostLayout);
    if (!hostLayoutValue) {
      throw new Error("Received invalid host layout");
    }

    const encodedActionEntries: Record<ActionEntryId, KeyboardActionEntry> = {};
    for (const response of activeActionEntriesResponses) {
      for (const actionEntry of response.entries) {
        encodedActionEntries[actionEntry.entryId] = actionEntry;
      }
    }

    const rootActionEntryIds = [];
    for (const response of layoutResponses) {
      // TODO: use response.startIndex
      for (const entryId of response.actionEntryIds) {
        rootActionEntryIds.push(entryId);
      }
    }

    const layerCount = checkRootActionsAndGetLayerCount(
      registry,
      hostLayoutValue,
      rootActionEntryIds,
      encodedActionEntries,
    );

    const keyActions = decodeActionEntries(
      registry,
      hostLayoutValue,
      encodedActionEntries,
    );

    return new KeymapState(
      messageManager,
      registry,
      hostLayoutValue,
      layerCount,
      rootActionEntryIds,
      encodedActionEntries,
      keyActions,
    );
  }

  async setHostLayout(hostLayout: Layout): Promise<void> {
    this.#setHostLayout(hostLayout);
    await this.#messageManager.setHostLayout(
      convertToRawHostLayout(hostLayout),
    );
    await this.#updateChangedActions();
  }

  #decodeActionEntry(actionEntry: KeyboardActionEntry): KeyAction {
    return decodeActionEntry(actionEntry, {
      hostLayout: this.hostLayout(),
      registry: this.#registry,
      resolveEntry: (actionEntryId) =>
        this.encodedActionEntries[actionEntryId]!.action, // TODO: change "!" to "?"
    });
  }

  #getPerLayerActionAt(keyIndex: number): PerLayer {
    const entryId = this.rootActionEntryIds[keyIndex];
    if (entryId === undefined) {
      throw new Error(`No action at key index ${keyIndex}`);
    }
    const perLayerAction = this.keyActions[entryId];
    if (perLayerAction === undefined) {
      throw new Error(`No key action with entry ID ${entryId} found`);
    } else if (perLayerAction instanceof PerLayer) {
      return perLayerAction;
    } else {
      throw new Error(
        `Action at key index ${keyIndex} is not a PerLayer action`,
      );
    }
  }

  #rootActions(): PerLayer[] {
    return this.rootActionEntryIds.map((rootEntryId) => {
      const rootAction = this.keyActions[rootEntryId];
      if (!(rootAction instanceof PerLayer)) {
        throw new Error(
          `Root action with entry ID ${rootEntryId} is not a PerLayer action`,
        );
      }
      return rootAction;
    });
  }

  actionEntryIdAt(layer: number, keyIndex: number): number {
    const rootEntryId = this.rootActionEntryIds[keyIndex];
    if (rootEntryId === undefined) {
      throw new Error(`No action at key index ${keyIndex}`);
    }
    const rootEncodedActionEntry = this.encodedActionEntries[rootEntryId];
    if (rootEncodedActionEntry === undefined) {
      throw new Error(`No encoded action with entry ID ${rootEntryId} found`);
    }
    const message = new capnp.Message(
      new Uint8Array(rootEncodedActionEntry.action.payload.toArrayBuffer()),
      false,
    );
    const perLayer = message.getRoot(PerLayerAction);
    const index = perLayer.layers.toArray().indexOf(layer);
    return perLayer.actionEntryIds[index]!;
  }

  actionAt(layer: number, keyIndex: number): KeyAction {
    const perLayerAction = this.#getPerLayerActionAt(keyIndex);
    const action = perLayerAction.getActionOfLayer(layer);
    if (!action) {
      throw new Error(`No action at layer ${layer}`);
    }
    return action;
  }

  #actionEntryAlreadyExists(actionEntry: KeyboardActionEntry): boolean {
    const existing = this.encodedActionEntries[actionEntry.entryId];
    return existing !== undefined && actionEntriesEqual(existing, actionEntry);
  }

  async pushActionAt(action: KeyAction, layer: number, keyIndex: number) {
    const currentAction = this.actionAt(layer, keyIndex);
    action.actionEntryId = currentAction.actionEntryId;

    await this.pushAction(action);

    if (!Object.is(action, currentAction)) {
      this.#getPerLayerActionAt(keyIndex).setActionOfLayer(layer, action);
    }
  }

  async pushAction(action: KeyAction): Promise<void> {
    const pending: Promise<void>[] = [];

    const getNextEntryId = () => {
      const newEntryId = this.#nextEntryId;
      this.#nextEntryId++;
      return newEntryId;
    };

    const pushActionEntry = (actionEntry: KeyboardActionEntry) => {
      if (!this.#actionEntryAlreadyExists(actionEntry)) {
        const action = this.#decodeActionEntry(actionEntry);
        this.#setKeyActions(actionEntry.entryId, action);
        this.#setEncodedActionEntries(actionEntry.entryId, actionEntry);
        pending.push(this.#messageManager.addActionEntries([actionEntry]));
      }
    };

    batch(() => {
      const actionEntry = action.encode({
        hostLayout: this.hostLayout(),
        nextActionEntryId: getNextEntryId,
        pushActionEntry,
      });

      if (!this.#actionEntryAlreadyExists(actionEntry)) {
        this.#setKeyActions(actionEntry.entryId, action);
        this.#setEncodedActionEntries(actionEntry.entryId, actionEntry);
        pending.push(this.#messageManager.addActionEntries([actionEntry]));
      }
    });

    await Promise.all(pending);
  }

  async #updateChangedActions() {
    const pending: Promise<void>[] = [];

    const getNextEntryId = () => {
      const newEntryId = this.#nextEntryId;
      this.#nextEntryId++;
      return newEntryId;
    };

    const pushActionEntry = (actionEntry: KeyboardActionEntry) => {
      if (!this.#actionEntryAlreadyExists(actionEntry)) {
        this.#setEncodedActionEntries(actionEntry.entryId, actionEntry);
        pending.push(this.#messageManager.addActionEntries([actionEntry]));
      }
    };

    batch(() => {
      for (const action of Object.values(this.keyActions)) {
        const actionEntry = action.encode({
          hostLayout: this.hostLayout(),
          nextActionEntryId: getNextEntryId,
          pushActionEntry,
        });
        if (!this.#actionEntryAlreadyExists(actionEntry)) {
          this.#setEncodedActionEntries(actionEntry.entryId, actionEntry);
          pending.push(this.#messageManager.addActionEntries([actionEntry]));
        }
      }
    });

    await Promise.all(pending);
  }

  async moveLayer(from: number, to: number): Promise<void> {
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

    // Batch the mutations so consumers never observe a mix of old and new
    // layer orders across keys.
    batch(() => {
      const rootActions = this.#rootActions();

      // Reorder the child actions of each root PerLayer action to reflect the
      // move: the action at `from` moves to `to`, and the layers in between
      // shift by one.
      for (const rootAction of rootActions) {
        const reordered = rootAction.actions
          .map(([layer, action]): [number, KeyAction] => [
            remapLayerIndex(layer, from, to),
            action,
          ])
          .sort(([a], [b]) => a - b);

        rootAction.setActions(reordered);
      }

      // Remap layer references inside all root actions (recursively).
      const remapContext: RemapLayersContext = {
        convertLayer: (index) => remapLayerIndex(index, from, to),
      };
      for (const rootAction of rootActions) {
        rootAction.remapLayers(remapContext);
      }
    });

    // Re-encode all actions and send the ones whose encoding changed.
    await this.#updateChangedActions();
  }

  async removeLayer(layer: number): Promise<void> {
    const layerCount = this.layerCount();
    if (layer < 0 || layer >= layerCount) {
      throw new Error(
        `Layer ${layer} is out of range, expected 0..${layerCount - 1}.`,
      );
    }
    if (layerCount <= 1) {
      throw new Error("Cannot remove the only remaining layer.");
    }

    // Batch the mutations so consumers never observe the intermediate state
    // where the layer is already gone but `layerCount` still includes it.
    batch(() => {
      const rootActions = this.#rootActions();

      // Remove the child action at `layer` from each root PerLayer action and
      // shift the layers after it down by one.
      for (const rootAction of rootActions) {
        const remaining = rootAction.actions
          .filter(([currentLayer]) => currentLayer !== layer)
          .map(([currentLayer, action]): [number, KeyAction] => [
            currentLayer > layer ? currentLayer - 1 : currentLayer,
            action,
          ])
          .sort(([a], [b]) => a - b);

        rootAction.setActions(remaining);
      }

      // Remap layer references inside all root actions (recursively).
      const remapContext: RemapLayersContext = {
        convertLayer: (index) => remapLayerIndexAfterRemoval(index, layer),
      };
      for (const rootAction of rootActions) {
        rootAction.remapLayers(remapContext);
      }

      this.#setLayerCount(layerCount - 1);
    });

    // Re-encode all actions and send the ones whose encoding changed.
    await this.#updateChangedActions();
  }

  async addLayer(): Promise<void> {
    const layerCount = this.layerCount();

    // Batch the mutations so the new layer and the incremented layer count
    // become visible together.
    batch(() => {
      // Append a new layer with a `None` action to every root PerLayer action.
      for (const rootAction of this.#rootActions()) {
        rootAction.setActions(rootAction.actions.length, [
          layerCount,
          new None(undefined),
        ]);
      }

      this.#setLayerCount(layerCount + 1);
    });

    // Re-encode all actions and send the ones whose encoding changed.
    await this.#updateChangedActions();
  }
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

/**
 * Maps a layer index after a layer has been removed. Layers after the removed
 * layer shift down by one.
 */
function remapLayerIndexAfterRemoval(index: number, removed: number): number {
  return index > removed ? index - 1 : index;
}

function actionEntriesEqual(
  a: KeyboardActionEntry,
  b: KeyboardActionEntry,
): boolean {
  if (a.entryId !== b.entryId) {
    return false;
  }
  if (a.action.actionId !== b.action.actionId) {
    return false;
  }

  const aPayload = new Uint8Array(a.action.payload.toArrayBuffer());
  const bPayload = new Uint8Array(b.action.payload.toArrayBuffer());
  if (aPayload.length !== bPayload.length) {
    return false;
  }

  for (let i = 0; i < aPayload.length; i++) {
    if (aPayload[i] !== bPayload[i]) {
      return false;
    }
  }

  return true;
}

function checkRootActionsAndGetLayerCount(
  registry: ActionRegistry,
  hostLayout: Layout,
  rootActionEntryIds: ActionEntryId[],
  encodedActionEntries: Record<ActionEntryId, KeyboardActionEntry>,
): number {
  let layerCount = undefined;

  for (const rootActionEntryId of rootActionEntryIds) {
    const rootActionEntry = encodedActionEntries[rootActionEntryId];
    if (!rootActionEntry) {
      throw new Error(
        `Root action with entry ID ${rootActionEntryId} does not exist`,
      );
    }
    if (rootActionEntry.action.actionId === ActionId.perLayer) {
      const payload = new Uint8Array(
        rootActionEntry.action.payload.toArrayBuffer(),
      );
      const perLayer = PerLayer.decode(rootActionEntryId, payload, {
        hostLayout,
        registry,
        resolveEntry: (actionEntryId) =>
          encodedActionEntries[actionEntryId]!.action, // TODO: change "!" to "?"
      });
      if (layerCount === undefined) {
        layerCount = perLayer.actions.length;
      } else if (perLayer.actions.length !== layerCount) {
        throw new Error(
          `PerLayer root action with entry ID ${rootActionEntryId} does not have ${layerCount} layers`,
        );
      }
    } else {
      throw new Error(
        `Root action with entry ID ${rootActionEntryId} is not a PerLayer action`,
      );
    }
  }

  return layerCount ?? 0;
}

function decodeActionEntries(
  registry: ActionRegistry,
  hostLayout: Layout,
  encodedActionEntries: Record<ActionEntryId, KeyboardActionEntry>,
): Record<ActionEntryId, KeyAction> {
  const keyActions: Record<ActionEntryId, KeyAction> = {};
  for (const [entryIdString, entry] of Object.entries(encodedActionEntries)) {
    const entryId = Number(entryIdString);
    keyActions[entryId] = decodeActionEntry(entry, {
      hostLayout,
      registry,
      resolveEntry: (actionEntryId) =>
        encodedActionEntries[actionEntryId]!.action, // TODO: replace "!" with "?"
    });
  }
  return keyActions;
}

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
