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
export declare const DeviceType: {
  readonly OTHER: 0;
  readonly KEYBOARD: 1;
  readonly MOUSE: 2;
  readonly MIDI_CONTROLLER: 3;
};
export type DeviceType = (typeof DeviceType)[keyof typeof DeviceType];
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
  get maxPayloadSize(): number;
  set maxPayloadSize(value: number);
  get deviceId(): number;
  set deviceId(value: number);
  get deviceName(): string;
  set deviceName(value: string);
  get deviceType(): DeviceType;
  set deviceType(value: DeviceType);
  toString(): string;
}
