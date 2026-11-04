import * as capnp from "capnp-es";
import {
  ArrowBigUp,
  ArrowBigUpDash,
  ArrowDown,
  ArrowLeft,
  ArrowRight,
  ArrowRightToLine,
  ArrowUp,
  Command,
  CornerDownLeft,
  Delete,
  Option,
  Power,
  Space,
  Space as SpaceIcon,
  Volume1,
  Volume2,
  VolumeX,
} from "lucide-solid";
import {
  KeyCodeAction,
  KeyCodeType,
  OrderedAction,
} from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import type { Layout } from "../../layouts";
import type { KeyAction } from "../action";
import type { ActionEntries } from "../codec/entries";
import { buildAction, buildActionEntry } from "../codec/entries";
import { defineAction } from "../definition";
import { englishUsKeyCodes } from "../layouts/english-us";
import { germanKeyCodes } from "../layouts/german";
import type { KeyCodeEncoding } from "../layouts/types";
import {
  IconSubtitleCombination,
  Small,
  SmallTextSubtitleCombination,
  TextSubtitleCombination,
} from "../render";

export class KeyCode implements KeyAction {
  readonly actionId = ActionId.keyCode;

  private constructor(public readonly value: string) {}

  displayValue(): string {
    const displayValues: Record<string, string> = {
      LControl: "Left Control",
      RControl: "Right Control",
      LShift: "Left Shift",
      RShift: "Right Shift",
      LAlt: "Left Alt",
      RAlt: "Right Alt",
      LGui: "Left Gui",
      RGui: "Right Gui",
      N1: "1",
      N2: "2",
      N3: "3",
      N4: "4",
      N5: "5",
      N6: "6",
      N7: "7",
      N8: "8",
      N9: "9",
      N0: "0",
      Minus: "-",
      Equal: "=",
      LeftBracket: "[",
      RightBracket: "]",
      Backslash: "\\",
      Hash: "#",
      Semicolon: ";",
      Apostrophe: "'",
      Grave: "`",
      Comma: ",",
      Dot: ".",
      Slash: "/",
    };

    return displayValues[this.value] ?? this.value;
  }

  tooltipValue(): string | null {
    switch (this.value) {
      case "Minus":
        return "Minus";
      case "Equal":
        return "Equal";
      case "LeftBracket":
        return "Left Bracket";
      case "RightBracket":
        return "Right Bracket";
      case "Backslash":
        return "Backslash";
      case "Hash":
        return "Hash";
      case "Semicolon":
        return "Semicolon";
      case "Apostrophe":
        return "Apostrophe";
      case "Grave":
        return "Grave";
      case "Comma":
        return "Comma";
      case "Dot":
        return "Dot";
      case "Slash":
        return "Slash";
      default:
        return null;
    }
  }

  toString(): string {
    return this.displayValue();
  }

