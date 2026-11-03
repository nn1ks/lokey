/**
 * A single key action instance. Actions are identified by their protocol action
 * ID, so third-party actions only need to provide a matching `actionId`.
 */
export interface KeyAction {
  readonly actionId: number;
}

/**
 * The actions of a single key: either one action shared across all layers, or
 * one action per layer.
 */
export type RootKeyAction =
  | { kind: "single"; action: KeyAction }
  | { kind: "perLayer"; actions: KeyAction[] };
