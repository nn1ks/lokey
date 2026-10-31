import { KeyCode } from "../index";
import type { KeyCodeEncoding } from "./types";
import { englishUsKeyCodes } from "./english-us";

/**
 * German (QWERTZ) layout. HID usage IDs describe physical key positions, so
 * characters that a German layout only produces with Shift or AltGr are
 * encoded as a sequence of the modifier followed by the key.
 */
export const germanKeyCodes = new Map<KeyCode, KeyCodeEncoding>(
  englishUsKeyCodes,
);

// Letters: German swaps Y and Z.
germanKeyCodes.set(KeyCode.Y, { kind: "simple", keyCode: 0x1d });
germanKeyCodes.set(KeyCode.Z, { kind: "simple", keyCode: 0x1c });

// Symbols
// "-" is on the key at the US "/" position.
germanKeyCodes.set(KeyCode.Minus, { kind: "simple", keyCode: 0x38 });
// "=" is Shift+0.
germanKeyCodes.set(KeyCode.Equal, {
  kind: "sequence",
  modifier: 0xe1,
  keyCode: 0x27,
});
// "[" is AltGr+8.
germanKeyCodes.set(KeyCode.LeftBracket, {
  kind: "sequence",
  modifier: 0xe6,
  keyCode: 0x25,
});
// "]" is AltGr+9.
germanKeyCodes.set(KeyCode.RightBracket, {
  kind: "sequence",
  modifier: 0xe6,
  keyCode: 0x26,
});
// "\" is AltGr+ß.
germanKeyCodes.set(KeyCode.Backslash, {
  kind: "sequence",
  modifier: 0xe6,
  keyCode: 0x2d,
});
// "#" is on the key at the US "\" position.
germanKeyCodes.set(KeyCode.Hash, { kind: "simple", keyCode: 0x31 });
// ";" is Shift+,
germanKeyCodes.set(KeyCode.Semicolon, {
  kind: "sequence",
  modifier: 0xe1,
  keyCode: 0x36,
});
// "'" is Shift+#
germanKeyCodes.set(KeyCode.Apostrophe, {
  kind: "sequence",
  modifier: 0xe1,
  keyCode: 0x31,
});
// "`" is Shift+´
germanKeyCodes.set(KeyCode.Grave, {
  kind: "sequence",
  modifier: 0xe1,
  keyCode: 0x2e,
});
// "/" is Shift+7
germanKeyCodes.set(KeyCode.Slash, {
  kind: "sequence",
  modifier: 0xe1,
  keyCode: 0x24,
});
