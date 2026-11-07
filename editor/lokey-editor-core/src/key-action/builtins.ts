import { HoldTap } from "./actions/hold-tap";
import { KeyCode } from "./actions/key-code";
import { Layer } from "./actions/layer";
import { None } from "./actions/none";
import { Ordered } from "./actions/ordered";
import { Sticky } from "./actions/sticky";
import { Toggle } from "./actions/toggle";
import { Unordered } from "./actions/unordered";
import { Transparent } from "./actions/transparent";
import type { ActionRegistry } from "./registry";
import { createActionRegistry } from "./registry";

/** Creates a registry populated with the built-in action definitions. */
export function createDefaultRegistry(): ActionRegistry {
  const registry = createActionRegistry();
  registry.register(None);
  registry.register(Transparent);
  registry.register(Layer);
  registry.register(HoldTap);
  registry.register(Ordered);
  registry.register(Unordered);
  registry.register(Sticky);
  registry.register(Toggle);
  registry.register(KeyCode);
  return registry;
}

/** The default registry used by the editor. */
export const actionRegistry = createDefaultRegistry();
