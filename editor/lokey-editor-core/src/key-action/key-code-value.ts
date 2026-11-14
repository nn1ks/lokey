import { Layout } from "./layouts";
import { englishUsKeyCodes } from "./layouts/english-us";
import { germanKeyCodes } from "./layouts/german";
import { KeyCodeEncoding } from "./layouts/types";

export class KeyCodeValue {
  constructor(readonly value: string) {}

  static readonly LControl = new KeyCodeValue("LControl");
  static readonly RControl = new KeyCodeValue("RControl");
  static readonly LShift = new KeyCodeValue("LShift");
  static readonly RShift = new KeyCodeValue("RShift");
  static readonly LAlt = new KeyCodeValue("LAlt");
  static readonly RAlt = new KeyCodeValue("RAlt");
  static readonly LGui = new KeyCodeValue("LGui");
  static readonly RGui = new KeyCodeValue("RGui");
  static readonly A = new KeyCodeValue("A");
  static readonly B = new KeyCodeValue("B");
  static readonly C = new KeyCodeValue("C");
  static readonly D = new KeyCodeValue("D");
  static readonly E = new KeyCodeValue("E");
  static readonly F = new KeyCodeValue("F");
  static readonly G = new KeyCodeValue("G");
  static readonly H = new KeyCodeValue("H");
  static readonly I = new KeyCodeValue("I");
  static readonly J = new KeyCodeValue("J");
  static readonly K = new KeyCodeValue("K");
  static readonly L = new KeyCodeValue("L");
  static readonly M = new KeyCodeValue("M");
  static readonly N = new KeyCodeValue("N");
  static readonly O = new KeyCodeValue("O");
  static readonly P = new KeyCodeValue("P");
  static readonly Q = new KeyCodeValue("Q");
  static readonly R = new KeyCodeValue("R");
  static readonly S = new KeyCodeValue("S");
  static readonly T = new KeyCodeValue("T");
  static readonly U = new KeyCodeValue("U");
  static readonly V = new KeyCodeValue("V");
  static readonly W = new KeyCodeValue("W");
  static readonly X = new KeyCodeValue("X");
  static readonly Y = new KeyCodeValue("Y");
  static readonly Z = new KeyCodeValue("Z");
  static readonly AE = new KeyCodeValue("AE");
  static readonly OE = new KeyCodeValue("OE");
  static readonly UE = new KeyCodeValue("UE");
  static readonly SharpS = new KeyCodeValue("SharpS");
  static readonly N1 = new KeyCodeValue("N1");
  static readonly N2 = new KeyCodeValue("N2");
  static readonly N3 = new KeyCodeValue("N3");
  static readonly N4 = new KeyCodeValue("N4");
  static readonly N5 = new KeyCodeValue("N5");
  static readonly N6 = new KeyCodeValue("N6");
  static readonly N7 = new KeyCodeValue("N7");
  static readonly N8 = new KeyCodeValue("N8");
  static readonly N9 = new KeyCodeValue("N9");
  static readonly N0 = new KeyCodeValue("N0");
  static readonly Enter = new KeyCodeValue("Enter");
  static readonly Escape = new KeyCodeValue("Escape");
  static readonly Backspace = new KeyCodeValue("Backspace");
  static readonly Tab = new KeyCodeValue("Tab");
  static readonly Space = new KeyCodeValue("Space");
  static readonly Minus = new KeyCodeValue("Minus");
  static readonly Equal = new KeyCodeValue("Equal");
  static readonly LeftParen = new KeyCodeValue("LeftParen");
  static readonly RightParen = new KeyCodeValue("RightParen");
  static readonly LeftBracket = new KeyCodeValue("LeftBracket");
  static readonly RightBracket = new KeyCodeValue("RightBracket");
  static readonly LeftBrace = new KeyCodeValue("LeftBrace");
  static readonly RightBrace = new KeyCodeValue("RightBrace");
  static readonly Backslash = new KeyCodeValue("Backslash");
  static readonly Hash = new KeyCodeValue("Hash");
  static readonly Semicolon = new KeyCodeValue("Semicolon");
  static readonly Apostrophe = new KeyCodeValue("Apostrophe");
  static readonly Grave = new KeyCodeValue("Grave");
  static readonly Acute = new KeyCodeValue("Acute");
  static readonly Comma = new KeyCodeValue("Comma");
  static readonly Dot = new KeyCodeValue("Dot");
  static readonly Slash = new KeyCodeValue("Slash");
  static readonly Tilde = new KeyCodeValue("Tilde");
  static readonly ExclamationMark = new KeyCodeValue("ExclamationMark");
  static readonly At = new KeyCodeValue("At");
  static readonly Dollar = new KeyCodeValue("Dollar");
  static readonly Percent = new KeyCodeValue("Percent");
  static readonly Caret = new KeyCodeValue("Caret");
  static readonly Degree = new KeyCodeValue("Degree");
  static readonly Ampersand = new KeyCodeValue("Ampersand");
  static readonly Asterisk = new KeyCodeValue("Asterisk");
  static readonly Underscore = new KeyCodeValue("Underscore");
  static readonly Plus = new KeyCodeValue("Plus");
  static readonly Pipe = new KeyCodeValue("Pipe");
  static readonly Colon = new KeyCodeValue("Colon");
  static readonly DoubleQuote = new KeyCodeValue("DoubleQuote");
  static readonly GreaterThan = new KeyCodeValue("GreaterThan");
  static readonly LessThan = new KeyCodeValue("LessThan");
  static readonly QuestionMark = new KeyCodeValue("QuestionMark");
  static readonly SectionSign = new KeyCodeValue("SectionSign");
  static readonly Euro = new KeyCodeValue("Euro");
  static readonly CapsLock = new KeyCodeValue("CapsLock");
  static readonly PrintScreen = new KeyCodeValue("PrintScreen");
  static readonly ScrollLock = new KeyCodeValue("ScrollLock");
  static readonly Pause = new KeyCodeValue("Pause");
  static readonly Insert = new KeyCodeValue("Insert");
  static readonly Home = new KeyCodeValue("Home");
  static readonly PageUp = new KeyCodeValue("PageUp");
  static readonly Delete = new KeyCodeValue("Delete");
  static readonly End = new KeyCodeValue("End");
  static readonly PageDown = new KeyCodeValue("PageDown");
  static readonly Right = new KeyCodeValue("Right");
  static readonly Left = new KeyCodeValue("Left");
  static readonly Down = new KeyCodeValue("Down");
  static readonly Up = new KeyCodeValue("Up");
  static readonly F1 = new KeyCodeValue("F1");
  static readonly F2 = new KeyCodeValue("F2");
  static readonly F3 = new KeyCodeValue("F3");
  static readonly F4 = new KeyCodeValue("F4");
  static readonly F5 = new KeyCodeValue("F5");
  static readonly F6 = new KeyCodeValue("F6");
  static readonly F7 = new KeyCodeValue("F7");
  static readonly F8 = new KeyCodeValue("F8");
  static readonly F9 = new KeyCodeValue("F9");
  static readonly F10 = new KeyCodeValue("F10");
  static readonly F11 = new KeyCodeValue("F11");
  static readonly F12 = new KeyCodeValue("F12");
  static readonly F13 = new KeyCodeValue("F13");
  static readonly F14 = new KeyCodeValue("F14");
  static readonly F15 = new KeyCodeValue("F15");
  static readonly F16 = new KeyCodeValue("F16");
  static readonly F17 = new KeyCodeValue("F17");
  static readonly F18 = new KeyCodeValue("F18");
  static readonly F19 = new KeyCodeValue("F19");
  static readonly F20 = new KeyCodeValue("F20");
  static readonly F21 = new KeyCodeValue("F21");
  static readonly F22 = new KeyCodeValue("F22");
  static readonly F23 = new KeyCodeValue("F23");
  static readonly F24 = new KeyCodeValue("F24");
  static readonly NumLock = new KeyCodeValue("NumLock");
  static readonly KpSlash = new KeyCodeValue("KpSlash");
  static readonly KpAsterisk = new KeyCodeValue("KpAsterisk");
  static readonly KpMinus = new KeyCodeValue("KpMinus");
  static readonly KpPlus = new KeyCodeValue("KpPlus");
  static readonly KpEnter = new KeyCodeValue("KpEnter");
  static readonly KpDot = new KeyCodeValue("KpDot");
  static readonly KpEqual = new KeyCodeValue("KpEqual");
  static readonly Kp1 = new KeyCodeValue("Kp1");
  static readonly Kp2 = new KeyCodeValue("Kp2");
  static readonly Kp3 = new KeyCodeValue("Kp3");
  static readonly Kp4 = new KeyCodeValue("Kp4");
  static readonly Kp5 = new KeyCodeValue("Kp5");
  static readonly Kp6 = new KeyCodeValue("Kp6");
  static readonly Kp7 = new KeyCodeValue("Kp7");
  static readonly Kp8 = new KeyCodeValue("Kp8");
  static readonly Kp9 = new KeyCodeValue("Kp9");
  static readonly Kp0 = new KeyCodeValue("Kp0");
  static readonly NonUsBackslash = new KeyCodeValue("NonUsBackslash");
  static readonly Application = new KeyCodeValue("Application");
  static readonly Power = new KeyCodeValue("Power");
  static readonly Execute = new KeyCodeValue("Execute");
  static readonly Help = new KeyCodeValue("Help");
  static readonly Menu = new KeyCodeValue("Menu");
  static readonly Select = new KeyCodeValue("Select");
  static readonly Stop = new KeyCodeValue("Stop");
  static readonly Again = new KeyCodeValue("Again");
  static readonly Undo = new KeyCodeValue("Undo");
  static readonly Cut = new KeyCodeValue("Cut");
  static readonly Copy = new KeyCodeValue("Copy");
  static readonly Paste = new KeyCodeValue("Paste");
  static readonly Find = new KeyCodeValue("Find");
  static readonly Mute = new KeyCodeValue("Mute");
  static readonly VolumeUp = new KeyCodeValue("VolumeUp");
  static readonly VolumeDown = new KeyCodeValue("VolumeDown");
  static readonly LockingCapsLock = new KeyCodeValue("LockingCapsLock");
  static readonly LockingNumLock = new KeyCodeValue("LockingNumLock");
  static readonly LockingScrollLock = new KeyCodeValue("LockingScrollLock");
  static readonly KpComma = new KeyCodeValue("KpComma");
  static readonly KpEqualSign = new KeyCodeValue("KpEqualSign");
  static readonly International1 = new KeyCodeValue("International1");
  static readonly International2 = new KeyCodeValue("International2");
  static readonly International3 = new KeyCodeValue("International3");
  static readonly International4 = new KeyCodeValue("International4");
  static readonly International5 = new KeyCodeValue("International5");
  static readonly International6 = new KeyCodeValue("International6");
  static readonly International7 = new KeyCodeValue("International7");
  static readonly International8 = new KeyCodeValue("International8");
  static readonly International9 = new KeyCodeValue("International9");
  static readonly Lang1 = new KeyCodeValue("Lang1");
  static readonly Lang2 = new KeyCodeValue("Lang2");
  static readonly Lang3 = new KeyCodeValue("Lang3");
  static readonly Lang4 = new KeyCodeValue("Lang4");
  static readonly Lang5 = new KeyCodeValue("Lang5");
  static readonly Lang6 = new KeyCodeValue("Lang6");
  static readonly Lang7 = new KeyCodeValue("Lang7");
  static readonly Lang8 = new KeyCodeValue("Lang8");
  static readonly Lang9 = new KeyCodeValue("Lang9");
  static readonly Kp00 = new KeyCodeValue("Kp00");
  static readonly Kp000 = new KeyCodeValue("Kp000");
  static readonly KpThousandsSeparator = new KeyCodeValue(
    "KpThousandsSeparator",
  );
  static readonly KpDecimalSeparator = new KeyCodeValue("KpDecimalSeparator");
  static readonly KpCurrencyUnit = new KeyCodeValue("KpCurrencyUnit");
  static readonly KpCurrencySubunit = new KeyCodeValue("KpCurrencySubunit");
  static readonly KpLeftParen = new KeyCodeValue("KpLeftParen");
  static readonly KpRightParen = new KeyCodeValue("KpRightParen");
  static readonly KpLeftBrace = new KeyCodeValue("KpLeftBrace");
  static readonly KpRightBrace = new KeyCodeValue("KpRightBrace");
  static readonly KpTab = new KeyCodeValue("KpTab");
  static readonly KpBackspace = new KeyCodeValue("KpBackspace");
  static readonly KpA = new KeyCodeValue("KpA");
  static readonly KpB = new KeyCodeValue("KpB");
  static readonly KpC = new KeyCodeValue("KpC");
  static readonly KpD = new KeyCodeValue("KpD");
  static readonly KpE = new KeyCodeValue("KpE");
  static readonly KpF = new KeyCodeValue("KpF");
  static readonly KpXor = new KeyCodeValue("KpXor");
  static readonly KpCaret = new KeyCodeValue("KpCaret");
  static readonly KpPercent = new KeyCodeValue("KpPercent");
  static readonly KpLessThan = new KeyCodeValue("KpLessThan");
  static readonly KpGreaterThan = new KeyCodeValue("KpGreaterThan");
  static readonly KpAmpersand = new KeyCodeValue("KpAmpersand");
  static readonly KpDoubleAmpersand = new KeyCodeValue("KpDoubleAmpersand");
  static readonly KpPipe = new KeyCodeValue("KpPipe");
  static readonly KpDoublePipe = new KeyCodeValue("KpDoublePipe");
  static readonly KpColon = new KeyCodeValue("KpColon");
  static readonly KpHash = new KeyCodeValue("KpHash");
  static readonly KpSpace = new KeyCodeValue("KpSpace");
  static readonly KpAt = new KeyCodeValue("KpAt");
  static readonly KpExclamation = new KeyCodeValue("KpExclamation");
  static readonly KpMemoryStore = new KeyCodeValue("KpMemoryStore");
  static readonly KpMemoryRecall = new KeyCodeValue("KpMemoryRecall");
  static readonly KpMemoryClear = new KeyCodeValue("KpMemoryClear");
  static readonly KpMemoryAdd = new KeyCodeValue("KpMemoryAdd");
  static readonly KpMemorySubtract = new KeyCodeValue("KpMemorySubtract");
  static readonly KpMemoryMultiply = new KeyCodeValue("KpMemoryMultiply");
  static readonly KpMemoryDivide = new KeyCodeValue("KpMemoryDivide");
  static readonly KpPlusMinus = new KeyCodeValue("KpPlusMinus");
  static readonly KpClear = new KeyCodeValue("KpClear");
  static readonly KpClearEntry = new KeyCodeValue("KpClearEntry");
  static readonly KpBinary = new KeyCodeValue("KpBinary");
  static readonly KpOctal = new KeyCodeValue("KpOctal");
  static readonly KpDecimal = new KeyCodeValue("KpDecimal");
  static readonly KpHexadecimal = new KeyCodeValue("KpHexadecimal");
}

