import * as $ from "capnp-es";
export declare const _capnpFileId = 12092723869270611711n;
export declare class GetLedArrayHookCapabilitiesRequest extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  toString(): string;
}
export declare class GetLedArrayHookCapabilitiesResponse extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  get hooks(): bigint;
  set hooks(value: bigint);
  toString(): string;
}
export declare class GetLedArrayHooksRequest extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  toString(): string;
}
export declare class GetLedArrayHooksResponse extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  get hooks(): bigint;
  set hooks(value: bigint);
  toString(): string;
}
export declare class SetLedArrayHooksRequest extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  get hooks(): bigint;
  set hooks(value: bigint);
  toString(): string;
}
