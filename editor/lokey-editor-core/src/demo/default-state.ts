import {
  HostLayout,
  KeyboardAction,
  type KeyboardActionEntry,
} from "lokey-editor-odyn/keyboard";
import type { ActionEntryId, State } from "./shared";
import { numKeys, numLayers } from "./shared";
import * as ledArrayHookId from "lokey-editor-odyn/led-array/hook-id";
import {
  HoldTap,
  KeyCode,
  Layer,
  None,
  Ordered,
  Sticky,
  Transparent,
} from "../key-action";
import type { KeyAction, RootKeyAction } from "../key-action";
import { encodeRootKeyAction } from "../key-action/codec/root-key-action";
import { buildActionEntry } from "../key-action/codec/build";

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
  new Ordered(KeyCode.LShift, KeyCode.Tab),
  KeyCode.LGui,
  KeyCode.LAlt,
  KeyCode.LControl,
  new None(),
  new None(),
  new None(),
  new None(),
  new None(),
  new None(),
  KeyCode.Tab,
  new Sticky(KeyCode.LGui),
  new Sticky(KeyCode.LAlt),
  new Sticky(KeyCode.LControl),
  KeyCode.Delete,
  new None(),
  KeyCode.Left,
  KeyCode.Down,
  KeyCode.Up,
  KeyCode.Right,
  new Ordered(KeyCode.LGui, KeyCode.PageUp),
  KeyCode.VolumeDown,
  KeyCode.VolumeUp,
  new Ordered(KeyCode.LGui, KeyCode.PageDown),
  KeyCode.Mute,
  new None(),
  KeyCode.Home,
  KeyCode.PageDown,
  KeyCode.PageUp,
  KeyCode.End,
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
];

const layer2Actions: KeyAction[] = [
  new None(),
  new None(),
  new None(),
  new None(),
  new None(),
  KeyCode.PrintScreen,
  new None(),
  new None(),
  KeyCode.F11,
  KeyCode.F12,
  KeyCode.N1,
  KeyCode.N2,
  KeyCode.N3,
  KeyCode.N4,
  KeyCode.N5,
  KeyCode.N6,
  KeyCode.N7,
  KeyCode.N8,
  KeyCode.N9,
  KeyCode.N0,
  KeyCode.F1,
  KeyCode.F2,
  KeyCode.F3,
  KeyCode.F4,
  KeyCode.F5,
  KeyCode.F6,
  KeyCode.F7,
  KeyCode.F8,
  KeyCode.F9,
  KeyCode.F10,
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
];

const layer3Actions: KeyAction[] = [
  KeyCode.AE,
  KeyCode.OE,
  KeyCode.UE,
  KeyCode.LeftBrace,
  KeyCode.Caret,
  KeyCode.Grave,
  KeyCode.RightBrace,
  KeyCode.ExclamationMark,
  KeyCode.QuestionMark,
  KeyCode.Degree,
  KeyCode.Asterisk,
  KeyCode.Plus,
  KeyCode.Equal,
  KeyCode.LeftParen,
  KeyCode.LessThan,
  KeyCode.GreaterThan,
  KeyCode.RightParen,
  KeyCode.Apostrophe,
  KeyCode.DoubleQuote,
  KeyCode.Dollar,
  KeyCode.SharpS,
  KeyCode.Tilde,
  KeyCode.At,
  KeyCode.LeftBracket,
  KeyCode.Hash,
  KeyCode.Percent,
  KeyCode.RightBracket,
  KeyCode.Ampersand,
  KeyCode.Pipe,
  KeyCode.Euro,
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

  const pushAction = (action: KeyboardAction) => {
    const entryId = nextEntryId;
    nextEntryId++;
    const actionEntry = buildActionEntry(entryId, action);
    keyboardActionEntries.set(entryId, actionEntry);
    return entryId;
  };

  for (let i = 0; i < numKeys; i++) {
    const actions = Array.from(
      { length: numLayers },
      (_, layer) => actionsPerLayer[layer]?.[i] ?? new None(),
    );
    const key: RootKeyAction = { kind: "perLayer", actions };
    keyboardLayout.push(
      encodeRootKeyAction(key, {
        hostLayout: "german",
        pushAction,
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
    hostLayout: HostLayout.GERMAN,
  };
}
