export type Listener<T> = (value: T) => void;

export class PubSub<T> {
  private readonly listeners = new Set<Listener<T>>();

  subscribe(listener: Listener<T>) {
    this.listeners.add(listener);

    return () => {
      this.listeners.delete(listener);
    };
  }

  next(): Promise<T> {
    return new Promise((resolve) => {
      const listener: Listener<T> = (value) => {
        this.listeners.delete(listener);
        resolve(value);
      };

      this.listeners.add(listener);
    });
  }

  publish(value: T) {
    this.listeners.forEach((listener) => listener(value));
  }

  clear() {
    this.listeners.clear();
  }
}

export default PubSub;
