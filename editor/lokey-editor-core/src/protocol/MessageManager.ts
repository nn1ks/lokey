import * as capnp from "capnp-es";
import {
  GetDeviceCapabilitiesRequest,
  GetDeviceCapabilitiesResponse,
  Message,
  SuccessResponse,
} from "lokey-editor-odyn/core";
import * as coreMessageId from "lokey-editor-odyn/core/message-id";
import * as ledArrayMessageId from "lokey-editor-odyn/led-array/message-id";
import * as keyboardMessageId from "lokey-editor-odyn/keyboard/message-id";
import PubSub from "./PubSub";
import { DemoMessageHandler } from "../demo";
import {
  GetLedArrayHookCapabilitiesRequest,
  GetLedArrayHookCapabilitiesResponse,
  GetLedArrayHooksRequest,
  GetLedArrayHooksResponse,
  SetLedArrayHooksRequest,
} from "lokey-editor-odyn/led-array";
import {
  AddActionEntriesRequest,
  GetActiveActionEntriesRequest,
  GetActiveActionEntriesResponse,
  GetHostLayoutRequest,
  GetHostLayoutResponse,
  GetLayoutRequest,
  GetLayoutResponse,
  GetNumberOfKeysRequest,
  GetNumberOfKeysResponse,
  HostLayout,
  KeyboardActionEntry,
  SetHostLayoutRequest,
  UpdateLayoutRequest,
} from "lokey-editor-odyn/keyboard";

export type MessageManagerTransport = {
  write: (data: Uint8Array) => Promise<void>;
  read: () => Promise<Uint8Array | undefined>;
};

const appendBytes = (current: Uint8Array, next: Uint8Array) => {
  const result = new Uint8Array(current.byteLength + next.byteLength);
  result.set(current);
  result.set(next, current.byteLength);
  return result;
};

export class MessageManager {
  readonly messages = new PubSub<Message>();
  private readError: unknown = undefined;

  constructor(private readonly transport: MessageManagerTransport) {
    void this.readMessages();
  }

  private async readMessages() {
    let buffer = new Uint8Array();

    try {
      // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
      while (true) {
        const chunk = await this.transport.read();
        if (!chunk) {
          throw new Error("The Web Serial port closed while reading messages.");
        }
        buffer = appendBytes(buffer, chunk);

        let receivedMessage: Message;
        try {
          const capnpMessage = new capnp.Message(buffer, false);
          receivedMessage = capnpMessage.getRoot(Message);
        } catch (e) {
          // A serial read may contain only part of a Cap'n Proto message.
          continue;
        }

        buffer = new Uint8Array();
        this.messages.publish(receivedMessage);
      }
    } catch (cause) {
      this.readError = cause;
    }
  }

  static fromWebSerial(
    writer: WritableStreamDefaultWriter<Uint8Array>,
    reader: ReadableStreamDefaultReader<Uint8Array>,
  ): MessageManager {
    return new MessageManager({
      write: async (data) => {
        await writer.write(data);
      },
      read: async () => {
        const result = await reader.read();
        return result.value;
      },
    });
  }

  static fromDemo(): MessageManager {
    var demoHandler = new DemoMessageHandler();
    return new MessageManager({
      write: async (data) => {
        const message = new capnp.Message(data, false);
        const envelope = message.getRoot(Message);
        await demoHandler.send(envelope);
      },
      read: async () => {
        const message = await demoHandler.receive();
        const capnpMessage = new capnp.Message(undefined, false);
        const envelope = capnpMessage.initRoot(Message);
        envelope.sequenceNumber = message.sequenceNumber;
        envelope.id = message.id;
        envelope.payload = message.payload;
        return new Uint8Array(capnpMessage.toArrayBuffer());
      },
    });
  }

  async requestSingle<
    TRequest extends capnp.Struct,
    TResponse extends capnp.Struct,
  >(
    requestId: number,
    requestType: capnp.StructCtor<TRequest>,
    setRequest: (message: TRequest) => void,
    responseId: number,
    responseType: capnp.StructCtor<TResponse>,
  ): Promise<TResponse> {
    if (this.readError) {
      throw this.readError;
    }

    const capnpRequestMessage = new capnp.Message();
    const requestMessage = capnpRequestMessage.initRoot(requestType);
    setRequest(requestMessage);
    const payload = new Uint8Array(capnpRequestMessage.toArrayBuffer());

    const message = new capnp.Message();
    const envelope = message.initRoot(Message);
    envelope.sequenceNumber = 0;
    envelope.id = requestId;
    envelope._initPayload(payload.byteLength).copyBuffer(payload);

    return new Promise<TResponse>((resolve, reject) => {
      const unsubscribe = this.messages.subscribe((message) => {
        if (message.id !== responseId) {
          return;
        }

        try {
          const responsePayload = new Uint8Array(
            message.payload.toArrayBuffer(),
          );
          const responseMessage = new capnp.Message(responsePayload, false);
          const response = responseMessage.getRoot(responseType);
          unsubscribe();
          resolve(response);
        } catch (cause) {
          unsubscribe();
          reject(cause);
        }
      });

      void this.transport
        .write(new Uint8Array(message.toArrayBuffer()))
        .catch((cause) => {
          unsubscribe();
          reject(cause);
        });
    });
  }

