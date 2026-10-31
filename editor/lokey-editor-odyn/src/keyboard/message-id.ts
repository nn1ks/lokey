export * from "../generated/odyn_keyboard_message_id.js";

// capnp-es currently does not emit top-level schema constants.
export const getNumberOfKeysRequest = 0x100;
export const getNumberOfKeysResponse = 0x101;
export const getLayoutRequest = 0x102;
export const getLayoutResponse = 0x103;
export const getAvailableActionsRequest = 0x104;
export const getAvailableActionsResponse = 0x105;
export const getActiveActionEntriesRequest = 0x106;
export const getActiveActionEntriesResponse = 0x107;
export const addActionEntriesRequest = 0x108;
export const updateLayoutRequest = 0x109;
