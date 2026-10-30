import * as $ from "capnp-es";
export declare const _capnpFileId = 10982841275687151197n;
export declare class KeyboardAction extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  get actionId(): number;
  set actionId(value: number);
  _adoptPayload(value: $.Orphan<$.Data>): void;
  _disownPayload(): $.Orphan<$.Data>;
  get payload(): $.Data;
  _hasPayload(): boolean;
  _initPayload(length: number): $.Data;
  set payload(value: $.Data);
  toString(): string;
}
export declare class KeyboardActionEntry extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  get entryId(): number;
  set entryId(value: number);
  _adoptAction(value: $.Orphan<KeyboardAction>): void;
  _disownAction(): $.Orphan<KeyboardAction>;
  get action(): KeyboardAction;
  _hasAction(): boolean;
  _initAction(): KeyboardAction;
  set action(value: KeyboardAction);
  toString(): string;
}
export declare class GetNumberOfKeysRequest extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  toString(): string;
}
export declare class GetNumberOfKeysResponse extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  get numKeys(): number;
  set numKeys(value: number);
  toString(): string;
}
export declare class GetLayoutRequest extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  toString(): string;
}
export declare class GetLayoutResponse extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  get startKeyIndex(): number;
  set startKeyIndex(value: number);
  _adoptActionEntryIds(value: $.Orphan<$.List<number>>): void;
  _disownActionEntryIds(): $.Orphan<$.List<number>>;
  get actionEntryIds(): $.List<number>;
  _hasActionEntryIds(): boolean;
  _initActionEntryIds(length: number): $.List<number>;
  set actionEntryIds(value: $.List<number>);
  get isFinalResponse(): boolean;
  set isFinalResponse(value: boolean);
  toString(): string;
}
export declare class GetAvailableActionsRequest extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  toString(): string;
}
export declare class GetAvailableActionsResponse extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  _adoptAvailableActionIds(value: $.Orphan<$.List<number>>): void;
  _disownAvailableActionIds(): $.Orphan<$.List<number>>;
  get availableActionIds(): $.List<number>;
  _hasAvailableActionIds(): boolean;
  _initAvailableActionIds(length: number): $.List<number>;
  set availableActionIds(value: $.List<number>);
  get isFinalResponse(): boolean;
  set isFinalResponse(value: boolean);
  toString(): string;
}
export declare class GetActiveActionEntriesRequest extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  toString(): string;
}
export declare class GetActiveActionEntriesResponse extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  static _Entries: $.ListCtor<KeyboardActionEntry>;
  _adoptEntries(value: $.Orphan<$.List<KeyboardActionEntry>>): void;
  _disownEntries(): $.Orphan<$.List<KeyboardActionEntry>>;
  get entries(): $.List<KeyboardActionEntry>;
  _hasEntries(): boolean;
  _initEntries(length: number): $.List<KeyboardActionEntry>;
  set entries(value: $.List<KeyboardActionEntry>);
  get isFinalResponse(): boolean;
  set isFinalResponse(value: boolean);
  toString(): string;
}
export declare class AddActionEntriesRequest extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  static _Entries: $.ListCtor<KeyboardActionEntry>;
  _adoptEntries(value: $.Orphan<$.List<KeyboardActionEntry>>): void;
  _disownEntries(): $.Orphan<$.List<KeyboardActionEntry>>;
  get entries(): $.List<KeyboardActionEntry>;
  _hasEntries(): boolean;
  _initEntries(length: number): $.List<KeyboardActionEntry>;
  set entries(value: $.List<KeyboardActionEntry>);
  toString(): string;
}
export declare class AddActionEntriesResponse extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  get success(): boolean;
  set success(value: boolean);
  toString(): string;
}
export declare class UpdateLayoutRequest extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  get startKeyIndex(): number;
  set startKeyIndex(value: number);
  _adoptActionEntryIds(value: $.Orphan<$.List<number>>): void;
  _disownActionEntryIds(): $.Orphan<$.List<number>>;
  get actionEntryIds(): $.List<number>;
  _hasActionEntryIds(): boolean;
  _initActionEntryIds(length: number): $.List<number>;
  set actionEntryIds(value: $.List<number>);
  toString(): string;
}
export declare class UpdateLayoutResponse extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  get success(): boolean;
  set success(value: boolean);
  toString(): string;
}
export declare class NoneAction extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  toString(): string;
}
export declare class TransparentAction extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  toString(): string;
}
export declare class KeyCodeAction extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  get keyCode(): number;
  set keyCode(value: number);
  toString(): string;
}
export declare class LayerAction extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  get layer(): number;
  set layer(value: number);
  toString(): string;
}
export declare class PerLayerAction extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  _adoptActionEntryIds(value: $.Orphan<$.List<number>>): void;
  _disownActionEntryIds(): $.Orphan<$.List<number>>;
  get actionEntryIds(): $.List<number>;
  _hasActionEntryIds(): boolean;
  _initActionEntryIds(length: number): $.List<number>;
  set actionEntryIds(value: $.List<number>);
  _adoptLayers(value: $.Orphan<$.List<number>>): void;
  _disownLayers(): $.Orphan<$.List<number>>;
  get layers(): $.List<number>;
  _hasLayers(): boolean;
  _initLayers(length: number): $.List<number>;
  set layers(value: $.List<number>);
  toString(): string;
}
export declare class HoldTapAction extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  get holdActionEntryId(): number;
  set holdActionEntryId(value: number);
  get tapActionEntryId(): number;
  set tapActionEntryId(value: number);
  get tappingTerm(): number;
  set tappingTerm(value: number);
  toString(): string;
}