  static readonly LControl = new KeyCode("LControl");
  static readonly RControl = new KeyCode("RControl");
  static readonly LShift = new KeyCode("LShift");
  static readonly RShift = new KeyCode("RShift");
  static readonly LAlt = new KeyCode("LAlt");
  static readonly RAlt = new KeyCode("RAlt");
  static readonly LGui = new KeyCode("LGui");
  static readonly RGui = new KeyCode("RGui");
  static readonly A = new KeyCode("A");
  static readonly B = new KeyCode("B");
  static readonly C = new KeyCode("C");
  static readonly D = new KeyCode("D");
  static readonly E = new KeyCode("E");
  static readonly F = new KeyCode("F");
  static readonly G = new KeyCode("G");
  static readonly H = new KeyCode("H");
  static readonly I = new KeyCode("I");
  static readonly J = new KeyCode("J");
  static readonly K = new KeyCode("K");
  static readonly L = new KeyCode("L");
  static readonly M = new KeyCode("M");
  static readonly N = new KeyCode("N");
  static readonly O = new KeyCode("O");
  static readonly P = new KeyCode("P");
  static readonly Q = new KeyCode("Q");
  static readonly R = new KeyCode("R");
  static readonly S = new KeyCode("S");
  static readonly T = new KeyCode("T");
  static readonly U = new KeyCode("U");
  static readonly V = new KeyCode("V");
  static readonly W = new KeyCode("W");
  static readonly X = new KeyCode("X");
  static readonly Y = new KeyCode("Y");
  static readonly Z = new KeyCode("Z");
  static readonly N1 = new KeyCode("N1");
  static readonly N2 = new KeyCode("N2");
  static readonly N3 = new KeyCode("N3");
  static readonly N4 = new KeyCode("N4");
  static readonly N5 = new KeyCode("N5");
  static readonly N6 = new KeyCode("N6");
  static readonly N7 = new KeyCode("N7");
  static readonly N8 = new KeyCode("N8");
  static readonly N9 = new KeyCode("N9");
  static readonly N0 = new KeyCode("N0");
  static readonly Enter = new KeyCode("Enter");
  static readonly Escape = new KeyCode("Escape");
  static readonly Backspace = new KeyCode("Backspace");
  static readonly Tab = new KeyCode("Tab");
  static readonly Space = new KeyCode("Space");
  static readonly Minus = new KeyCode("Minus");
  static readonly Equal = new KeyCode("Equal");
  static readonly LeftBracket = new KeyCode("LeftBracket");
  static readonly RightBracket = new KeyCode("RightBracket");
  static readonly Backslash = new KeyCode("Backslash");
  static readonly Hash = new KeyCode("Hash");
  static readonly Semicolon = new KeyCode("Semicolon");
  static readonly Apostrophe = new KeyCode("Apostrophe");
  static readonly Grave = new KeyCode("Grave");
  static readonly Comma = new KeyCode("Comma");
  static readonly Dot = new KeyCode("Dot");
  static readonly Slash = new KeyCode("Slash");
  static readonly CapsLock = new KeyCode("CapsLock");
  static readonly PrintScreen = new KeyCode("PrintScreen");
  static readonly ScrollLock = new KeyCode("ScrollLock");
  static readonly Pause = new KeyCode("Pause");
  static readonly Insert = new KeyCode("Insert");
  static readonly Home = new KeyCode("Home");
  static readonly PageUp = new KeyCode("PageUp");
  static readonly Delete = new KeyCode("Delete");
  static readonly End = new KeyCode("End");
  static readonly PageDown = new KeyCode("PageDown");
  static readonly Right = new KeyCode("Right");
  static readonly Left = new KeyCode("Left");
  static readonly Down = new KeyCode("Down");
  static readonly Up = new KeyCode("Up");
  static readonly F1 = new KeyCode("F1");
  static readonly F2 = new KeyCode("F2");
  static readonly F3 = new KeyCode("F3");
  static readonly F4 = new KeyCode("F4");
  static readonly F5 = new KeyCode("F5");
  static readonly F6 = new KeyCode("F6");
  static readonly F7 = new KeyCode("F7");
  static readonly F8 = new KeyCode("F8");
  static readonly F9 = new KeyCode("F9");
  static readonly F10 = new KeyCode("F10");
  static readonly F11 = new KeyCode("F11");
  static readonly F12 = new KeyCode("F12");
  static readonly F13 = new KeyCode("F13");
  static readonly F14 = new KeyCode("F14");
  static readonly F15 = new KeyCode("F15");
  static readonly F16 = new KeyCode("F16");
  static readonly F17 = new KeyCode("F17");
  static readonly F18 = new KeyCode("F18");
  static readonly F19 = new KeyCode("F19");
  static readonly F20 = new KeyCode("F20");
  static readonly F21 = new KeyCode("F21");
  static readonly F22 = new KeyCode("F22");
  static readonly F23 = new KeyCode("F23");
  static readonly F24 = new KeyCode("F24");
  static readonly NumLock = new KeyCode("NumLock");
  static readonly KpSlash = new KeyCode("KpSlash");
  static readonly KpAsterisk = new KeyCode("KpAsterisk");
  static readonly KpMinus = new KeyCode("KpMinus");
  static readonly KpPlus = new KeyCode("KpPlus");
  static readonly KpEnter = new KeyCode("KpEnter");
  static readonly KpDot = new KeyCode("KpDot");
  static readonly KpEqual = new KeyCode("KpEqual");
  static readonly Kp1 = new KeyCode("Kp1");
  static readonly Kp2 = new KeyCode("Kp2");
  static readonly Kp3 = new KeyCode("Kp3");
  static readonly Kp4 = new KeyCode("Kp4");
  static readonly Kp5 = new KeyCode("Kp5");
  static readonly Kp6 = new KeyCode("Kp6");
  static readonly Kp7 = new KeyCode("Kp7");
  static readonly Kp8 = new KeyCode("Kp8");
  static readonly Kp9 = new KeyCode("Kp9");
  static readonly Kp0 = new KeyCode("Kp0");
  static readonly NonUsBackslash = new KeyCode("NonUsBackslash");
  static readonly Application = new KeyCode("Application");
  static readonly Power = new KeyCode("Power");
  static readonly Execute = new KeyCode("Execute");
  static readonly Help = new KeyCode("Help");
  static readonly Menu = new KeyCode("Menu");
  static readonly Select = new KeyCode("Select");
  static readonly Stop = new KeyCode("Stop");
  static readonly Again = new KeyCode("Again");
  static readonly Undo = new KeyCode("Undo");
  static readonly Cut = new KeyCode("Cut");
  static readonly Copy = new KeyCode("Copy");
  static readonly Paste = new KeyCode("Paste");
  static readonly Find = new KeyCode("Find");
  static readonly Mute = new KeyCode("Mute");
  static readonly VolumeUp = new KeyCode("VolumeUp");
  static readonly VolumeDown = new KeyCode("VolumeDown");
  static readonly LockingCapsLock = new KeyCode("LockingCapsLock");
  static readonly LockingNumLock = new KeyCode("LockingNumLock");
  static readonly LockingScrollLock = new KeyCode("LockingScrollLock");
  static readonly KpComma = new KeyCode("KpComma");
  static readonly KpEqualSign = new KeyCode("KpEqualSign");
  static readonly International1 = new KeyCode("International1");
  static readonly International2 = new KeyCode("International2");
  static readonly International3 = new KeyCode("International3");
  static readonly International4 = new KeyCode("International4");
  static readonly International5 = new KeyCode("International5");
  static readonly International6 = new KeyCode("International6");
  static readonly International7 = new KeyCode("International7");
  static readonly International8 = new KeyCode("International8");
  static readonly International9 = new KeyCode("International9");
  static readonly Lang1 = new KeyCode("Lang1");
  static readonly Lang2 = new KeyCode("Lang2");
  static readonly Lang3 = new KeyCode("Lang3");
  static readonly Lang4 = new KeyCode("Lang4");
  static readonly Lang5 = new KeyCode("Lang5");
  static readonly Lang6 = new KeyCode("Lang6");
  static readonly Lang7 = new KeyCode("Lang7");
  static readonly Lang8 = new KeyCode("Lang8");
  static readonly Lang9 = new KeyCode("Lang9");
  static readonly Kp00 = new KeyCode("Kp00");
  static readonly Kp000 = new KeyCode("Kp000");
  static readonly KpThousandsSeparator = new KeyCode("KpThousandsSeparator");
  static readonly KpDecimalSeparator = new KeyCode("KpDecimalSeparator");
  static readonly KpCurrencyUnit = new KeyCode("KpCurrencyUnit");
  static readonly KpCurrencySubunit = new KeyCode("KpCurrencySubunit");
  static readonly KpLeftParen = new KeyCode("KpLeftParen");
  static readonly KpRightParen = new KeyCode("KpRightParen");
  static readonly KpLeftBrace = new KeyCode("KpLeftBrace");
  static readonly KpRightBrace = new KeyCode("KpRightBrace");
  static readonly KpTab = new KeyCode("KpTab");
  static readonly KpBackspace = new KeyCode("KpBackspace");
  static readonly KpA = new KeyCode("KpA");
  static readonly KpB = new KeyCode("KpB");
  static readonly KpC = new KeyCode("KpC");
  static readonly KpD = new KeyCode("KpD");
  static readonly KpE = new KeyCode("KpE");
  static readonly KpF = new KeyCode("KpF");
  static readonly KpXor = new KeyCode("KpXor");
  static readonly KpCaret = new KeyCode("KpCaret");
  static readonly KpPercent = new KeyCode("KpPercent");
  static readonly KpLessThan = new KeyCode("KpLessThan");
  static readonly KpGreaterThan = new KeyCode("KpGreaterThan");
  static readonly KpAmpersand = new KeyCode("KpAmpersand");
  static readonly KpDoubleAmpersand = new KeyCode("KpDoubleAmpersand");
  static readonly KpPipe = new KeyCode("KpPipe");
  static readonly KpDoublePipe = new KeyCode("KpDoublePipe");
  static readonly KpColon = new KeyCode("KpColon");
  static readonly KpHash = new KeyCode("KpHash");
  static readonly KpSpace = new KeyCode("KpSpace");
  static readonly KpAt = new KeyCode("KpAt");
  static readonly KpExclamation = new KeyCode("KpExclamation");
  static readonly KpMemoryStore = new KeyCode("KpMemoryStore");
  static readonly KpMemoryRecall = new KeyCode("KpMemoryRecall");
  static readonly KpMemoryClear = new KeyCode("KpMemoryClear");
  static readonly KpMemoryAdd = new KeyCode("KpMemoryAdd");
  static readonly KpMemorySubtract = new KeyCode("KpMemorySubtract");
  static readonly KpMemoryMultiply = new KeyCode("KpMemoryMultiply");
  static readonly KpMemoryDivide = new KeyCode("KpMemoryDivide");
  static readonly KpPlusMinus = new KeyCode("KpPlusMinus");
  static readonly KpClear = new KeyCode("KpClear");
  static readonly KpClearEntry = new KeyCode("KpClearEntry");
  static readonly KpBinary = new KeyCode("KpBinary");
  static readonly KpOctal = new KeyCode("KpOctal");
  static readonly KpDecimal = new KeyCode("KpDecimal");
  static readonly KpHexadecimal = new KeyCode("KpHexadecimal");
}

