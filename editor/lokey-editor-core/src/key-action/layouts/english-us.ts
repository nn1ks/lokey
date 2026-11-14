import { KeyCodeValue } from "../key-code-value";
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
const Y = rawKeyCode("Y", 0x1c);
const Z = rawKeyCode("Z", 0x1d);

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
const Minus = rawKeyCode("Minus", 0x2d);
const Equal = rawKeyCode("Equal", 0x2e);
const LeftBracket = rawKeyCode("LeftBracket", 0x2f);
const RightBracket = rawKeyCode("RightBracket", 0x30);
const Backslash = rawKeyCode("Backslash", 0x31);
const Hash = rawKeyCode("Hash", 0x32);
const Semicolon = rawKeyCode("Semicolon", 0x33);
const Apostrophe = rawKeyCode("Apostrophe", 0x34);
const Grave = rawKeyCode("Grave", 0x35);
const Comma = rawKeyCode("Comma", 0x36);
const Dot = rawKeyCode("Dot", 0x37);
const Slash = rawKeyCode("Slash", 0x38);

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
const NonUsBackslash = rawKeyCode("NonUsBackslash", 0x64);
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

export function englishUsKeyCodes(): Map<KeyCodeValue, KeyCodeEncoding> {
  return new Map<KeyCodeValue, KeyCodeEncoding>([
    // Modifiers
    [KeyCodeValue.LControl, single(LControl)],
    [KeyCodeValue.RControl, single(RControl)],
    [KeyCodeValue.LShift, single(LShift)],
    [KeyCodeValue.RShift, single(RShift)],
    [KeyCodeValue.LAlt, single(LAlt)],
    [KeyCodeValue.RAlt, single(RAlt)],
    [KeyCodeValue.LGui, single(LGui)],
    [KeyCodeValue.RGui, single(RGui)],

    // Letters
    [KeyCodeValue.A, single(A)],
    [KeyCodeValue.B, single(B)],
    [KeyCodeValue.C, single(C)],
    [KeyCodeValue.D, single(D)],
    [KeyCodeValue.E, single(E)],
    [KeyCodeValue.F, single(F)],
    [KeyCodeValue.G, single(G)],
    [KeyCodeValue.H, single(H)],
    [KeyCodeValue.I, single(I)],
    [KeyCodeValue.J, single(J)],
    [KeyCodeValue.K, single(K)],
    [KeyCodeValue.L, single(L)],
    [KeyCodeValue.M, single(M)],
    [KeyCodeValue.N, single(N)],
    [KeyCodeValue.O, single(O)],
    [KeyCodeValue.P, single(P)],
    [KeyCodeValue.Q, single(Q)],
    [KeyCodeValue.R, single(R)],
    [KeyCodeValue.S, single(S)],
    [KeyCodeValue.T, single(T)],
    [KeyCodeValue.U, single(U)],
    [KeyCodeValue.V, single(V)],
    [KeyCodeValue.W, single(W)],
    [KeyCodeValue.X, single(X)],
    [KeyCodeValue.Y, single(Y)],
    [KeyCodeValue.Z, single(Z)],

    // Digits
    [KeyCodeValue.N1, single(N1)],
    [KeyCodeValue.N2, single(N2)],
    [KeyCodeValue.N3, single(N3)],
    [KeyCodeValue.N4, single(N4)],
    [KeyCodeValue.N5, single(N5)],
    [KeyCodeValue.N6, single(N6)],
    [KeyCodeValue.N7, single(N7)],
    [KeyCodeValue.N8, single(N8)],
    [KeyCodeValue.N9, single(N9)],
    [KeyCodeValue.N0, single(N0)],

    // Symbols
    [KeyCodeValue.Minus, single(Minus)],
    [KeyCodeValue.Equal, single(Equal)],
    [KeyCodeValue.LeftBracket, single(LeftBracket)],
    [KeyCodeValue.RightBracket, single(RightBracket)],
    [KeyCodeValue.Backslash, single(Backslash)],
    [KeyCodeValue.Hash, single(Hash)],
    [KeyCodeValue.Semicolon, single(Semicolon)],
    [KeyCodeValue.Apostrophe, single(Apostrophe)],
    [KeyCodeValue.Grave, single(Grave)],
    [KeyCodeValue.Comma, single(Comma)],
    [KeyCodeValue.Dot, single(Dot)],
    [KeyCodeValue.Slash, single(Slash)],
    [KeyCodeValue.Tilde, combined(LShift, Grave)],
    [KeyCodeValue.ExclamationMark, combined(LShift, N1)],
    [KeyCodeValue.At, combined(LShift, N2)],
    [KeyCodeValue.Hash, combined(LShift, N3)],
    [KeyCodeValue.Dollar, combined(LShift, N4)],
    [KeyCodeValue.Percent, combined(LShift, N5)],
    [KeyCodeValue.Caret, combined(LShift, N6)],
    [KeyCodeValue.Ampersand, combined(LShift, N7)],
    [KeyCodeValue.Asterisk, combined(LShift, N8)],
    [KeyCodeValue.LeftParen, combined(LShift, N9)],
    [KeyCodeValue.RightParen, combined(LShift, N0)],
    [KeyCodeValue.Underscore, combined(LShift, Minus)],
    [KeyCodeValue.Plus, combined(LShift, Equal)],
    [KeyCodeValue.LeftBrace, combined(LShift, LeftBracket)],
    [KeyCodeValue.RightBrace, combined(LShift, RightBracket)],
    [KeyCodeValue.Pipe, combined(LShift, Backslash)],
    [KeyCodeValue.Colon, combined(LShift, Semicolon)],
    [KeyCodeValue.DoubleQuote, combined(LShift, Apostrophe)],
    [KeyCodeValue.LessThan, combined(LShift, Comma)],
    [KeyCodeValue.GreaterThan, combined(LShift, Dot)],
    [KeyCodeValue.QuestionMark, combined(LShift, Slash)],

    // Whitespace and control keys
    [KeyCodeValue.Enter, single(Enter)],
    [KeyCodeValue.Escape, single(Escape)],
    [KeyCodeValue.Backspace, single(Backspace)],
    [KeyCodeValue.Tab, single(Tab)],
    [KeyCodeValue.Space, single(Space)],
    [KeyCodeValue.CapsLock, single(CapsLock)],

    // Navigation and editing
    [KeyCodeValue.PrintScreen, single(PrintScreen)],
    [KeyCodeValue.ScrollLock, single(ScrollLock)],
    [KeyCodeValue.Pause, single(Pause)],
    [KeyCodeValue.Insert, single(Insert)],
    [KeyCodeValue.Home, single(Home)],
    [KeyCodeValue.PageUp, single(PageUp)],
    [KeyCodeValue.Delete, single(Delete)],
    [KeyCodeValue.End, single(End)],
    [KeyCodeValue.PageDown, single(PageDown)],
    [KeyCodeValue.Right, single(Right)],
    [KeyCodeValue.Left, single(Left)],
    [KeyCodeValue.Down, single(Down)],
    [KeyCodeValue.Up, single(Up)],

    // Function keys
    [KeyCodeValue.F1, single(F1)],
    [KeyCodeValue.F2, single(F2)],
    [KeyCodeValue.F3, single(F3)],
    [KeyCodeValue.F4, single(F4)],
    [KeyCodeValue.F5, single(F5)],
    [KeyCodeValue.F6, single(F6)],
    [KeyCodeValue.F7, single(F7)],
    [KeyCodeValue.F8, single(F8)],
    [KeyCodeValue.F9, single(F9)],
    [KeyCodeValue.F10, single(F10)],
    [KeyCodeValue.F11, single(F11)],
    [KeyCodeValue.F12, single(F12)],
    [KeyCodeValue.F13, single(F13)],
    [KeyCodeValue.F14, single(F14)],
    [KeyCodeValue.F15, single(F15)],
    [KeyCodeValue.F16, single(F16)],
    [KeyCodeValue.F17, single(F17)],
    [KeyCodeValue.F18, single(F18)],
    [KeyCodeValue.F19, single(F19)],
    [KeyCodeValue.F20, single(F20)],
    [KeyCodeValue.F21, single(F21)],
    [KeyCodeValue.F22, single(F22)],
    [KeyCodeValue.F23, single(F23)],
    [KeyCodeValue.F24, single(F24)],

    // Keypad
    [KeyCodeValue.NumLock, single(NumLock)],
    [KeyCodeValue.KpSlash, single(KpSlash)],
    [KeyCodeValue.KpAsterisk, single(KpAsterisk)],
    [KeyCodeValue.KpMinus, single(KpMinus)],
    [KeyCodeValue.KpPlus, single(KpPlus)],
    [KeyCodeValue.KpEnter, single(KpEnter)],
    [KeyCodeValue.KpDot, single(KpDot)],
    [KeyCodeValue.KpEqual, single(KpEqual)],
    [KeyCodeValue.Kp1, single(Kp1)],
    [KeyCodeValue.Kp2, single(Kp2)],
    [KeyCodeValue.Kp3, single(Kp3)],
    [KeyCodeValue.Kp4, single(Kp4)],
    [KeyCodeValue.Kp5, single(Kp5)],
    [KeyCodeValue.Kp6, single(Kp6)],
    [KeyCodeValue.Kp7, single(Kp7)],
    [KeyCodeValue.Kp8, single(Kp8)],
    [KeyCodeValue.Kp9, single(Kp9)],
    [KeyCodeValue.Kp0, single(Kp0)],
    [KeyCodeValue.KpComma, single(KpComma)],
    [KeyCodeValue.KpEqualSign, single(KpEqualSign)],
    [KeyCodeValue.Kp00, single(Kp00)],
    [KeyCodeValue.Kp000, single(Kp000)],
    [KeyCodeValue.KpThousandsSeparator, single(KpThousandsSeparator)],
    [KeyCodeValue.KpDecimalSeparator, single(KpDecimalSeparator)],
    [KeyCodeValue.KpCurrencyUnit, single(KpCurrencyUnit)],
    [KeyCodeValue.KpCurrencySubunit, single(KpCurrencySubunit)],
    [KeyCodeValue.KpLeftParen, single(KpLeftParen)],
    [KeyCodeValue.KpRightParen, single(KpRightParen)],
    [KeyCodeValue.KpLeftBrace, single(KpLeftBrace)],
    [KeyCodeValue.KpRightBrace, single(KpRightBrace)],
    [KeyCodeValue.KpTab, single(KpTab)],
    [KeyCodeValue.KpBackspace, single(KpBackspace)],
    [KeyCodeValue.KpA, single(KpA)],
    [KeyCodeValue.KpB, single(KpB)],
    [KeyCodeValue.KpC, single(KpC)],
    [KeyCodeValue.KpD, single(KpD)],
    [KeyCodeValue.KpE, single(KpE)],
    [KeyCodeValue.KpF, single(KpF)],
    [KeyCodeValue.KpXor, single(KpXor)],
    [KeyCodeValue.KpCaret, single(KpCaret)],
    [KeyCodeValue.KpPercent, single(KpPercent)],
    [KeyCodeValue.KpLessThan, single(KpLessThan)],
    [KeyCodeValue.KpGreaterThan, single(KpGreaterThan)],
    [KeyCodeValue.KpAmpersand, single(KpAmpersand)],
    [KeyCodeValue.KpDoubleAmpersand, single(KpDoubleAmpersand)],
    [KeyCodeValue.KpPipe, single(KpPipe)],
    [KeyCodeValue.KpDoublePipe, single(KpDoublePipe)],
    [KeyCodeValue.KpColon, single(KpColon)],
    [KeyCodeValue.KpHash, single(KpHash)],
    [KeyCodeValue.KpSpace, single(KpSpace)],
    [KeyCodeValue.KpAt, single(KpAt)],
    [KeyCodeValue.KpExclamation, single(KpExclamation)],
    [KeyCodeValue.KpMemoryStore, single(KpMemoryStore)],
    [KeyCodeValue.KpMemoryRecall, single(KpMemoryRecall)],
    [KeyCodeValue.KpMemoryClear, single(KpMemoryClear)],
    [KeyCodeValue.KpMemoryAdd, single(KpMemoryAdd)],
    [KeyCodeValue.KpMemorySubtract, single(KpMemorySubtract)],
    [KeyCodeValue.KpMemoryMultiply, single(KpMemoryMultiply)],
    [KeyCodeValue.KpMemoryDivide, single(KpMemoryDivide)],
    [KeyCodeValue.KpPlusMinus, single(KpPlusMinus)],
    [KeyCodeValue.KpClear, single(KpClear)],
    [KeyCodeValue.KpClearEntry, single(KpClearEntry)],
    [KeyCodeValue.KpBinary, single(KpBinary)],
    [KeyCodeValue.KpOctal, single(KpOctal)],
    [KeyCodeValue.KpDecimal, single(KpDecimal)],
    [KeyCodeValue.KpHexadecimal, single(KpHexadecimal)],

    // Miscellaneous
    [KeyCodeValue.NonUsBackslash, single(NonUsBackslash)],
    [KeyCodeValue.Application, single(Application)],
    [KeyCodeValue.Power, single(Power)],
    [KeyCodeValue.Execute, single(Execute)],
    [KeyCodeValue.Help, single(Help)],
    [KeyCodeValue.Menu, single(Menu)],
    [KeyCodeValue.Select, single(Select)],
    [KeyCodeValue.Stop, single(Stop)],
    [KeyCodeValue.Again, single(Again)],
    [KeyCodeValue.Undo, single(Undo)],
    [KeyCodeValue.Cut, single(Cut)],
    [KeyCodeValue.Copy, single(Copy)],
    [KeyCodeValue.Paste, single(Paste)],
    [KeyCodeValue.Find, single(Find)],
    [KeyCodeValue.Mute, single(Mute)],
    [KeyCodeValue.VolumeUp, single(VolumeUp)],
    [KeyCodeValue.VolumeDown, single(VolumeDown)],
    [KeyCodeValue.LockingCapsLock, single(LockingCapsLock)],
    [KeyCodeValue.LockingNumLock, single(LockingNumLock)],
    [KeyCodeValue.LockingScrollLock, single(LockingScrollLock)],

    // International and language keys
    [KeyCodeValue.International1, single(International1)],
    [KeyCodeValue.International2, single(International2)],
    [KeyCodeValue.International3, single(International3)],
    [KeyCodeValue.International4, single(International4)],
    [KeyCodeValue.International5, single(International5)],
    [KeyCodeValue.International6, single(International6)],
    [KeyCodeValue.International7, single(International7)],
    [KeyCodeValue.International8, single(International8)],
    [KeyCodeValue.International9, single(International9)],
    [KeyCodeValue.Lang1, single(Lang1)],
    [KeyCodeValue.Lang2, single(Lang2)],
    [KeyCodeValue.Lang3, single(Lang3)],
    [KeyCodeValue.Lang4, single(Lang4)],
    [KeyCodeValue.Lang5, single(Lang5)],
    [KeyCodeValue.Lang6, single(Lang6)],
    [KeyCodeValue.Lang7, single(Lang7)],
    [KeyCodeValue.Lang8, single(Lang8)],
    [KeyCodeValue.Lang9, single(Lang9)],
  ]);
}
