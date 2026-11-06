export type RawKeyCode = { name: string; code: number };

export function rawKeyCode(name: string, code: number): RawKeyCode {
  return { name, code };
}

export type KeyCodeEncoding =
  | { kind: "single"; rawKeyCode: RawKeyCode }
  | { kind: "combined"; rawKeyCode1: RawKeyCode; rawKeyCode2: RawKeyCode };

export function single(rawKeyCode: RawKeyCode): KeyCodeEncoding {
  return { kind: "single", rawKeyCode };
}

export function combined(
  rawKeyCode1: RawKeyCode,
  rawKeyCode2: RawKeyCode,
): KeyCodeEncoding {
  return { kind: "combined", rawKeyCode1, rawKeyCode2 };
}
