let counter = 0n;

export class LayerId {
  private readonly id: bigint;

  constructor() {
    this.id = counter;
    counter += 1n;
  }

  equals(other: LayerId): boolean {
    return this.id === other.id;
  }
}
