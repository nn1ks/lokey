import type { KeyCode } from "../actions/key-code";
import { combined, rawKeyCode, single, type KeyCodeEncoding } from "./types";

// Modifiers
const LControl = rawKeyCode("LControl", 0xe0);
const RControl = rawKeyCode("RControl", 0xe4);
const LShift = rawKeyCode("LShift", 0xe1);
const RShift = rawKeyCode("RShift", 0xe5);
const LAlt = rawKeyCode("LAlt", 0xe2);
const RAlt = rawKeyCode("RAlt", 0xe6);
const LGui = rawKeyCode("LGui", 0xe3);
const RGui = rawKeyCode("RGui", 0xe7);

// Letters
const A = rawKeyCode("A", 0x04);
const B = rawKeyCode("B", 0x05);
const C = rawKeyCode("C", 0x06);
const D = rawKeyCode("D", 0x07);
const E = rawKeyCode("E", 0x08);
const F = rawKeyCode("F", 0x09);
const G = rawKeyCode("G", 0x0a);
const H = rawKeyCode("H", 0x0b);
const I = rawKeyCode("I", 0x0c);
const J = rawKeyCode("J", 0x0d);
const K = rawKeyCode("K", 0x0e);
const L = rawKeyCode("L", 0x0f);
const M = rawKeyCode("M", 0x10);
const N = rawKeyCode("N", 0x11);
const O = rawKeyCode("O", 0x12);
const P = rawKeyCode("P", 0x13);
const Q = rawKeyCode("Q", 0x14);
const R = rawKeyCode("R", 0x15);
const S = rawKeyCode("S", 0x16);
const T = rawKeyCode("T", 0x17);
const U = rawKeyCode("U", 0x18);
const V = rawKeyCode("V", 0x19);
const W = rawKeyCode("W", 0x1a);
const X = rawKeyCode("X", 0x1b);
const Y = rawKeyCode("Y", 0x1d);
const Z = rawKeyCode("Z", 0x1c);
const AE = rawKeyCode("Ä", 0x34);
const OE = rawKeyCode("Ö", 0x33);
const UE = rawKeyCode("Ü", 0x2f);
const SharpS = rawKeyCode("ß", 0x2d);

// Digits
const N1 = rawKeyCode("N1", 0x1e);
const N2 = rawKeyCode("N2", 0x1f);
const N3 = rawKeyCode("N3", 0x20);
const N4 = rawKeyCode("N4", 0x21);
const N5 = rawKeyCode("N5", 0x22);
const N6 = rawKeyCode("N6", 0x23);
const N7 = rawKeyCode("N7", 0x24);
const N8 = rawKeyCode("N8", 0x25);
const N9 = rawKeyCode("N9", 0x26);
const N0 = rawKeyCode("N0", 0x27);

// Symbols
const Caret = rawKeyCode("Caret", 0x35);
const Acute = rawKeyCode("Grave", 0x2e);
const Plus = rawKeyCode("Plus", 0x30);
const Hash = rawKeyCode("Hash", 0x32);
const LessThan = rawKeyCode("LessThan", 0x64);
const Comma = rawKeyCode("Comma", 0x36);
const Dot = rawKeyCode("Comma", 0x37);
const Minus = rawKeyCode("Minus", 0x38);

// Whitespace and control keys
const Enter = rawKeyCode("Enter", 0x28);
const Escape = rawKeyCode("Escape", 0x29);
const Backspace = rawKeyCode("Backspace", 0x2a);
const Tab = rawKeyCode("Tab", 0x2b);
const Space = rawKeyCode("Space", 0x2c);
const CapsLock = rawKeyCode("CapsLock", 0x39);

