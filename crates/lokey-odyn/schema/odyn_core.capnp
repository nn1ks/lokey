@0xa81fe53a47c93ccd;

struct Message {
  sequenceNumber @0 :UInt32;
  id @1 :UInt32;
  payload @2 :Data;
}


struct SuccessResponse {
}

struct ErrorResponse {
  code @0 :UInt32;
}

enum DeviceType {
    other @0;
    keyboard @1;
    mouse @2;
    midiController @3;
}

struct GetDeviceCapabilitiesRequest {
}

struct GetDeviceCapabilitiesResponse {
  maxPayloadSize @0 :UInt32;
  deviceId @1 :UInt32;
  deviceName @2 :Text;
  deviceType @3 :DeviceType;
}
