import { createMemo } from "solid-js";
import type { Component } from "solid-js";
import { useActionRegistry } from "../../contexts/ActionRegistryContext";
import type { KeyAction, PickerContext } from "../../key-action";
import { ActionRenderer } from "./ActionRenderer";

export type ActionPickerRendererProps = {
  action: KeyAction;
  onSelect: (action: KeyAction, close: boolean) => void;
};

export const ActionPickerRenderer: Component<ActionPickerRendererProps> = (
  props,
) => {
  const registry = useActionRegistry();

  const ctx = (): PickerContext => ({
    onSelect: (action, close) => props.onSelect(action, close),
    renderAction: (action) => <ActionRenderer action={action} level={0} />,
  });

  const definition = () => registry.byId(props.action.actionId);

  const pickerElement = createMemo(() => {
    const def = definition();
    if (def?.renderPicker) {
      return def.renderPicker(props.action, ctx());
    }
    return <ActionRenderer action={props.action} level={0} />;
  });

  return <>{pickerElement()}</>;
};

export default ActionPickerRenderer;
