import * as capnp from "capnp-es";
import {
  GetDeviceCapabilitiesResponse,
  SuccessResponse,
} from "lokey-editor-odyn/core";
import { GetNumberOfKeysResponse } from "lokey-editor-odyn/keyboard";
import {
  GetLedArrayHookCapabilitiesResponse,
  GetLedArrayHooksResponse,
  SetLedArrayHooksRequest,
} from "lokey-editor-odyn/led-array";
import * as ledArrayHookId from "lokey-editor-odyn/led-array/hook-id";
import * as ledArrayMessageId from "lokey-editor-odyn/led-array/message-id";
import * as keyboardMessageId from "lokey-editor-odyn/keyboard/message-id";
import * as coreMessageId from "lokey-editor-odyn/core/message-id";
import { PubSub } from "../protocol";
import { Message } from "lokey-editor-odyn/core";

type State = {
  ledArrayHooks: bigint;
};

function defaultState(): State {
  return {
    ledArrayHooks: BigInt(
      ledArrayHookId.boot |
        ledArrayHookId.layerChange |
        ledArrayHookId.blePairing |
        ledArrayHookId.bleProfile,
    ),
  };
}

export class DemoMessageHandler {
  readonly pubsub = new PubSub<Message>();
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
    return await this.pubsub.next();
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

    this.pubsub.publish(responseMessage);
  }

  #handleGetNumberOfKeysRequest(sequenceNumber: number, _payload: capnp.Data) {
    const responseMessage = this.#buildMessage(
      sequenceNumber,
      keyboardMessageId.getNumberOfKeysResponse,
      GetNumberOfKeysResponse,
      (response) => {
        response.numKeys = 36;
      },
    );

    this.pubsub.publish(responseMessage);
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

    this.pubsub.publish(responseMessage);
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

    this.pubsub.publish(responseMessage);
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

    this.pubsub.publish(responseMessage);
  }
}