// Navigation and editing
const PrintScreen = rawKeyCode("PrintScreen", 0x46);
const ScrollLock = rawKeyCode("ScrollLock", 0x47);
const Pause = rawKeyCode("Pause", 0x48);
const Insert = rawKeyCode("Insert", 0x49);
const Home = rawKeyCode("Home", 0x4a);
const PageUp = rawKeyCode("PageUp", 0x4b);
const Delete = rawKeyCode("Delete", 0x4c);
const End = rawKeyCode("End", 0x4d);
const PageDown = rawKeyCode("PageDown", 0x4e);
const Right = rawKeyCode("Right", 0x4f);
const Left = rawKeyCode("Left", 0x50);
const Down = rawKeyCode("Down", 0x51);
const Up = rawKeyCode("Up", 0x52);

// Function keys
const F1 = rawKeyCode("F1", 0x3a);
const F2 = rawKeyCode("F2", 0x3b);
const F3 = rawKeyCode("F3", 0x3c);
const F4 = rawKeyCode("F4", 0x3d);
const F5 = rawKeyCode("F5", 0x3e);
const F6 = rawKeyCode("F6", 0x3f);
const F7 = rawKeyCode("F7", 0x40);
const F8 = rawKeyCode("F8", 0x41);
const F9 = rawKeyCode("F9", 0x42);
const F10 = rawKeyCode("F10", 0x43);
const F11 = rawKeyCode("F11", 0x44);
const F12 = rawKeyCode("F12", 0x45);
const F13 = rawKeyCode("F13", 0x68);
const F14 = rawKeyCode("F14", 0x69);
const F15 = rawKeyCode("F15", 0x6a);
const F16 = rawKeyCode("F16", 0x6b);
const F17 = rawKeyCode("F17", 0x6c);
const F18 = rawKeyCode("F18", 0x6d);
const F19 = rawKeyCode("F19", 0x6e);
const F20 = rawKeyCode("F20", 0x6f);
const F21 = rawKeyCode("F21", 0x70);
const F22 = rawKeyCode("F22", 0x71);
const F23 = rawKeyCode("F23", 0x72);
const F24 = rawKeyCode("F24", 0x73);

