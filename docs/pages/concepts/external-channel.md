# External Channel

An **external channel** wraps an [external transport](./external-transports.md) and provides a higher-level API for sending, receiving, and observing host-facing messages.

The external channel is available from the [`Context`](https://docs.rs/lokey/latest/lokey/struct.Context.html) and can be used by components and other framework code.

See [`external::Channel`](https://docs.rs/lokey/latest/lokey/external/struct.Channel.html) for full API details.

## Sending

Sending messages to the host is done by calling the [`send`](https://docs.rs/lokey/latest/lokey/external/struct.Channel.html#method.send) or [`try_send`](https://docs.rs/lokey/latest/lokey/external/struct.Channel.html#method.try_send)[^try-methods] method on the external channel:

```rust
context.external_channel.send(message).await;
```

## Receiving

Receiving messages from the host is done by creating a receiver instance by calling the [`receiver`](https://docs.rs/lokey/latest/lokey/external/struct.Channel.html#method.receiver) or [`try_receiver`](https://docs.rs/lokey/latest/lokey/external/struct.Channel.html#method.try_receiver)[^try-methods] method on the external channel, and then calling the `next` method on the receiver:

```rust
let mut receiver = context.external_channel.receiver::<MyMessageType>()?;
let message = receiver.next().await;
```

## Observing

Observing messages that will be sent to the host is done by creating an observer instance by calling the [`observer`](https://docs.rs/lokey/latest/lokey/external/struct.Channel.html#method.observer) or [`try_observer`](https://docs.rs/lokey/latest/lokey/external/struct.Channel.html#method.try_observer)[^try-methods] method on the external channel, and then calling the `next` method on the observer:

```rust
let mut observer = context.external_channel.observer::<MyMessageType>()?;
let message = observer.next().await;
```

<br>

[^try-methods]: The difference between the `try_*` methods and regular methods is that the `try_*` methods take any external message type, while the regular methods require that the message type is the one specified by the transport via the `TxMessage`/`RxMessage` associated types, or a type that can be converted to it via the [`TryFromMessage`](https://docs.rs/lokey/latest/lokey/external/trait.TryFromMessage.html) trait. The `try_*` methods can be used if you don't know the the message type of the external transport. If you do know the message type, the regular methods should be used, as they will check the message type at compile time and are a bit more performant.
