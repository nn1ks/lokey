export { None, noneActionDefinition } from "./none";
export { Transparent, transparentActionDefinition } from "./transparent";
export { Layer, layerActionDefinition } from "./layer";
export { HoldTap, holdTapActionDefinition } from "./hold-tap";
export { Sequence, sequenceActionDefinition } from "./sequence";
export { KeyCode, keyCodeActionDefinition } from "./key-code";

/** The built-in action types, for code that needs exhaustive narrowing. */
export type KnownKeyAction =
  NoAction | Transparent | Layer | HoldTap | Sequence | KeyCode;