const keyCodeValuesByLayout = {
  "english-us": englishUsKeyCodes(),
  german: germanKeyCodes(),
};

export function keyCodeValues(
  layout: Layout,
): Map<KeyCodeValue, KeyCodeEncoding> {
  return keyCodeValuesByLayout[layout];
}

export class ReverseKeyCodeValues {
  readonly #simple: Map<number, KeyCodeValue> = new Map();
  readonly #sequence: Map<string, KeyCodeValue> = new Map();

  private constructor() {}

  static build(
    encodings: Map<KeyCodeValue, KeyCodeEncoding>,
  ): ReverseKeyCodeValues {
    const reverseKeyCodeValues = new ReverseKeyCodeValues();

    for (const [keyCodeValue, encoding] of encodings) {
      switch (encoding.kind) {
        case "single":
          reverseKeyCodeValues.#set(
            encoding.rawKeyCode.code,
            undefined,
            keyCodeValue,
          );
          break;
        case "combined":
          reverseKeyCodeValues.#set(
            encoding.rawKeyCode1.code,
            encoding.rawKeyCode2.code,
            keyCodeValue,
          );
          break;
      }
    }

    return reverseKeyCodeValues;
  }

  #set(
    rawKeyCode1: number,
    rawKeyCode2: number | undefined,
    keyCodeValue: KeyCodeValue,
  ): void {
    if (rawKeyCode2) {
      this.#sequence.set(`${rawKeyCode1}:${rawKeyCode2}`, keyCodeValue);
    } else {
      this.#simple.set(rawKeyCode1, keyCodeValue);
    }
  }

  get(rawKeyCode1: number, rawKeyCode2?: number): KeyCodeValue | undefined {
    if (rawKeyCode2) {
      return this.#sequence.get(`${rawKeyCode1}:${rawKeyCode2}`);
    }
    return this.#simple.get(rawKeyCode1);
  }
}

const reverseKeyCodeValuesByLayout = {
  "english-us": ReverseKeyCodeValues.build(keyCodeValuesByLayout["english-us"]),
  german: ReverseKeyCodeValues.build(keyCodeValuesByLayout.german),
};

export function reverseKeyCodeValues(layout: Layout): ReverseKeyCodeValues {
  return reverseKeyCodeValuesByLayout[layout];
}
