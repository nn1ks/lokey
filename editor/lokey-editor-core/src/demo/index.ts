import * as capnp from "capnp-es";
import { defaultState } from "./default-state";
import { State, numKeys, ActionId, ActionEntryId } from "./shared";
import {
  GetDeviceCapabilitiesResponse,
  SuccessResponse,
} from "lokey-editor-odyn/core";
import {
  AddActionEntriesRequest,
  GetActiveActionEntriesResponse,
  GetAvailableActionsResponse,
  GetLayoutResponse,
  GetNumberOfKeysResponse,
  KeyboardActionEntry,
  UpdateLayoutRequest,
} from "lokey-editor-odyn/keyboard";
import {
  GetLedArrayHookCapabilitiesResponse,
  GetLedArrayHooksResponse,
  SetLedArrayHooksRequest,
} from "lokey-editor-odyn/led-array";
import * as ledArrayHookId from "lokey-editor-odyn/led-array/hook-id";
import * as ledArrayMessageId from "lokey-editor-odyn/led-array/message-id";
import * as keyboardActionId from "lokey-editor-odyn/keyboard/action-id";
import * as keyboardMessageId from "lokey-editor-odyn/keyboard/message-id";
import * as coreMessageId from "lokey-editor-odyn/core/message-id";
import { Message } from "lokey-editor-odyn/core";

export class DemoMessageHandler {
  readonly #queue: Message[] = [];
  #waiter: ((message: Message) => void) | undefined;
  state = defaultState();

