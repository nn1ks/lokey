import type { JSX } from "solid-js";
import type { Layout } from "../layouts";
import type { KeyAction } from "./action";
import type { ActionEntries } from "./codec/entries";

export interface EncodeContext {
  /** The host layout used to translate key codes. */
  readonly layout: Layout;
  /** Encodes a child action and returns its entry ID. */
  encodeChild(action: KeyAction): number;
  /** Allocates a new action entry ID. */
  allocateEntryId(): number;
}

export interface DecodeContext {
  /** The host layout used to translate key codes. */
  readonly layout: Layout;
  /** Resolves an action entry ID to its decoded action. */
  decodeChild(entryId: number): KeyAction;
}

export interface RemapContext {
  /** Maps a layer index through a layer move. */
  remapLayer(index: number): number;
  /** Recursively remaps a child action. */
  remapAction(action: KeyAction): KeyAction;
}

export interface RenderContext {
  /** Renders a child action at the given nesting level. */
  renderAction(action: KeyAction, level: number): JSX.Element;
  /** Renders a child action's tooltip. */
  renderTooltip(action: KeyAction): JSX.Element;
}

export interface ActionDefinition<A extends KeyAction> {
  /** The protocol action ID. Must be unique across all registered actions. */
  readonly actionId: number;
  /** Human-readable name for action pickers. */
  readonly label: string;

  encode(action: A, ctx: EncodeContext): ActionEntries;
  decode(payload: Uint8Array, ctx: DecodeContext): KeyAction;

  render(action: A, level: number, ctx: RenderContext): JSX.Element;
  renderTooltip(action: A, ctx: RenderContext): JSX.Element;

  remap(action: A, ctx: RemapContext): A;
  children(action: A): KeyAction[];
}

/** Identity helper that preserves the concrete action type. */
export function defineAction<A extends KeyAction>(
  definition: ActionDefinition<A>,
): ActionDefinition<A> {
  return definition;
}
