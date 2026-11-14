import { HostLayout, KeyboardActionEntry } from "lokey-editor-odyn/keyboard";
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
import type { KeyAction } from "../key-action";
import { KeyCodeValue } from "../key-action/key-code-value";
import { PerLayer } from "../key-action/actions/per-layer";

function createLayer0Actions(): KeyAction[] {
  return [
    new KeyCode(undefined, KeyCodeValue.Q),
    new KeyCode(undefined, KeyCodeValue.W),
    new KeyCode(undefined, KeyCodeValue.F),
    new KeyCode(undefined, KeyCodeValue.P),
    new KeyCode(undefined, KeyCodeValue.B),
    new KeyCode(undefined, KeyCodeValue.J),
    new KeyCode(undefined, KeyCodeValue.L),
    new KeyCode(undefined, KeyCodeValue.U),
    new KeyCode(undefined, KeyCodeValue.Y),
    new KeyCode(undefined, KeyCodeValue.Slash),
    new KeyCode(undefined, KeyCodeValue.A),
    new KeyCode(undefined, KeyCodeValue.R),
    new KeyCode(undefined, KeyCodeValue.S),
    new KeyCode(undefined, KeyCodeValue.T),
    new KeyCode(undefined, KeyCodeValue.G),
    new KeyCode(undefined, KeyCodeValue.M),
    new KeyCode(undefined, KeyCodeValue.N),
    new KeyCode(undefined, KeyCodeValue.E),
    new KeyCode(undefined, KeyCodeValue.I),
    new KeyCode(undefined, KeyCodeValue.O),
    new KeyCode(undefined, KeyCodeValue.Z),
    new KeyCode(undefined, KeyCodeValue.X),
    new KeyCode(undefined, KeyCodeValue.C),
    new KeyCode(undefined, KeyCodeValue.D),
    new KeyCode(undefined, KeyCodeValue.V),
    new KeyCode(undefined, KeyCodeValue.K),
    new KeyCode(undefined, KeyCodeValue.H),
    new KeyCode(undefined, KeyCodeValue.Comma),
    new KeyCode(undefined, KeyCodeValue.Dot),
    new KeyCode(undefined, KeyCodeValue.Minus),
    new HoldTap(
      undefined,
      new Layer(undefined, 1),
      new KeyCode(undefined, KeyCodeValue.Escape),
    ),
    new KeyCode(undefined, KeyCodeValue.LShift),
    new KeyCode(undefined, KeyCodeValue.Space),
    new KeyCode(undefined, KeyCodeValue.Enter),
    new Layer(undefined, 3),
    new HoldTap(
      undefined,
      new Layer(undefined, 2),
      new KeyCode(undefined, KeyCodeValue.Backspace),
    ),
  ];
}

function createLayer1Actions(): KeyAction[] {
  return [
    new Ordered(
      undefined,
      new KeyCode(undefined, KeyCodeValue.LShift),
      new KeyCode(undefined, KeyCodeValue.Tab),
    ),
    new KeyCode(undefined, KeyCodeValue.LGui),
    new KeyCode(undefined, KeyCodeValue.LAlt),
    new KeyCode(undefined, KeyCodeValue.LControl),
    new None(undefined),
    new None(undefined),
    new None(undefined),
    new None(undefined),
    new None(undefined),
    new None(undefined),
    new KeyCode(undefined, KeyCodeValue.Tab),
    new Sticky(undefined, new KeyCode(undefined, KeyCodeValue.LGui)),
    new Sticky(undefined, new KeyCode(undefined, KeyCodeValue.LAlt)),
    new Sticky(undefined, new KeyCode(undefined, KeyCodeValue.LControl)),
    new KeyCode(undefined, KeyCodeValue.Delete),
    new None(undefined),
    new KeyCode(undefined, KeyCodeValue.Left),
    new KeyCode(undefined, KeyCodeValue.Down),
    new KeyCode(undefined, KeyCodeValue.Up),
    new KeyCode(undefined, KeyCodeValue.Right),
    new Ordered(
      undefined,
      new KeyCode(undefined, KeyCodeValue.LGui),
      new KeyCode(undefined, KeyCodeValue.PageUp),
    ),
    new KeyCode(undefined, KeyCodeValue.VolumeDown),
    new KeyCode(undefined, KeyCodeValue.VolumeUp),
    new Ordered(
      undefined,
      new KeyCode(undefined, KeyCodeValue.LGui),
      new KeyCode(undefined, KeyCodeValue.PageDown),
    ),
    new KeyCode(undefined, KeyCodeValue.Mute),
    new None(undefined),
    new KeyCode(undefined, KeyCodeValue.Home),
    new KeyCode(undefined, KeyCodeValue.PageDown),
    new KeyCode(undefined, KeyCodeValue.PageUp),
    new KeyCode(undefined, KeyCodeValue.End),
    new Transparent(undefined),
    new Transparent(undefined),
    new Transparent(undefined),
    new Transparent(undefined),
    new Transparent(undefined),
    new Transparent(undefined),
  ];
}