  async getDeviceCapabilities(): Promise<GetDeviceCapabilitiesResponse> {
    return await this.requestSingle(
      coreMessageId.getDeviceCapabilitiesRequest,
      GetDeviceCapabilitiesRequest,
      (_request) => {},
      coreMessageId.getDeviceCapabilitiesResponse,
      GetDeviceCapabilitiesResponse,
    );
  }

  async getLedArrayHookCapabilities(): Promise<GetLedArrayHookCapabilitiesResponse> {
    return await this.requestSingle(
      ledArrayMessageId.getLedArrayHookCapabilitiesRequest,
      GetLedArrayHookCapabilitiesRequest,
      (_request) => {},
      ledArrayMessageId.getLedArrayHookCapabilitiesResponse,
      GetLedArrayHookCapabilitiesResponse,
    );
  }

  async getLedArrayHooks(): Promise<GetLedArrayHooksResponse> {
    return await this.requestSingle(
      ledArrayMessageId.getLedArrayHooksRequest,
      GetLedArrayHooksRequest,
      (_request) => {},
      ledArrayMessageId.getLedArrayHooksResponse,
      GetLedArrayHooksResponse,
    );
  }

  async getNumberOfKeys(): Promise<GetNumberOfKeysResponse> {
    return await this.requestSingle(
      keyboardMessageId.getNumberOfKeysRequest,
      GetNumberOfKeysRequest,
      (_request) => {},
      keyboardMessageId.getNumberOfKeysResponse,
      GetNumberOfKeysResponse,
    );
  }

  async getHostLayout(): Promise<GetHostLayoutResponse> {
    return await this.requestSingle(
      keyboardMessageId.getHostLayoutRequest,
      GetHostLayoutRequest,
      (_request) => {},
      keyboardMessageId.getHostLayoutResponse,
      GetHostLayoutResponse,
    );
  }

  async setHostLayout(hostLayout: HostLayout): Promise<void> {
    await this.requestSingle(
      keyboardMessageId.setHostLayoutRequest,
      SetHostLayoutRequest,
      (request) => {
        request.hostLayout = hostLayout;
      },
      coreMessageId.successResponse,
      SuccessResponse,
    );
  }

  async getLayout(): Promise<GetLayoutResponse> {
    return await this.requestSingle(
      keyboardMessageId.getLayoutRequest,
      GetLayoutRequest,
      (_request) => {},
      keyboardMessageId.getLayoutResponse,
      GetLayoutResponse,
    );
  }

  async getActiveActionEntries(): Promise<GetActiveActionEntriesResponse> {
    return await this.requestSingle(
      keyboardMessageId.getActiveActionEntriesRequest,
      GetActiveActionEntriesRequest,
      (_request) => {},
      keyboardMessageId.getActiveActionEntriesResponse,
      GetActiveActionEntriesResponse,
    );
  }

  async addActionEntries(entries: KeyboardActionEntry[]): Promise<void> {
    await this.requestSingle(
      keyboardMessageId.addActionEntriesRequest,
      AddActionEntriesRequest,
      (request) => {
        const list = request._initEntries(entries.length);
        entries.forEach((entry, index) => {
          const target = list.get(index);
          target.entryId = entry.entryId;
          target.action = entry.action;
        });
      },
      coreMessageId.successResponse,
      SuccessResponse,
    );
  }

  async updateLayout(
    startKeyIndex: number,
    actionEntryIds: number[],
  ): Promise<void> {
    await this.requestSingle(
      keyboardMessageId.updateLayoutRequest,
      UpdateLayoutRequest,
      (request) => {
        request.startKeyIndex = startKeyIndex;
        const list = request._initActionEntryIds(actionEntryIds.length);
        actionEntryIds.forEach((entryId, index) => list.set(index, entryId));
      },
      coreMessageId.successResponse,
      SuccessResponse,
    );
  }

  async setLedArrayHooks(hooks: bigint): Promise<void> {
    await this.requestSingle(
      ledArrayMessageId.setLedArrayHooksRequest,
      SetLedArrayHooksRequest,
      (request) => {
        request.hooks = hooks;
      },
      coreMessageId.successResponse,
      SuccessResponse,
    );
  }
}

export default MessageManager;
