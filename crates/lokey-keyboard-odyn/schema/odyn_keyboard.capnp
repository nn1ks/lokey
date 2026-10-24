@0x986ae3f35dd4a65d;

struct KeyboardAction {
  actionId @0 :UInt32;
  payload @1 :Data;
}

struct KeyboardActionEntry {
  entryId @0 :UInt32;
  action @1 :KeyboardAction;
}

# Messages

struct GetNumberOfKeysRequest {
}

struct GetNumberOfKeysResponse {
  numKeys @0 :UInt32;
}

struct GetLayoutRequest {
}

struct GetLayoutResponse {
  startKeyIndex @0 :UInt32;
  actionEntryIds @1 :List(UInt32);
  isFinalResponse @2 :Bool;
}

struct GetAvailableActionsRequest {
}

struct GetAvailableActionsResponse {
  availableActionIds @0 :List(UInt32);
  isFinalResponse @1 :Bool;
}

struct GetActiveActionEntriesRequest {
}

struct GetActiveActionEntriesResponse {
  entries @0 :List(KeyboardActionEntry);
  isFinalResponse @1 :Bool;
}

struct AddActionEntriesRequest {
  entries @0 :List(KeyboardActionEntry);
}

struct AddActionEntriesResponse {
  success @0 :Bool;
}

struct UpdateLayoutRequest {
  startKeyIndex @0 :UInt32;
  actionEntryIds @1 :List(UInt32);
}

struct UpdateLayoutResponse {
  success @0 :Bool;
}

# Actions

struct NoneAction {
}

struct TransparentAction {
}

struct KeyCodeAction {
  keyCode @0 :UInt32;
}

struct LayerAction {
  layer @0 :UInt32;
}

struct PerLayerAction {
  actionEntryIds @0 :List(UInt32);
  layers @1 :List(UInt32);
}

struct HoldTapAction {
  holdActionEntryId @0 :UInt32;
  tapActionEntryId @1 :UInt32;
  tappingTerm @2 :UInt32;
}
