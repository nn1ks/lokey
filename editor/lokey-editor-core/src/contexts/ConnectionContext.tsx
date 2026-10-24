import { createContext, createSignal, useContext } from "solid-js";
import type { Accessor, ParentComponent } from "solid-js";

export type Writer = {
  write: (data: Uint8Array) => Promise<void>;
};

export type Reader = {
  read: () => Promise<Uint8Array | undefined>;
};

export type SerialPortLike = {
  open(options: { baudRate: number }): Promise<void>;
  close(): Promise<void>;
  addEventListener(type: "disconnect", listener: () => void): void;
  removeEventListener(type: "disconnect", listener: () => void): void;
  readable?: ReadableStream<Uint8Array>;
  writable?: WritableStream<Uint8Array>;
};

export type ConnectionState =
  | { status: "unsupported" }
  | { status: "disconnected" }
  | {
      status: "connected";
      port: SerialPortLike;
      writer: WritableStreamDefaultWriter<Uint8Array>;
      reader: ReadableStreamDefaultReader<Uint8Array>;
      isDemo: boolean;
    };

export type ConnectionContextValue = {
  connection: Accessor<ConnectionState>;
  connect: (port: SerialPortLike) => Promise<void>;
  connectDemo: () => Promise<void>;
  disconnect: () => Promise<void>;
};

export const ConnectionContext = createContext<ConnectionContextValue>();

const supportsWebSerial = () =>
  typeof navigator !== "undefined" && "serial" in navigator;

export const ConnectionProvider: ParentComponent = (props) => {
  const [connection, setConnection] = createSignal<ConnectionState>(
    supportsWebSerial()
      ? { status: "disconnected" }
      : { status: "unsupported" },
  );

  const connect = async (port: SerialPortLike) => {
    await port.open({ baudRate: 115200 });

    if (!port.readable) {
      throw new Error("The Web Serial port is not readable.");
    }
    if (!port.writable) {
      throw new Error("The Web Serial port is not writable.");
    }

    const writer: WritableStreamDefaultWriter<Uint8Array> =
      port.writable.getWriter();
    const reader: ReadableStreamDefaultReader<Uint8Array> =
      port.readable.getReader();

    const handleDisconnect = () => {
      port.removeEventListener("disconnect", handleDisconnect);
      writer.releaseLock();
      // void reader.cancel()
      reader.releaseLock();
      setConnection({ status: "disconnected" });
    };

    port.addEventListener("disconnect", handleDisconnect);
    setConnection({ status: "connected", port, writer, reader, isDemo: false });
  };

  const connectDemo = async () => {
    const writer = {} as WritableStreamDefaultWriter<Uint8Array>;
    const reader = {} as ReadableStreamDefaultReader<Uint8Array>;

    setConnection({
      status: "connected",
      port: {} as SerialPortLike,
      writer,
      reader,
      isDemo: true,
    });
  };

  const disconnect = async () => {
    const current = connection();

    if (current.status === "connected") {
      try {
        await current.reader.cancel();
        current.writer.releaseLock();
        await current.port.close();
        current.reader.releaseLock();
      } catch (cause) {
        // eslint-disable-next-line no-console
        console.error("Unable to close the Web Serial port.", cause);
      }
    }

    setConnection({ status: "disconnected" });
  };

  const value: ConnectionContextValue = {
    connection,
    connect,
    connectDemo,
    disconnect,
  };

  return (
    <ConnectionContext.Provider value={value}>
      {props.children}
    </ConnectionContext.Provider>
  );
};

export const useConnection = () => {
  const context = useContext(ConnectionContext);

  if (!context) {
    throw new Error("useConnection must be used inside a ConnectionProvider");
  }

  return context;
};
