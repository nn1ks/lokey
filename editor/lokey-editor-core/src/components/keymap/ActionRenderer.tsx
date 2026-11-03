import type { Component, JSX } from "solid-js";
import { useActionRegistry } from "../../contexts/ActionRegistryContext";
import type { KeyAction } from "../../key-action";
import type { ActionRegistry } from "../../key-action/registry";

function renderAction(
  registry: ActionRegistry,
  action: KeyAction,
  level: number,
): JSX.Element {
  const definition = registry.byId(action.actionId);
  if (!definition) {
    return <>Unsupported action ({action.actionId})</>;
  }

  return definition.render(action, level, {
    renderAction: (child, childLevel) =>
      renderAction(registry, child, childLevel),
    renderTooltip: (child) => renderTooltip(registry, child),
  });
}

function renderTooltip(
  registry: ActionRegistry,
  action: KeyAction,
): JSX.Element {
  const definition = registry.byId(action.actionId);
  if (!definition) {
    return <>Unsupported action ({action.actionId})</>;
  }

  return definition.renderTooltip(action, {
    renderAction: (child, childLevel) =>
      renderAction(registry, child, childLevel),
    renderTooltip: (child) => renderTooltip(registry, child),
  });
}

export const ActionRenderer: Component<{
  action: KeyAction;
  level?: number;
}> = (props) => {
  const registry = useActionRegistry();
  return <>{renderAction(registry, props.action, props.level ?? 0)}</>;
};

export const ActionTooltipRenderer: Component<{ action: KeyAction }> = (
  props,
) => {
  const registry = useActionRegistry();
  return <>{renderTooltip(registry, props.action)}</>;
};