const keyCodesByLayout = {
  "english-us": englishUsKeyCodes(KeyCode),
  german: germanKeyCodes(KeyCode),
};

type ReverseKeyCodes = {
  simple: Map<number, KeyCode>;
  sequence: Map<string, KeyCode>;
};

function buildReverseKeyCodes(
  encodings: Map<KeyCode, KeyCodeEncoding>,
): ReverseKeyCodes {
  const simple = new Map<number, KeyCode>();
  const sequence = new Map<string, KeyCode>();

  for (const [keyCode, encoding] of encodings) {
    switch (encoding.kind) {
      case "simple":
        simple.set(encoding.keyCode, keyCode);
        break;
      case "sequence":
        sequence.set(`${encoding.modifier}:${encoding.keyCode}`, keyCode);
        break;
    }
  }

  return { simple, sequence };
}

const reverseKeyCodesByLayout = {
  "english-us": buildReverseKeyCodes(keyCodesByLayout["english-us"]),
  german: buildReverseKeyCodes(keyCodesByLayout.german),
};

export function keyCodesForLayout(
  layout: Layout,
): Map<KeyCode, KeyCodeEncoding> {
  return keyCodesByLayout[layout];
}

export function reverseKeyCodes(layout: Layout): ReverseKeyCodes {
  return reverseKeyCodesByLayout[layout];
}

