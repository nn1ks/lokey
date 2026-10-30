@0xa7d1fc4a02c316ff;

struct GetLedArrayHookCapabilitiesRequest {
}

struct GetLedArrayHookCapabilitiesResponse {
  hooks @0 :UInt64;
}

struct GetLedArrayHooksRequest {
}

struct GetLedArrayHooksResponse {
  hooks @0 :UInt64;
}

struct SetLedArrayHooksRequest {
  hooks @0 :UInt64;
}