// Keypad
const NumLock = rawKeyCode("NumLock", 0x53);
const KpSlash = rawKeyCode("KpSlash", 0x54);
const KpAsterisk = rawKeyCode("KpAsterisk", 0x55);
const KpMinus = rawKeyCode("KpMinus", 0x56);
const KpPlus = rawKeyCode("KpPlus", 0x57);
const KpEnter = rawKeyCode("KpEnter", 0x58);
const KpDot = rawKeyCode("KpDot", 0x63);
const KpEqual = rawKeyCode("KpEqual", 0x67);
const Kp1 = rawKeyCode("Kp1", 0x59);
const Kp2 = rawKeyCode("Kp2", 0x5a);
const Kp3 = rawKeyCode("Kp3", 0x5b);
const Kp4 = rawKeyCode("Kp4", 0x5c);
const Kp5 = rawKeyCode("Kp5", 0x5d);
const Kp6 = rawKeyCode("Kp6", 0x5e);
const Kp7 = rawKeyCode("Kp7", 0x5f);
const Kp8 = rawKeyCode("Kp8", 0x60);
const Kp9 = rawKeyCode("Kp9", 0x61);
const Kp0 = rawKeyCode("Kp0", 0x62);
const KpComma = rawKeyCode("KpComma", 0x85);
const KpEqualSign = rawKeyCode("KpEqualSign", 0x86);
const Kp00 = rawKeyCode("Kp00", 0xb0);
const Kp000 = rawKeyCode("Kp000", 0xb1);
const KpThousandsSeparator = rawKeyCode("KpThousandsSeparator", 0xb2);
const KpDecimalSeparator = rawKeyCode("KpDecimalSeparator", 0xb3);
const KpCurrencyUnit = rawKeyCode("KpCurrencyUnit", 0xb4);
const KpCurrencySubunit = rawKeyCode("KpCurrencySubunit", 0xb5);
const KpLeftParen = rawKeyCode("KpLeftParen", 0xb6);
const KpRightParen = rawKeyCode("KpRightParen", 0xb7);
const KpLeftBrace = rawKeyCode("KpLeftBrace", 0xb8);
const KpRightBrace = rawKeyCode("KpRightBrace", 0xb9);
const KpTab = rawKeyCode("KpTab", 0xba);
const KpBackspace = rawKeyCode("KpBackspace", 0xbb);
const KpA = rawKeyCode("KpA", 0xbc);
const KpB = rawKeyCode("KpB", 0xbd);
const KpC = rawKeyCode("KpC", 0xbe);
const KpD = rawKeyCode("KpD", 0xbf);
const KpE = rawKeyCode("KpE", 0xc0);
const KpF = rawKeyCode("KpF", 0xc1);
const KpXor = rawKeyCode("KpXor", 0xc2);
const KpCaret = rawKeyCode("KpCaret", 0xc3);
const KpPercent = rawKeyCode("KpPercent", 0xc4);
const KpLessThan = rawKeyCode("KpLessThan", 0xc5);
const KpGreaterThan = rawKeyCode("KpGreaterThan", 0xc6);
const KpAmpersand = rawKeyCode("KpAmpersand", 0xc7);
const KpDoubleAmpersand = rawKeyCode("KpDoubleAmpersand", 0xc8);
const KpPipe = rawKeyCode("KpPipe", 0xc9);
const KpDoublePipe = rawKeyCode("KpDoublePipe", 0xca);
const KpColon = rawKeyCode("KpColon", 0xcb);
const KpHash = rawKeyCode("KpHash", 0xcc);
const KpSpace = rawKeyCode("KpSpace", 0xcd);
const KpAt = rawKeyCode("KpAt", 0xce);
const KpExclamation = rawKeyCode("KpExclamation", 0xcf);
const KpMemoryStore = rawKeyCode("KpMemoryStore", 0xd0);
const KpMemoryRecall = rawKeyCode("KpMemoryRecall", 0xd1);
const KpMemoryClear = rawKeyCode("KpMemoryClear", 0xd2);
const KpMemoryAdd = rawKeyCode("KpMemoryAdd", 0xd3);
const KpMemorySubtract = rawKeyCode("KpMemorySubtract", 0xd4);
const KpMemoryMultiply = rawKeyCode("KpMemoryMultiply", 0xd5);
const KpMemoryDivide = rawKeyCode("KpMemoryDivide", 0xd6);
const KpPlusMinus = rawKeyCode("KpPlusMinus", 0xd7);
const KpClear = rawKeyCode("KpClear", 0xd8);
const KpClearEntry = rawKeyCode("KpClearEntry", 0xd9);
const KpBinary = rawKeyCode("KpBinary", 0xda);
const KpOctal = rawKeyCode("KpOctal", 0xdb);
const KpDecimal = rawKeyCode("KpDecimal", 0xdc);
const KpHexadecimal = rawKeyCode("KpHexadecimal", 0xdd);

// Miscellaneous
const Application = rawKeyCode("Application", 0x65);
const Power = rawKeyCode("Power", 0x66);
const Execute = rawKeyCode("Execute", 0x74);
const Help = rawKeyCode("Help", 0x75);
const Menu = rawKeyCode("Menu", 0x76);
const Select = rawKeyCode("Select", 0x77);
const Stop = rawKeyCode("Stop", 0x78);
const Again = rawKeyCode("Again", 0x79);
const Undo = rawKeyCode("Undo", 0x7a);
const Cut = rawKeyCode("Cut", 0x7b);
const Copy = rawKeyCode("Copy", 0x7c);
const Paste = rawKeyCode("Paste", 0x7d);
const Find = rawKeyCode("Find", 0x7e);
const Mute = rawKeyCode("Mute", 0x7f);
const VolumeUp = rawKeyCode("VolumeUp", 0x80);
const VolumeDown = rawKeyCode("VolumeDown", 0x81);
const LockingCapsLock = rawKeyCode("LockingCapsLock", 0x82);
const LockingNumLock = rawKeyCode("LockingNumLock", 0x83);
const LockingScrollLock = rawKeyCode("LockingScrollLock", 0x84);

