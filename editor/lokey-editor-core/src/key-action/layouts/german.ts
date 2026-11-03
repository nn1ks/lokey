import type { KeyCode } from "../actions/key-code";
import type { KeyCodeEncoding } from "./types";
import { englishUsKeyCodes } from "./english-us";

/**
 * German (QWERTZ) layout. HID usage IDs describe physical key positions, so
 * characters that a German layout only produces with Shift or AltGr are
 * encoded as a sequence of the modifier followed by the key.
 */
export function germanKeyCodes(
  keyCodeClass: typeof import("../actions/key-code").KeyCode,
): Map<KeyCode, KeyCodeEncoding> {
  const keyCodes = englishUsKeyCodes(keyCodeClass);

  // Letters: German swaps Y and Z.
  keyCodes.set(keyCodeClass.Y, { kind: "simple", keyCode: 0x1d });
  keyCodes.set(keyCodeClass.Z, { kind: "simple", keyCode: 0x1c });

  // Symbols
  // "-" is on the key at the US "/" position.
  keyCodes.set(keyCodeClass.Minus, { kind: "simple", keyCode: 0x38 });
  // "=" is Shift+0.
  keyCodes.set(keyCodeClass.Equal, {
    kind: "sequence",
    modifier: 0xe1,
    keyCode: 0x27,
  });
  // "[" is AltGr+8.
  keyCodes.set(keyCodeClass.LeftBracket, {
    kind: "sequence",
    modifier: 0xe6,
    keyCode: 0x25,
  });
  // "]" is AltGr+9.
  keyCodes.set(keyCodeClass.RightBracket, {
    kind: "sequence",
    modifier: 0xe6,
    keyCode: 0x26,
  });
  // "\" is AltGr+ß.
  keyCodes.set(keyCodeClass.Backslash, {
    kind: "sequence",
    modifier: 0xe6,
    keyCode: 0x2d,
  });
  // "#" is on the key at the US "\" position.
  keyCodes.set(keyCodeClass.Hash, { kind: "simple", keyCode: 0x31 });
  // ";" is Shift+,
  keyCodes.set(keyCodeClass.Semicolon, {
    kind: "sequence",
    modifier: 0xe1,
    keyCode: 0x36,
  });
  // "'" is Shift+#
  keyCodes.set(keyCodeClass.Apostrophe, {
    kind: "sequence",
    modifier: 0xe1,
    keyCode: 0x31,
  });
  // "`" is Shift+´
  keyCodes.set(keyCodeClass.Grave, {
    kind: "sequence",
    modifier: 0xe1,
    keyCode: 0x2e,
  });
  // "/" is Shift+7
  keyCodes.set(keyCodeClass.Slash, {
    kind: "sequence",
    modifier: 0xe1,
    keyCode: 0x24,
  });
  return keyCodes;
}
