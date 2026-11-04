import type { KeyboardActionEntry } from "lokey-editor-odyn/keyboard";
import type { ActionEntryId, State } from "./shared";
import { numKeys, numLayers } from "./shared";
import * as ledArrayHookId from "lokey-editor-odyn/led-array/hook-id";
import {
  HoldTap,
  KeyCode,
  Layer,
  NoAction,
  Sequence,
  Transparent,
} from "../key-action";
import type { KeyAction, RootKeyAction } from "../key-action";
import { actionRegistry } from "../key-action/builtins";
import { encodeKeyState } from "../key-action/codec/key-state";

const layer0Actions: KeyAction[] = [
  KeyCode.Q,
  KeyCode.W,
  KeyCode.F,
  KeyCode.P,
  KeyCode.B,
  KeyCode.J,
  KeyCode.L,
  KeyCode.U,
  KeyCode.Y,
  KeyCode.Slash,
  KeyCode.A,
  KeyCode.R,
  KeyCode.S,
  KeyCode.T,
  KeyCode.G,
  KeyCode.M,
  KeyCode.N,
  KeyCode.E,
  KeyCode.I,
  KeyCode.O,
  KeyCode.Z,
  KeyCode.X,
  KeyCode.C,
  KeyCode.D,
  KeyCode.V,
  KeyCode.K,
  KeyCode.H,
  KeyCode.Comma,
  KeyCode.Dot,
  KeyCode.Minus,
  new HoldTap(new Layer(1), KeyCode.Escape),
  KeyCode.LShift,
  KeyCode.Space,
  KeyCode.Enter,
  new Layer(3),
  new HoldTap(new Layer(2), KeyCode.Backspace),
];

const layer1Actions: KeyAction[] = [
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
];

const layer2Actions: KeyAction[] = [
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
];

const layer3Actions: KeyAction[] = [
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
];

const actionsPerLayer: KeyAction[][] = [
  layer0Actions,
  layer1Actions,
  layer2Actions,
  layer3Actions,
];

export function defaultState(): State {
  const keyboardActionEntries = new Map<ActionEntryId, KeyboardActionEntry>();
  const keyboardLayout = new Array<ActionEntryId>();
  let nextEntryId = 0;

  const allocateEntryId = () => nextEntryId++;
  const pushEntry = (entry: KeyboardActionEntry) => {
    keyboardActionEntries.set(entry.entryId, entry);
  };

  for (let i = 0; i < numKeys; i++) {
    const actions = Array.from(
      { length: numLayers },
      (_, layer) => actionsPerLayer[layer]?.[i] ?? new NoAction(),
    );
    const key: RootKeyAction = { kind: "perLayer", actions };
    keyboardLayout.push(
      encodeKeyState(key, {
        registry: actionRegistry,
        layout: "english-us",
        allocateEntryId,
        pushEntry,
      }),
    );
  }

  return {
    ledArrayHooks: BigInt(
      ledArrayHookId.boot |
        ledArrayHookId.blePairing |
        ledArrayHookId.bleProfile,
    ),
    keyboardActionEntries,
    keyboardLayout,
  };
}