  async send(message: Message): Promise<void> {
    switch (message.id) {
      case coreMessageId.getDeviceCapabilitiesRequest: {
        this.#handleGetDeviceCapabilitiesRequest(
          message.sequenceNumber,
          message.payload,
        );
        break;
      }
      case keyboardMessageId.getNumberOfKeysRequest: {
        this.#handleGetNumberOfKeysRequest(
          message.sequenceNumber,
          message.payload,
        );
        break;
      }
      case keyboardMessageId.getAvailableActionsRequest: {
        this.#handleGetAvailableActionsRequest(
          message.sequenceNumber,
          message.payload,
        );
        break;
      }
      case keyboardMessageId.getLayoutRequest: {
        this.#handleGetLayoutRequest(message.sequenceNumber, message.payload);
        break;
      }
      case keyboardMessageId.getActiveActionEntriesRequest: {
        this.#handleGetActiveActionEntriesRequest(
          message.sequenceNumber,
          message.payload,
        );
        break;
      }
      case keyboardMessageId.addActionEntriesRequest: {
        this.#handleAddActionEntriesRequest(
          message.sequenceNumber,
          message.payload,
        );
        break;
      }
      case keyboardMessageId.updateLayoutRequest: {
        this.#handleUpdateLayoutRequest(
          message.sequenceNumber,
          message.payload,
        );
        break;
      }
      case ledArrayMessageId.getLedArrayHookCapabilitiesRequest: {
        this.#handleGetLedArrayHookCapabilitiesRequest(
          message.sequenceNumber,
          message.payload,
        );
        break;
      }
      case ledArrayMessageId.getLedArrayHooksRequest: {
        this.#handleGetLedArrayHooksRequest(
          message.sequenceNumber,
          message.payload,
        );
        break;
      }
      case ledArrayMessageId.setLedArrayHooksRequest: {
        this.#handleSetLedArrayHooksRequest(
          message.sequenceNumber,
          message.payload,
        );
        break;
      }
      default:
        break;
    }
  }

  async receive(): Promise<Message> {
    const queued = this.#queue.shift();
    if (queued) {
      return queued;
    }

    return await new Promise<Message>((resolve) => {
      this.#waiter = resolve;
    });
  }

  #publish(message: Message) {
    const waiter = this.#waiter;
    if (waiter) {
      this.#waiter = undefined;
      waiter(message);
      return;
    }

    this.#queue.push(message);
  }

  #buildMessage<T extends capnp.Struct>(
    sequenceNumber: number,
    messageId: number,
    ResponseClass: capnp.StructCtor<T>,
    setMessage: (message: T) => void,
  ): Message {
    const capnpMessagePayload = new capnp.Message();
    const messagePayload = capnpMessagePayload.initRoot(ResponseClass);
    setMessage(messagePayload);

    const capnpMessage = new capnp.Message();
    const message = capnpMessage.initRoot(Message);
    message.sequenceNumber = sequenceNumber;
    message.id = messageId;
    const serializedResponse = new Uint8Array(
      capnpMessagePayload.toArrayBuffer(),
    );
    message
      ._initPayload(serializedResponse.byteLength)
      .copyBuffer(serializedResponse);

    return message;
  }

  #handleGetDeviceCapabilitiesRequest(
    sequenceNumber: number,
    _payload: capnp.Data,
  ) {
    const responseMessage = this.#buildMessage(
      sequenceNumber,
      coreMessageId.getDeviceCapabilitiesResponse,
      GetDeviceCapabilitiesResponse,
      (response) => {
        response.deviceId = 0x1000_0001;
        response.maxPayloadSize = 32;
      },
    );

    this.#publish(responseMessage);
  }

  #handleGetNumberOfKeysRequest(sequenceNumber: number, _payload: capnp.Data) {
    const responseMessage = this.#buildMessage(
      sequenceNumber,
      keyboardMessageId.getNumberOfKeysResponse,
      GetNumberOfKeysResponse,
      (response) => {
        response.numKeys = numKeys;
      },
    );

    this.#publish(responseMessage);
  }

  #handleGetAvailableActionsRequest(
    sequenceNumber: number,
    _payload: capnp.Data,
  ) {
    const availableActionIds = [
      keyboardActionId.none,
      keyboardActionId.transparent,
      keyboardActionId.keyCode,
      keyboardActionId.layer,
      keyboardActionId.perLayer,
      keyboardActionId.holdTap,
    ];
    const responseMessage = this.#buildMessage(
      sequenceNumber,
      keyboardMessageId.getAvailableActionsResponse,
      GetAvailableActionsResponse,
      (response) => {
        const actionIds = response._initAvailableActionIds(
          availableActionIds.length,
        );
        availableActionIds.forEach((actionId, index) =>
          actionIds.set(index, actionId),
        );
        response.isFinalResponse = true;
      },
    );

    this.#publish(responseMessage);
  }

  #handleGetLayoutRequest(sequenceNumber: number, _payload: capnp.Data) {
    const responseMessage = this.#buildMessage(
      sequenceNumber,
      keyboardMessageId.getLayoutResponse,
      GetLayoutResponse,
      (response) => {
        response.startKeyIndex = 0;
        const actionEntryIds = response._initActionEntryIds(
          this.state.keyboardLayout.length,
        );
        this.state.keyboardLayout.forEach((actionEntryId, index) => {
          actionEntryIds.set(index, actionEntryId);
        });
        response.isFinalResponse = true;
      },
    );

    this.#publish(responseMessage);
  }

  #handleGetActiveActionEntriesRequest(
    sequenceNumber: number,
    _payload: capnp.Data,
  ) {
    const responseMessage = this.#buildMessage(
      sequenceNumber,
      keyboardMessageId.getActiveActionEntriesResponse,
      GetActiveActionEntriesResponse,
      (response) => {
        const actionEntries = Array.from(
          this.state.keyboardActionEntries.values(),
        );
        const entries = response._initEntries(actionEntries.length);
        actionEntries.forEach((actionEntry, index) => {
          // `List.set()` corrupts composite lists, so copy the fields explicitly.
          const entry = entries.get(index);
          entry.entryId = actionEntry.entryId;
          entry.action = actionEntry.action;
        });
        response.isFinalResponse = true;
      },
    );

    this.#publish(responseMessage);
  }

  #handleAddActionEntriesRequest(sequenceNumber: number, payload: capnp.Data) {
    const requestPayload = new Uint8Array(payload.toArrayBuffer());
    const requestMessage = new capnp.Message(requestPayload, false);
    const request = requestMessage.getRoot(AddActionEntriesRequest);
    const entries = request.entries;
    for (let i = 0; i < entries.length; i++) {
      const entry = entries.get(i);
      this.state.keyboardActionEntries.set(entry.entryId, entry);
    }

    const responseMessage = this.#buildMessage(
      sequenceNumber,
      coreMessageId.successResponse,
      SuccessResponse,
      (_response) => {},
    );

    this.#publish(responseMessage);
  }

  #handleUpdateLayoutRequest(sequenceNumber: number, payload: capnp.Data) {
    const requestPayload = new Uint8Array(payload.toArrayBuffer());
    const requestMessage = new capnp.Message(requestPayload, false);
    const request = requestMessage.getRoot(UpdateLayoutRequest);
    const actionEntryIds = request.actionEntryIds;
    for (let i = 0; i < actionEntryIds.length; i++) {
      this.state.keyboardLayout[request.startKeyIndex + i] =
        actionEntryIds.get(i);
    }

    const responseMessage = this.#buildMessage(
      sequenceNumber,
      coreMessageId.successResponse,
      SuccessResponse,
      (_response) => {},
    );

    this.#publish(responseMessage);
  }

  #handleGetLedArrayHookCapabilitiesRequest(
    sequenceNumber: number,
    _payload: capnp.Data,
  ) {
    const responseMessage = this.#buildMessage(
      sequenceNumber,
      ledArrayMessageId.getLedArrayHookCapabilitiesResponse,
      GetLedArrayHookCapabilitiesResponse,
      (response) => {
        response.hooks = BigInt(
          ledArrayHookId.boot |
            ledArrayHookId.layerChange |
            ledArrayHookId.blePairing |
            ledArrayHookId.bleProfile,
        );
      },
    );

    this.#publish(responseMessage);
  }

  #handleGetLedArrayHooksRequest(sequenceNumber: number, _payload: capnp.Data) {
    const responseMessage = this.#buildMessage(
      sequenceNumber,
      ledArrayMessageId.getLedArrayHooksResponse,
      GetLedArrayHooksResponse,
      (response) => {
        response.hooks = this.state.ledArrayHooks;
      },
    );

    this.#publish(responseMessage);
  }

  #handleSetLedArrayHooksRequest(sequenceNumber: number, payload: capnp.Data) {
    const requestPayload = new Uint8Array(payload.toArrayBuffer());
    const requestMessage = new capnp.Message(requestPayload, false);
    const request = requestMessage.getRoot(SetLedArrayHooksRequest);
    this.state.ledArrayHooks = request.hooks;

    const responseMessage = this.#buildMessage(
      sequenceNumber,
      coreMessageId.successResponse,
      SuccessResponse,
      (_response) => {},
    );

    this.#publish(responseMessage);
  }
}