/** All key codes supported by the English-US layout. */
export const keyCodes: readonly KeyCode[] = [
  ...keyCodesForLayout("english-us").keys(),
];

export const keyCodeActionDefinition = defineAction<KeyCode>({
  actionId: ActionId.keyCode,
  label: "Key Code",
  encode: (action, ctx) => {
    const encoding = keyCodesForLayout(ctx.layout).get(action);
    if (!encoding) {
      throw new Error(`Unexpected key code "${action.value}".`);
    }

    switch (encoding.kind) {
      case "simple":
        return encodeKeyCodeSimple(encoding.keyCode, ctx.allocateEntryId);
      case "sequence":
        return encodeKeyCodeSequence(
          encoding.modifier,
          encoding.keyCode,
          ctx.allocateEntryId,
        );
    }
  },
  decode: (payload, ctx) => {
    const message = new capnp.Message(payload, false);
    const keyCodeAction = message.getRoot(KeyCodeAction);

    if (keyCodeAction.type !== KeyCodeType.KEYBOARD) {
      throw new Error("Consumer key codes are not supported yet.");
    }

    const keyCode = reverseKeyCodes(ctx.layout).simple.get(
      keyCodeAction.keyCode,
    );
    if (!keyCode) {
      throw new Error(
        `Unknown key code 0x${keyCodeAction.keyCode.toString(16)} for layout ${ctx.layout}.`,
      );
    }

    return keyCode;
  },
  render: (action, level) => {
    switch (action) {
      case KeyCode.LControl:
        return level == 0 ? (
          <TextSubtitleCombination text="Ctrl" subtitle="Left" />
        ) : (
          <>Ctrl</>
        );
      case KeyCode.RControl:
        return level == 0 ? (
          <TextSubtitleCombination text="Ctrl" subtitle="Right" />
        ) : (
          <>Ctrl</>
        );
      case KeyCode.LShift:
        return level == 0 ? (
          <IconSubtitleCombination
            icon={<ArrowBigUp />}
            subtitle="Left Shift"
          />
        ) : (
          <ArrowBigUp />
        );
      case KeyCode.RShift:
        return level == 0 ? (
          <IconSubtitleCombination
            icon={<ArrowBigUp />}
            subtitle="Right Shift"
          />
        ) : (
          <ArrowBigUp />
        );
      case KeyCode.LAlt:
        return level == 0 ? (
          <IconSubtitleCombination icon={<Option />} subtitle="Left Alt" />
        ) : (
          <Option />
        );
      case KeyCode.RAlt:
        return level == 0 ? (
          <IconSubtitleCombination icon={<Option />} subtitle="Right Alt" />
        ) : (
          <Option />
        );
      case KeyCode.LGui:
        return level == 0 ? (
          <IconSubtitleCombination icon={<Command />} subtitle="Left Super" />
        ) : (
          <Command />
        );
      case KeyCode.RGui:
        return level == 0 ? (
          <IconSubtitleCombination icon={<Command />} subtitle="Right Super" />
        ) : (
          <Command />
        );
      case KeyCode.Enter:
        return <CornerDownLeft />;
      case KeyCode.Escape:
        return <>ESC</>;
      case KeyCode.Backspace:
        return <Delete />;
      case KeyCode.Tab:
        return <ArrowRightToLine />;
      case KeyCode.Space:
        return <SpaceIcon />;
      case KeyCode.CapsLock:
        return <ArrowBigUpDash />;
      case KeyCode.PrintScreen:
        return <Small>Print Screen</Small>;
      case KeyCode.ScrollLock:
        return <Small>Scroll Lock</Small>;
      case KeyCode.PageUp:
        return <Small>Page Up</Small>;
      case KeyCode.PageDown:
        return <Small>Page Down</Small>;
      case KeyCode.Pause:
        return <Small>Pause</Small>;
      case KeyCode.Insert:
        return <Small>Insert</Small>;
      case KeyCode.Home:
        return <Small>Home</Small>;
      case KeyCode.Delete:
        return <Small>Delete</Small>;
      case KeyCode.End:
        return <Small>End</Small>;
      case KeyCode.Left:
        return <ArrowLeft />;
      case KeyCode.Right:
        return <ArrowRight />;
      case KeyCode.Up:
        return <ArrowUp />;
      case KeyCode.Down:
        return <ArrowDown />;
      case KeyCode.Application:
        return <Small>Application</Small>;
      case KeyCode.Power:
        return <Power />;
      case KeyCode.Execute:
        return <Small>Execute</Small>;
      case KeyCode.Help:
        return <Small>Help</Small>;
      case KeyCode.Menu:
        return <Small>Menu</Small>;
      case KeyCode.Select:
        return <Small>Select</Small>;
      case KeyCode.Stop:
        return <Small>Stop</Small>;
      case KeyCode.Again:
        return <Small>Again</Small>;
      case KeyCode.Undo:
        return <Small>Undo</Small>;
      case KeyCode.Cut:
        return <Small>Cut</Small>;
      case KeyCode.Copy:
        return <Small>Copy</Small>;
      case KeyCode.Paste:
        return <Small>Paste</Small>;
      case KeyCode.Find:
        return <Small>Find</Small>;
      case KeyCode.LockingCapsLock:
        return <Small>Locking Caps Lock</Small>;
      case KeyCode.LockingNumLock:
        return <Small>Locking Num Lock</Small>;
      case KeyCode.LockingScrollLock:
        return <Small>Locking Scroll Lock</Small>;
      case KeyCode.Mute:
        return <VolumeX />;
      case KeyCode.VolumeDown:
        return <Volume1 />;
      case KeyCode.VolumeUp:
        return <Volume2 />;
      case KeyCode.NonUsBackslash:
        return <Small>Non-US Backslash</Small>;
      case KeyCode.International1:
        return <Small>Int 1</Small>;
      case KeyCode.International2:
        return <Small>Int 2</Small>;
      case KeyCode.International3:
        return <Small>Int 3</Small>;
      case KeyCode.International4:
        return <Small>Int 4</Small>;
      case KeyCode.International5:
        return <Small>Int 5</Small>;
      case KeyCode.International6:
        return <Small>Int 6</Small>;
      case KeyCode.International7:
        return <Small>Int 7</Small>;
      case KeyCode.International8:
        return <Small>Int 8</Small>;
      case KeyCode.International9:
        return <Small>Int 9</Small>;
      case KeyCode.Lang1:
        return <Small>Lang 1</Small>;
      case KeyCode.Lang2:
        return <Small>Lang 2</Small>;
      case KeyCode.Lang3:
        return <Small>Lang 3</Small>;
      case KeyCode.Lang4:
        return <Small>Lang 4</Small>;
      case KeyCode.Lang5:
        return <Small>Lang 5</Small>;
      case KeyCode.Lang6:
        return <Small>Lang 6</Small>;
      case KeyCode.Lang7:
        return <Small>Lang 7</Small>;
      case KeyCode.Lang8:
        return <Small>Lang 8</Small>;
      case KeyCode.Lang9:
        return <Small>Lang 9</Small>;
      case KeyCode.Kp1:
        return <TextSubtitleCombination text="1" subtitle="Keypad" />;
      case KeyCode.Kp2:
        return <TextSubtitleCombination text="2" subtitle="Keypad" />;
      case KeyCode.Kp3:
        return <TextSubtitleCombination text="3" subtitle="Keypad" />;
      case KeyCode.Kp4:
        return <TextSubtitleCombination text="4" subtitle="Keypad" />;
      case KeyCode.Kp5:
        return <TextSubtitleCombination text="5" subtitle="Keypad" />;
      case KeyCode.Kp6:
        return <TextSubtitleCombination text="6" subtitle="Keypad" />;
      case KeyCode.Kp7:
        return <TextSubtitleCombination text="7" subtitle="Keypad" />;
      case KeyCode.Kp8:
        return <TextSubtitleCombination text="8" subtitle="Keypad" />;
      case KeyCode.Kp9:
        return <TextSubtitleCombination text="9" subtitle="Keypad" />;
      case KeyCode.Kp0:
        return <TextSubtitleCombination text="0" subtitle="Keypad" />;
      case KeyCode.Kp00:
        return <TextSubtitleCombination text="00" subtitle="Keypad" />;
      case KeyCode.Kp000:
        return <TextSubtitleCombination text="000" subtitle="Keypad" />;
      case KeyCode.NumLock:
        return <Small>Num Lock</Small>;
      case KeyCode.KpEnter:
        return (
          <IconSubtitleCombination
            icon={<CornerDownLeft />}
            subtitle="Keypad"
          />
        );
      case KeyCode.KpTab:
        return (
          <IconSubtitleCombination
            icon={<ArrowRightToLine />}
            subtitle="Keypad"
          />
        );
      case KeyCode.KpBackspace:
        return <IconSubtitleCombination icon={<Delete />} subtitle="Keypad" />;
      case KeyCode.KpA:
        return <TextSubtitleCombination text="A" subtitle="Keypad" />;
      case KeyCode.KpB:
        return <TextSubtitleCombination text="B" subtitle="Keypad" />;
      case KeyCode.KpC:
        return <TextSubtitleCombination text="C" subtitle="Keypad" />;
      case KeyCode.KpD:
        return <TextSubtitleCombination text="D" subtitle="Keypad" />;
      case KeyCode.KpE:
        return <TextSubtitleCombination text="E" subtitle="Keypad" />;
      case KeyCode.KpF:
        return <TextSubtitleCombination text="F" subtitle="Keypad" />;
      case KeyCode.KpMemoryStore:
        return (
          <SmallTextSubtitleCombination text="Memory Store" subtitle="Keypad" />
        );
      case KeyCode.KpMemoryRecall:
        return (
          <SmallTextSubtitleCombination
            text="Memory Recall"
            subtitle="Keypad"
          />
        );
      case KeyCode.KpMemoryClear:
        return (
          <SmallTextSubtitleCombination text="Memory Clear" subtitle="Keypad" />
        );
      case KeyCode.KpMemoryAdd:
        return (
          <SmallTextSubtitleCombination text="Memory Add" subtitle="Keypad" />
        );
      case KeyCode.KpMemorySubtract:
        return (
          <SmallTextSubtitleCombination
            text="Memory Subtract"
            subtitle="Keypad"
          />
        );
      case KeyCode.KpMemoryMultiply:
        return (
          <SmallTextSubtitleCombination
            text="Memory Multiply"
            subtitle="Keypad"
          />
        );
      case KeyCode.KpMemoryDivide:
        return (
          <SmallTextSubtitleCombination
            text="Memory Divide"
            subtitle="Keypad"
          />
        );
      case KeyCode.KpClear:
        return <SmallTextSubtitleCombination text="Clear" subtitle="Keypad" />;
      case KeyCode.KpClearEntry:
        return (
          <SmallTextSubtitleCombination text="Clear Entry" subtitle="Keypad" />
        );
      case KeyCode.KpBinary:
        return <SmallTextSubtitleCombination text="Binary" subtitle="Keypad" />;
      case KeyCode.KpOctal:
        return <SmallTextSubtitleCombination text="Octal" subtitle="Keypad" />;
      case KeyCode.KpDecimal:
        return (
          <SmallTextSubtitleCombination text="Decimal" subtitle="Keypad" />
        );
      case KeyCode.KpHexadecimal:
        return (
          <SmallTextSubtitleCombination text="Hexadecimal" subtitle="Keypad" />
        );
      case KeyCode.KpSlash:
        return <TextSubtitleCombination text="/" subtitle="Keypad" />;
      case KeyCode.KpAsterisk:
        return <TextSubtitleCombination text="*" subtitle="Keypad" />;
      case KeyCode.KpMinus:
        return <TextSubtitleCombination text="-" subtitle="Keypad" />;
      case KeyCode.KpPlus:
        return <TextSubtitleCombination text="+" subtitle="Keypad" />;
      case KeyCode.KpDot:
        return <TextSubtitleCombination text="." subtitle="Keypad" />;
      case KeyCode.KpEqual:
        return <TextSubtitleCombination text="=" subtitle="Keypad" />;
      case KeyCode.KpComma:
        return <TextSubtitleCombination text="," subtitle="Keypad" />;
      case KeyCode.KpEqualSign:
        return <TextSubtitleCombination text="=" subtitle="Keypad" />;
      case KeyCode.KpThousandsSeparator:
        return (
          <SmallTextSubtitleCombination
            text="Thousands Separator"
            subtitle="Keypad"
          />
        );
      case KeyCode.KpDecimalSeparator:
        return (
          <SmallTextSubtitleCombination
            text="Decimal Separator"
            subtitle="Keypad"
          />
        );
      case KeyCode.KpCurrencyUnit:
        return (
          <SmallTextSubtitleCombination
            text="Currency Unit"
            subtitle="Keypad"
          />
        );
      case KeyCode.KpCurrencySubunit:
        return (
          <SmallTextSubtitleCombination
            text="Currency Subunit"
            subtitle="Keypad"
          />
        );
      case KeyCode.KpLeftParen:
        return <TextSubtitleCombination text="(" subtitle="Keypad" />;
      case KeyCode.KpRightParen:
        return <TextSubtitleCombination text="(" subtitle="Keypad" />;
      case KeyCode.KpLeftBrace:
        return <TextSubtitleCombination text="{" subtitle="Keypad" />;
      case KeyCode.KpRightBrace:
        return <TextSubtitleCombination text="}" subtitle="Keypad" />;
      case KeyCode.KpXor:
        return <SmallTextSubtitleCombination text="XOR" subtitle="Keypad" />;
      case KeyCode.KpCaret:
        return <TextSubtitleCombination text="^" subtitle="Keypad" />;
      case KeyCode.KpPercent:
        return <TextSubtitleCombination text="%" subtitle="Keypad" />;
      case KeyCode.KpLessThan:
        return <TextSubtitleCombination text="<" subtitle="Keypad" />;
      case KeyCode.KpGreaterThan:
        return <TextSubtitleCombination text=">" subtitle="Keypad" />;
      case KeyCode.KpAmpersand:
        return <TextSubtitleCombination text="&" subtitle="Keypad" />;
      case KeyCode.KpDoubleAmpersand:
        return <TextSubtitleCombination text="&&" subtitle="Keypad" />;
      case KeyCode.KpPipe:
        return <TextSubtitleCombination text="|" subtitle="Keypad" />;
      case KeyCode.KpDoublePipe:
        return <TextSubtitleCombination text="||" subtitle="Keypad" />;
      case KeyCode.KpColon:
        return <TextSubtitleCombination text=":" subtitle="Keypad" />;
      case KeyCode.KpHash:
        return <TextSubtitleCombination text="#" subtitle="Keypad" />;
      case KeyCode.KpSpace:
        return <IconSubtitleCombination icon={<Space />} subtitle="Keypad" />;
      case KeyCode.KpAt:
        return <TextSubtitleCombination text="@" subtitle="Keypad" />;
      case KeyCode.KpExclamation:
        return <TextSubtitleCombination text="!" subtitle="Keypad" />;
      case KeyCode.KpPlusMinus:
        return <TextSubtitleCombination text="+-" subtitle="Keypad" />;
      default:
        return <>{action.displayValue()}</>;
    }
  },
  renderTooltip: (action) => {
    return <>{action.tooltipValue() ?? action.displayValue()}</>;
  },
  remap: (action) => action,
  children: () => [],
});

