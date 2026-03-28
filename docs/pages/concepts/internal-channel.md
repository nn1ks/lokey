# Internal Channel

An **internal channel** wraps an [internal transport](./internal-transports.md) and provides a higher-level API for sending and receiving messages between device parts. Additionally, it exchanges messages on the same device part, for example between multiple components.

The internal channel is available from the [`Context`](https://docs.rs/lokey/latest/lokey/struct.Context.html) and can be used by components and other framework code.

See [`internal::Channel`](https://docs.rs/lokey/latest/lokey/internal/struct.Channel.html) for full API details.

## Sending

Sending messages to other device parts is done by calling the [`send`](https://docs.rs/lokey/latest/lokey/internal/struct.Channel.html#method.send) method on the internal channel:

```rust
context.internal_channel.send(message).await;
```

## Receiving

Receiving messages from other device parts, as well as from the same device part, is done by creating a receiver instance by calling the [`receiver`](https://docs.rs/lokey/latest/lokey/internal/struct.Channel.html#method.receiver) method on the internal channel, and then calling the `next` method on the receiver:

```rust
let mut receiver = context.internal_channel.receiver::<MyMessageType>()?;
let message = receiver.next().await;
```
