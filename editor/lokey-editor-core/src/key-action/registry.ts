import type { KeyAction } from "./action";
import type { ActionDefinition } from "./definition";

/**
 * Type-erased definition used internally by the registry. The registry only
 * passes actions whose `actionId` matches the definition, so the cast in
 * `register` is safe.
 */
export type RegisteredActionDefinition = ActionDefinition<KeyAction>;

export class ActionRegistry {
  readonly #byId = new Map<number, RegisteredActionDefinition>();

  register<A extends KeyAction>(definition: ActionDefinition<A>): void {
    if (this.#byId.has(definition.actionId)) {
      throw new Error(
        `An action with ID ${definition.actionId} is already registered.`,
      );
    }
    this.#byId.set(
      definition.actionId,
      definition as unknown as RegisteredActionDefinition,
    );
  }

  byId(actionId: number): RegisteredActionDefinition | undefined {
    return this.#byId.get(actionId);
  }

  all(): RegisteredActionDefinition[] {
    return [...this.#byId.values()];
  }

  available(actionIds: ReadonlySet<number>): RegisteredActionDefinition[] {
    return this.all().filter((definition) =>
      actionIds.has(definition.actionId),
    );
  }
}

export function createActionRegistry(): ActionRegistry {
  return new ActionRegistry();
}
