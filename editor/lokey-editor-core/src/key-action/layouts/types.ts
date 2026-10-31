/**
 * How a key code is encoded for a layout: either a single HID usage, or a
 * sequence of a modifier followed by a key.
 */
export type KeyCodeEncoding =
  | { kind: "simple"; keyCode: number }
  | { kind: "sequence"; modifier: number; keyCode: number };
