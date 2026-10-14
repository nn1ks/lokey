import * as $ from "capnp-es";
export declare const _capnpFileId = 12114653561125158093n;
export declare class Message extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  get sequenceNumber(): number;
  set sequenceNumber(value: number);
  get id(): number;
  set id(value: number);
  _adoptPayload(value: $.Orphan<$.Data>): void;
  _disownPayload(): $.Orphan<$.Data>;
  get payload(): $.Data;
  _hasPayload(): boolean;
  _initPayload(length: number): $.Data;
  set payload(value: $.Data);
  toString(): string;
}
export declare class GetDeviceCapabilitiesRequest extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  toString(): string;
}
export declare class GetDeviceCapabilitiesResponse extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  get deviceId(): number;
  set deviceId(value: number);
  get maxPayloadSize(): number;
  set maxPayloadSize(value: number);
  toString(): string;
}
export declare class SuccessResponse extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  toString(): string;
}
export declare class ErrorResponse extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  get code(): number;
  set code(value: number);
  toString(): string;
}