function createLayer2Actions(): KeyAction[] {
  return [
    new None(undefined),
    new None(undefined),
    new None(undefined),
    new None(undefined),
    new None(undefined),
    new KeyCode(undefined, KeyCodeValue.PrintScreen),
    new None(undefined),
    new None(undefined),
    new KeyCode(undefined, KeyCodeValue.F11),
    new KeyCode(undefined, KeyCodeValue.F12),
    new KeyCode(undefined, KeyCodeValue.N1),
    new KeyCode(undefined, KeyCodeValue.N2),
    new KeyCode(undefined, KeyCodeValue.N3),
    new KeyCode(undefined, KeyCodeValue.N4),
    new KeyCode(undefined, KeyCodeValue.N5),
    new KeyCode(undefined, KeyCodeValue.N6),
    new KeyCode(undefined, KeyCodeValue.N7),
    new KeyCode(undefined, KeyCodeValue.N8),
    new KeyCode(undefined, KeyCodeValue.N9),
    new KeyCode(undefined, KeyCodeValue.N0),
    new KeyCode(undefined, KeyCodeValue.F1),
    new KeyCode(undefined, KeyCodeValue.F2),
    new KeyCode(undefined, KeyCodeValue.F3),
    new KeyCode(undefined, KeyCodeValue.F4),
    new KeyCode(undefined, KeyCodeValue.F5),
    new KeyCode(undefined, KeyCodeValue.F6),
    new KeyCode(undefined, KeyCodeValue.F7),
    new KeyCode(undefined, KeyCodeValue.F8),
    new KeyCode(undefined, KeyCodeValue.F9),
    new KeyCode(undefined, KeyCodeValue.F10),
    new Transparent(undefined),
    new Transparent(undefined),
    new Transparent(undefined),
    new Transparent(undefined),
    new Transparent(undefined),
    new Transparent(undefined),
  ];
}

function createLayer3Actions(): KeyAction[] {
  return [
    new KeyCode(undefined, KeyCodeValue.AE),
    new KeyCode(undefined, KeyCodeValue.OE),
    new KeyCode(undefined, KeyCodeValue.UE),
    new KeyCode(undefined, KeyCodeValue.LeftBrace),
    new KeyCode(undefined, KeyCodeValue.Caret),
    new KeyCode(undefined, KeyCodeValue.Grave),
    new KeyCode(undefined, KeyCodeValue.RightBrace),
    new KeyCode(undefined, KeyCodeValue.ExclamationMark),
    new KeyCode(undefined, KeyCodeValue.QuestionMark),
    new KeyCode(undefined, KeyCodeValue.Degree),
    new KeyCode(undefined, KeyCodeValue.Asterisk),
    new KeyCode(undefined, KeyCodeValue.Plus),
    new KeyCode(undefined, KeyCodeValue.Equal),
    new KeyCode(undefined, KeyCodeValue.LeftParen),
    new KeyCode(undefined, KeyCodeValue.LessThan),
    new KeyCode(undefined, KeyCodeValue.GreaterThan),
    new KeyCode(undefined, KeyCodeValue.RightParen),
    new KeyCode(undefined, KeyCodeValue.Apostrophe),
    new KeyCode(undefined, KeyCodeValue.DoubleQuote),
    new KeyCode(undefined, KeyCodeValue.Dollar),
    new KeyCode(undefined, KeyCodeValue.SharpS),
    new KeyCode(undefined, KeyCodeValue.Tilde),
    new KeyCode(undefined, KeyCodeValue.At),
    new KeyCode(undefined, KeyCodeValue.LeftBracket),
    new KeyCode(undefined, KeyCodeValue.Hash),
    new KeyCode(undefined, KeyCodeValue.Percent),
    new KeyCode(undefined, KeyCodeValue.RightBracket),
    new KeyCode(undefined, KeyCodeValue.Ampersand),
    new KeyCode(undefined, KeyCodeValue.Pipe),
    new KeyCode(undefined, KeyCodeValue.Euro),
    new Transparent(undefined),
    new Transparent(undefined),
    new Transparent(undefined),
    new Transparent(undefined),
    new Transparent(undefined),
    new Transparent(undefined),
  ];
}

export function defaultState(): State {
  const actionsPerLayer: KeyAction[][] = [
    createLayer0Actions(),
    createLayer1Actions(),
    createLayer2Actions(),
    createLayer3Actions(),
  ];

  const keyboardActionEntries = new Map<ActionEntryId, KeyboardActionEntry>();
  const keyboardLayout = new Array<ActionEntryId>();
  let nextEntryId = 0;

  const getNextEntryId = () => {
    const value = nextEntryId;
    nextEntryId++;
    return value;
  };

  const pushAction = (actionEntry: KeyboardActionEntry) => {
    keyboardActionEntries.set(actionEntry.entryId, actionEntry);
  };

  for (let i = 0; i < numKeys; i++) {
    const actions = Array.from(
      { length: numLayers },
      (_, layer) => actionsPerLayer[layer]?.[i] ?? new None(undefined),
    );
    const layers = Array.from({ length: numLayers }, (_, layer) => layer);
    const perLayerAction = new PerLayer(undefined, actions, layers);
    const encodedPerLayerActionEntry = perLayerAction.encode({
      hostLayout: "german",
      nextActionEntryId: getNextEntryId,
      pushActionEntry: pushAction,
    });
    keyboardActionEntries.set(
      encodedPerLayerActionEntry.entryId,
      encodedPerLayerActionEntry,
    );
    keyboardLayout.push(encodedPerLayerActionEntry.entryId);
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
