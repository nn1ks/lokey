import type { KeyAction, KeyActionClass } from "./action";

export class ActionRegistry {
  readonly #byId = new Map<number, KeyActionClass>();

  register<A extends KeyAction>(cls: KeyActionClass<A>): void {
    if (this.#byId.has(cls.actionId)) {
      throw new Error(
        `An action with ID ${cls.actionId} is already registered.`,
      );
    }
    this.#byId.set(cls.actionId, cls as KeyActionClass);
  }

  byId(actionId: number): KeyActionClass | undefined {
    return this.#byId.get(actionId);
  }

  all(): KeyActionClass[] {
    return [...this.#byId.values()];
  }
}

export function createActionRegistry(): ActionRegistry {
  return new ActionRegistry();
}