// International and language keys
const International1 = rawKeyCode("International1", 0x87);
const International2 = rawKeyCode("International2", 0x88);
const International3 = rawKeyCode("International3", 0x89);
const International4 = rawKeyCode("International4", 0x8a);
const International5 = rawKeyCode("International5", 0x8b);
const International6 = rawKeyCode("International6", 0x8c);
const International7 = rawKeyCode("International7", 0x8d);
const International8 = rawKeyCode("International8", 0x8e);
const International9 = rawKeyCode("International9", 0x8f);
const Lang1 = rawKeyCode("Lang1", 0x90);
const Lang2 = rawKeyCode("Lang2", 0x91);
const Lang3 = rawKeyCode("Lang3", 0x92);
const Lang4 = rawKeyCode("Lang4", 0x93);
const Lang5 = rawKeyCode("Lang5", 0x94);
const Lang6 = rawKeyCode("Lang6", 0x95);
const Lang7 = rawKeyCode("Lang7", 0x96);
const Lang8 = rawKeyCode("Lang8", 0x97);
const Lang9 = rawKeyCode("Lang9", 0x98);

export function germanKeyCodes(
  keyCodeClass: typeof import("../actions/key-code").KeyCode,
): Map<KeyCode, KeyCodeEncoding> {
  return new Map<KeyCode, KeyCodeEncoding>([
    // Modifiers
    [keyCodeClass.LControl, single(LControl)],
    [keyCodeClass.RControl, single(RControl)],
    [keyCodeClass.LShift, single(LShift)],
    [keyCodeClass.RShift, single(RShift)],
    [keyCodeClass.LAlt, single(LAlt)],
    [keyCodeClass.RAlt, single(RAlt)],
    [keyCodeClass.LGui, single(LGui)],
    [keyCodeClass.RGui, single(RGui)],

    // Letters
    [keyCodeClass.A, single(A)],
    [keyCodeClass.B, single(B)],
    [keyCodeClass.C, single(C)],
    [keyCodeClass.D, single(D)],
    [keyCodeClass.E, single(E)],
    [keyCodeClass.F, single(F)],
    [keyCodeClass.G, single(G)],
    [keyCodeClass.H, single(H)],
    [keyCodeClass.I, single(I)],
    [keyCodeClass.J, single(J)],
    [keyCodeClass.K, single(K)],
    [keyCodeClass.L, single(L)],
    [keyCodeClass.M, single(M)],
    [keyCodeClass.N, single(N)],
    [keyCodeClass.O, single(O)],
    [keyCodeClass.P, single(P)],
    [keyCodeClass.Q, single(Q)],
    [keyCodeClass.R, single(R)],
    [keyCodeClass.S, single(S)],
    [keyCodeClass.T, single(T)],
    [keyCodeClass.U, single(U)],
    [keyCodeClass.V, single(V)],
    [keyCodeClass.W, single(W)],
    [keyCodeClass.X, single(X)],
    [keyCodeClass.Y, single(Y)],
    [keyCodeClass.Z, single(Z)],
    [keyCodeClass.AE, single(AE)],
    [keyCodeClass.OE, single(OE)],
    [keyCodeClass.UE, single(UE)],
    [keyCodeClass.SharpS, single(SharpS)],

    // Digits
    [keyCodeClass.N1, single(N1)],
    [keyCodeClass.N2, single(N2)],
    [keyCodeClass.N3, single(N3)],
    [keyCodeClass.N4, single(N4)],
    [keyCodeClass.N5, single(N5)],
    [keyCodeClass.N6, single(N6)],
    [keyCodeClass.N7, single(N7)],
    [keyCodeClass.N8, single(N8)],
    [keyCodeClass.N9, single(N9)],
    [keyCodeClass.N0, single(N0)],

    // Symbols
    [keyCodeClass.Caret, single(Caret)],
    [keyCodeClass.Acute, single(Acute)],
    [keyCodeClass.Plus, single(Plus)],
    [keyCodeClass.Hash, single(Hash)],
    [keyCodeClass.LessThan, single(LessThan)],
    [keyCodeClass.Comma, single(Comma)],
    [keyCodeClass.Dot, single(Dot)],
    [keyCodeClass.Minus, single(Minus)],
    [keyCodeClass.LeftParen, combined(LShift, N8)],
    [keyCodeClass.RightParen, combined(LShift, N9)],
    [keyCodeClass.LeftBracket, combined(RAlt, N8)],
    [keyCodeClass.RightBracket, combined(RAlt, N9)],
    [keyCodeClass.LeftBrace, combined(RAlt, N7)],
    [keyCodeClass.RightBrace, combined(RAlt, N0)],
    [keyCodeClass.Degree, combined(LShift, Caret)],
    [keyCodeClass.ExclamationMark, combined(LShift, N1)],
    [keyCodeClass.DoubleQuote, combined(LShift, N2)],
    [keyCodeClass.SectionSign, combined(LShift, N3)],
    [keyCodeClass.Dollar, combined(LShift, N4)],
    [keyCodeClass.Percent, combined(LShift, N5)],
    [keyCodeClass.Ampersand, combined(LShift, N6)],
    [keyCodeClass.Slash, combined(LShift, N7)],
    [keyCodeClass.Equal, combined(LShift, N0)],
    [keyCodeClass.QuestionMark, combined(LShift, SharpS)],
    [keyCodeClass.Backslash, combined(RAlt, SharpS)],
    [keyCodeClass.Grave, combined(LShift, Acute)],
    [keyCodeClass.At, combined(RAlt, Q)],
    [keyCodeClass.Euro, combined(RAlt, E)],
    [keyCodeClass.Asterisk, combined(LShift, Plus)],
    [keyCodeClass.Tilde, combined(RAlt, Plus)],
    [keyCodeClass.Apostrophe, combined(LShift, Hash)],
    [keyCodeClass.GreaterThan, combined(LShift, LessThan)],
    [keyCodeClass.Pipe, combined(RAlt, LessThan)],
    [keyCodeClass.Semicolon, combined(LShift, Comma)],
    [keyCodeClass.Colon, combined(LShift, Dot)],
    [keyCodeClass.Underscore, combined(LShift, Minus)],

    // Whitespace and control keys
    [keyCodeClass.Enter, single(Enter)],
    [keyCodeClass.Escape, single(Escape)],
    [keyCodeClass.Backspace, single(Backspace)],
    [keyCodeClass.Tab, single(Tab)],
    [keyCodeClass.Space, single(Space)],
    [keyCodeClass.CapsLock, single(CapsLock)],

    // Navigation and editing
    [keyCodeClass.PrintScreen, single(PrintScreen)],
    [keyCodeClass.ScrollLock, single(ScrollLock)],
    [keyCodeClass.Pause, single(Pause)],
    [keyCodeClass.Insert, single(Insert)],
    [keyCodeClass.Home, single(Home)],
    [keyCodeClass.PageUp, single(PageUp)],
    [keyCodeClass.Delete, single(Delete)],
    [keyCodeClass.End, single(End)],
    [keyCodeClass.PageDown, single(PageDown)],
    [keyCodeClass.Right, single(Right)],
    [keyCodeClass.Left, single(Left)],
    [keyCodeClass.Down, single(Down)],
    [keyCodeClass.Up, single(Up)],

    // Function keys
    [keyCodeClass.F1, single(F1)],
    [keyCodeClass.F2, single(F2)],
    [keyCodeClass.F3, single(F3)],
    [keyCodeClass.F4, single(F4)],
    [keyCodeClass.F5, single(F5)],
    [keyCodeClass.F6, single(F6)],
    [keyCodeClass.F7, single(F7)],
    [keyCodeClass.F8, single(F8)],
    [keyCodeClass.F9, single(F9)],
    [keyCodeClass.F10, single(F10)],
    [keyCodeClass.F11, single(F11)],
    [keyCodeClass.F12, single(F12)],
    [keyCodeClass.F13, single(F13)],
    [keyCodeClass.F14, single(F14)],
    [keyCodeClass.F15, single(F15)],
    [keyCodeClass.F16, single(F16)],
    [keyCodeClass.F17, single(F17)],
    [keyCodeClass.F18, single(F18)],
    [keyCodeClass.F19, single(F19)],
    [keyCodeClass.F20, single(F20)],
    [keyCodeClass.F21, single(F21)],
    [keyCodeClass.F22, single(F22)],
    [keyCodeClass.F23, single(F23)],
    [keyCodeClass.F24, single(F24)],

    // Keypad
    [keyCodeClass.NumLock, single(NumLock)],
    [keyCodeClass.KpSlash, single(KpSlash)],
    [keyCodeClass.KpAsterisk, single(KpAsterisk)],
    [keyCodeClass.KpMinus, single(KpMinus)],
    [keyCodeClass.KpPlus, single(KpPlus)],
    [keyCodeClass.KpEnter, single(KpEnter)],
    [keyCodeClass.KpDot, single(KpDot)],
    [keyCodeClass.KpEqual, single(KpEqual)],
    [keyCodeClass.Kp1, single(Kp1)],
    [keyCodeClass.Kp2, single(Kp2)],
    [keyCodeClass.Kp3, single(Kp3)],
    [keyCodeClass.Kp4, single(Kp4)],
    [keyCodeClass.Kp5, single(Kp5)],
    [keyCodeClass.Kp6, single(Kp6)],
    [keyCodeClass.Kp7, single(Kp7)],
    [keyCodeClass.Kp8, single(Kp8)],
    [keyCodeClass.Kp9, single(Kp9)],
    [keyCodeClass.Kp0, single(Kp0)],
    [keyCodeClass.KpComma, single(KpComma)],
    [keyCodeClass.KpEqualSign, single(KpEqualSign)],
    [keyCodeClass.Kp00, single(Kp00)],
    [keyCodeClass.Kp000, single(Kp000)],
    [keyCodeClass.KpThousandsSeparator, single(KpThousandsSeparator)],
    [keyCodeClass.KpDecimalSeparator, single(KpDecimalSeparator)],
    [keyCodeClass.KpCurrencyUnit, single(KpCurrencyUnit)],
    [keyCodeClass.KpCurrencySubunit, single(KpCurrencySubunit)],
    [keyCodeClass.KpLeftParen, single(KpLeftParen)],
    [keyCodeClass.KpRightParen, single(KpRightParen)],
    [keyCodeClass.KpLeftBrace, single(KpLeftBrace)],
    [keyCodeClass.KpRightBrace, single(KpRightBrace)],
    [keyCodeClass.KpTab, single(KpTab)],
    [keyCodeClass.KpBackspace, single(KpBackspace)],
    [keyCodeClass.KpA, single(KpA)],
    [keyCodeClass.KpB, single(KpB)],
    [keyCodeClass.KpC, single(KpC)],
    [keyCodeClass.KpD, single(KpD)],
    [keyCodeClass.KpE, single(KpE)],
    [keyCodeClass.KpF, single(KpF)],
    [keyCodeClass.KpXor, single(KpXor)],
    [keyCodeClass.KpCaret, single(KpCaret)],
    [keyCodeClass.KpPercent, single(KpPercent)],
    [keyCodeClass.KpLessThan, single(KpLessThan)],
    [keyCodeClass.KpGreaterThan, single(KpGreaterThan)],
    [keyCodeClass.KpAmpersand, single(KpAmpersand)],
    [keyCodeClass.KpDoubleAmpersand, single(KpDoubleAmpersand)],
    [keyCodeClass.KpPipe, single(KpPipe)],
    [keyCodeClass.KpDoublePipe, single(KpDoublePipe)],
    [keyCodeClass.KpColon, single(KpColon)],
    [keyCodeClass.KpHash, single(KpHash)],
    [keyCodeClass.KpSpace, single(KpSpace)],
    [keyCodeClass.KpAt, single(KpAt)],
    [keyCodeClass.KpExclamation, single(KpExclamation)],
    [keyCodeClass.KpMemoryStore, single(KpMemoryStore)],
    [keyCodeClass.KpMemoryRecall, single(KpMemoryRecall)],
    [keyCodeClass.KpMemoryClear, single(KpMemoryClear)],
    [keyCodeClass.KpMemoryAdd, single(KpMemoryAdd)],
    [keyCodeClass.KpMemorySubtract, single(KpMemorySubtract)],
    [keyCodeClass.KpMemoryMultiply, single(KpMemoryMultiply)],
    [keyCodeClass.KpMemoryDivide, single(KpMemoryDivide)],
    [keyCodeClass.KpPlusMinus, single(KpPlusMinus)],
    [keyCodeClass.KpClear, single(KpClear)],
    [keyCodeClass.KpClearEntry, single(KpClearEntry)],
    [keyCodeClass.KpBinary, single(KpBinary)],
    [keyCodeClass.KpOctal, single(KpOctal)],
    [keyCodeClass.KpDecimal, single(KpDecimal)],
    [keyCodeClass.KpHexadecimal, single(KpHexadecimal)],

    // Miscellaneous
    [keyCodeClass.Application, single(Application)],
    [keyCodeClass.Power, single(Power)],
    [keyCodeClass.Execute, single(Execute)],
    [keyCodeClass.Help, single(Help)],
    [keyCodeClass.Menu, single(Menu)],
    [keyCodeClass.Select, single(Select)],
    [keyCodeClass.Stop, single(Stop)],
    [keyCodeClass.Again, single(Again)],
    [keyCodeClass.Undo, single(Undo)],
    [keyCodeClass.Cut, single(Cut)],
    [keyCodeClass.Copy, single(Copy)],
    [keyCodeClass.Paste, single(Paste)],
    [keyCodeClass.Find, single(Find)],
    [keyCodeClass.Mute, single(Mute)],
    [keyCodeClass.VolumeUp, single(VolumeUp)],
    [keyCodeClass.VolumeDown, single(VolumeDown)],
    [keyCodeClass.LockingCapsLock, single(LockingCapsLock)],
    [keyCodeClass.LockingNumLock, single(LockingNumLock)],
    [keyCodeClass.LockingScrollLock, single(LockingScrollLock)],

    // International and language keys
    [keyCodeClass.International1, single(International1)],
    [keyCodeClass.International2, single(International2)],
    [keyCodeClass.International3, single(International3)],
    [keyCodeClass.International4, single(International4)],
    [keyCodeClass.International5, single(International5)],
    [keyCodeClass.International6, single(International6)],
    [keyCodeClass.International7, single(International7)],
    [keyCodeClass.International8, single(International8)],
    [keyCodeClass.International9, single(International9)],
    [keyCodeClass.Lang1, single(Lang1)],
    [keyCodeClass.Lang2, single(Lang2)],
    [keyCodeClass.Lang3, single(Lang3)],
    [keyCodeClass.Lang4, single(Lang4)],
    [keyCodeClass.Lang5, single(Lang5)],
    [keyCodeClass.Lang6, single(Lang6)],
    [keyCodeClass.Lang7, single(Lang7)],
    [keyCodeClass.Lang8, single(Lang8)],
    [keyCodeClass.Lang9, single(Lang9)],
  ]);
}