function encodeKeyCodeSimple(
  value: number,
  allocateEntryId: () => number,
): ActionEntries {
  return {
    rootActionEntry: buildActionEntry(
      allocateEntryId(),
      buildAction(ActionId.keyCode, KeyCodeAction, (action) => {
        action.keyCode = value;
        action.type = KeyCodeType.KEYBOARD;
      }),
    ),
    childActionEntries: [],
  };
}

function encodeKeyCodeSequence(
  keyCode1: number,
  keyCode2: number,
  allocateEntryId: () => number,
): ActionEntries {
  const entryId1 = allocateEntryId();
  const entryId2 = allocateEntryId();

  const entry1 = buildActionEntry(
    entryId1,
    buildAction(ActionId.keyCode, KeyCodeAction, (action) => {
      action.keyCode = keyCode1;
      action.type = KeyCodeType.KEYBOARD;
    }),
  );
  const entry2 = buildActionEntry(
    entryId2,
    buildAction(ActionId.keyCode, KeyCodeAction, (action) => {
      action.keyCode = keyCode2;
      action.type = KeyCodeType.KEYBOARD;
    }),
  );

  const sequenceAction = buildAction(
    ActionId.ordered,
    OrderedAction,
    (action) => {
      action.action1EntryId = entryId1;
      action.action2EntryId = entryId2;
    },
  );

  return {
    rootActionEntry: buildActionEntry(allocateEntryId(), sequenceAction),
    childActionEntries: [entry1, entry2],
  };
}
