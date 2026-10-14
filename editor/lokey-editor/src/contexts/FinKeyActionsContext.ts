import { createContext } from "solid-js";
import type { KeyAction, LayerId } from "lokey-editor-core";

export type FinKeyActions = Array<[LayerId, KeyAction[]]>;
export const FinKeyActionsContext = createContext<FinKeyActions | undefined>();
