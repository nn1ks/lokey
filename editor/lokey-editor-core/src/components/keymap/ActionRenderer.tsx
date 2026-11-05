import type { Component, JSX } from "solid-js";
import { useActionRegistry } from "../../contexts/ActionRegistryContext";
import type { KeyAction, RenderContext } from "../../key-action";
import type { ActionRegistry } from "../../key-action/registry";

function renderAction(
  registry: ActionRegistry,
  action: KeyAction,
  level: number,
  ctx: RenderContext,
): JSX.Element {
  const definition = registry.byId(action.actionId);
  if (!definition) {
    return <>Unsupported action ({action.actionId})</>;
  }

  return definition.render(action, level, ctx);
}

function renderTooltip(
  registry: ActionRegistry,
  action: KeyAction,
  ctx: RenderContext,
): JSX.Element {
  const definition = registry.byId(action.actionId);
  if (!definition) {
    return <>Unsupported action ({action.actionId})</>;
  }

  return definition.renderTooltip(action, ctx);
}

function createRenderContext(registry: ActionRegistry): RenderContext {
  const ctx: RenderContext = {
    renderAction: (action, level) => renderAction(registry, action, level, ctx),
    renderTooltip: (action) => renderTooltip(registry, action, ctx),
  };
  return ctx;
}

export const ActionRenderer: Component<{
  action: KeyAction;
  level?: number;
}> = (props) => {
  const registry = useActionRegistry();
  const ctx = createRenderContext(registry);
  return <>{renderAction(registry, props.action, props.level ?? 0, ctx)}</>;
};

export const ActionTooltipRenderer: Component<{ action: KeyAction }> = (
  props,
) => {
  const registry = useActionRegistry();
  const ctx = createRenderContext(registry);
  return <>{renderTooltip(registry, props.action, ctx)}</>;
};
