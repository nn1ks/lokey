@0xa81fe53a47c93ccd;

struct Message {
  sequenceNumber @0 :UInt32;
  id @1 :UInt32;
  payload @2 :Data;
}


struct GetDeviceCapabilitiesRequest {
}

struct GetDeviceCapabilitiesResponse {
  deviceId @0 :UInt32;
  maxPayloadSize @1 :UInt32;
}

struct SuccessResponse {
}

struct ErrorResponse {
  code @0 :UInt32;
}
