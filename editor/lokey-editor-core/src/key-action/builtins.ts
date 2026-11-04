import { holdTapActionDefinition } from "./actions/hold-tap";
import { keyCodeActionDefinition } from "./actions/key-code";
import { layerActionDefinition } from "./actions/layer";
import { noneActionDefinition } from "./actions/none";
import { sequenceActionDefinition } from "./actions/sequence";
import { transparentActionDefinition } from "./actions/transparent";
import type { ActionRegistry } from "./registry";
import { createActionRegistry } from "./registry";

/** Creates a registry populated with the built-in action definitions. */
export function createDefaultRegistry(): ActionRegistry {
  const registry = createActionRegistry();
  registry.register(noneActionDefinition);
  registry.register(transparentActionDefinition);
  registry.register(layerActionDefinition);
  registry.register(holdTapActionDefinition);
  registry.register(sequenceActionDefinition);
  registry.register(keyCodeActionDefinition);
  return registry;
}

/** The default registry used by the editor. */
export const actionRegistry = createDefaultRegistry();
