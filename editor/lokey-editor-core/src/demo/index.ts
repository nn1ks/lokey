import * as capnp from 'capnp-es'
import { GetDeviceCapabilitiesResponse } from 'lokey-editor-odyn/core'
import {
  getDeviceCapabilitiesRequest,
  getDeviceCapabilitiesResponse,
} from 'lokey-editor-odyn/core/message-id'
import { PubSub } from '../protocol'
import { Message } from 'lokey-editor-odyn/core'

export class DemoMessageHandler {
  readonly pubsub = new PubSub<Message>()

  async send(message: Message): Promise<void> {
    switch (message.id) {
      case getDeviceCapabilitiesRequest: {
        const responsePayload = new capnp.Message()
        const response = responsePayload.initRoot(GetDeviceCapabilitiesResponse)
        response.deviceId = 0x1000_0001
        response.maxPayloadSize = 32

        const responseEnvelopeMessage = new capnp.Message()
        const responseEnvelope = responseEnvelopeMessage.initRoot(Message)
        responseEnvelope.sequenceNumber = message.sequenceNumber
        responseEnvelope.id = getDeviceCapabilitiesResponse
        const serializedResponse = new Uint8Array(responsePayload.toArrayBuffer())
        responseEnvelope._initPayload(serializedResponse.byteLength).copyBuffer(serializedResponse)

        this.pubsub.publish(responseEnvelope)
        break
      }
      default:
        break
    }
  }

  async receive(): Promise<Message> {
    return await this.pubsub.next()
  }
}
