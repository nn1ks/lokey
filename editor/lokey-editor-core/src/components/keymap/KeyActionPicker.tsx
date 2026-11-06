import { Accordion, Tabs, Tooltip } from "@ark-ui/solid";
import { For, createEffect, createMemo, createSignal } from "solid-js";
import { Portal } from "solid-js/web";
import type { Component } from "solid-js";
import { HoldTap } from "../../key-action/actions/hold-tap";
import { KeyCode } from "../../key-action/actions/key-code";
import { Layer } from "../../key-action/actions/layer";
import { None } from "../../key-action/actions/none";
import { Ordered } from "../../key-action/actions/ordered";
import { Sticky } from "../../key-action/actions/sticky";
import { Toggle } from "../../key-action/actions/toggle";
import { Transparent } from "../../key-action/actions/transparent";
import type { KeyAction } from "../../key-action/action";
import StyledTooltipContent from "../StyledTooltipContent";
import { ActionRenderer, ActionTooltipRenderer } from "./ActionRenderer";
import styles from "./KeyActionPicker.module.css";
import { ActionPickerRenderer } from "./ActionPickerRenderer";
import { keyCodeGroups, keyCodeCategoryOrder } from "./key-code-groups";
import type { KeyCodeCategory } from "./key-code-groups";
import { Unordered } from "../../key-action/actions/unordered";
import { useActionRegistry } from "../../contexts/ActionRegistryContext";
import { useKeyboardActions } from "../../contexts";
import { Layout } from "../../layouts";

export type KeyActionPickerProps = {
  onSelect: (action: KeyAction, close: boolean) => void;
  selectedAction?: KeyAction;
};

function isSelected(
  selected: KeyAction | undefined,
  action: KeyAction,
): boolean {
  if (!selected || selected.actionId !== action.actionId) {
    return false;
  }
  if (selected instanceof KeyCode && action instanceof KeyCode) {
    return selected.value === action.value;
  }
  return true;
}

function tabForAction(action: KeyAction | undefined, layout: Layout): string {
  const actualKeyCodeGroups = keyCodeGroups(layout);
  if (!action) {
    return "Letters";
  }
  if (action instanceof KeyCode) {
    for (const category of keyCodeCategoryOrder) {
      if (actualKeyCodeGroups[category].kind === "flat") {
        if (
          actualKeyCodeGroups[category].content.some(
            (keyCode) => keyCode.value === action.value,
          )
        ) {
          return category;
        }
      } else if (actualKeyCodeGroups[category].kind === "group") {
        for (const keyCodes of Object.values(
          actualKeyCodeGroups[category].content,
        )) {
          if (keyCodes.some((keyCode) => keyCode.value === action.value)) {
            return category;
          }
        }
      }
    }
    return "Letters";
  }
  return "Behaviour";
}

type FlatPickerActions = readonly KeyAction[];

type GroupPickerActions = Record<string, readonly KeyAction[]>;

type PickerActions =
  | { kind: "flat"; actions: FlatPickerActions }
  | { kind: "group"; actions: GroupPickerActions };

const tabs = [...keyCodeCategoryOrder, "Behaviour"];

function keyCodeActions(layout: Layout): Map<KeyCodeCategory, PickerActions> {
  const actualKeyCodeGroups = keyCodeGroups(layout);
  return new Map<KeyCodeCategory, PickerActions>(
    keyCodeCategoryOrder.map((category): [KeyCodeCategory, PickerActions] => {
      const content = actualKeyCodeGroups[category];
      if (content.kind === "flat") {
        return [category, { kind: "flat", actions: content.content }];
      }

      const actions: GroupPickerActions = {};
      for (const [group, keyCodes] of Object.entries(content.content)) {
        actions[group] = keyCodes;
      }
      return [category, { kind: "group", actions: actions }];
    }),
  );
}

const ActionGrid: Component<{
  actions: readonly KeyAction[];
  onSelect: (action: KeyAction) => void;
  selectedAction?: KeyAction;
}> = (props) => (
  <div class="grid grid-cols-[repeat(auto-fill,minmax(64px,1fr))] gap-2">
    <For each={props.actions}>
      {(action) => (
        <Tooltip.Root positioning={{ placement: "bottom", flip: false }}>
          <Tooltip.Trigger
            type="button"
            class={`
              relative flex h-16 w-full cursor-pointer items-center justify-center
              rounded-lg overflow
              text-neutral-900 bg-neutral-200 hover:bg-neutral-300
              dark:text-neutral-100 dark:bg-neutral-800 dark:hover:bg-neutral-700
              text-[16px] font-medium
              transition-colors duration-50
              [&_svg]:size-5
              ${
                isSelected(props.selectedAction, action)
                  ? "shadow-[0_0_0_2px_var(--color-accent)]"
                  : ""
              }
            `}
            onClick={() => props.onSelect(action)}
          >
            <ActionRenderer action={action} level={0} />
          </Tooltip.Trigger>
          <Portal>
            <Tooltip.Positioner>
              <StyledTooltipContent>
                <ActionTooltipRenderer action={action} />
              </StyledTooltipContent>
            </Tooltip.Positioner>
          </Portal>
        </Tooltip.Root>
      )}
    </For>
  </div>
);

const GroupedActionGrid: Component<{
  groups: GroupPickerActions;
  onSelect: (action: KeyAction) => void;
  selectedAction?: KeyAction;
}> = (props) => (
  <div class="flex flex-col gap-3">
    <For each={Object.entries(props.groups)}>
      {([label, actions]) => (
        <div class="flex flex-col gap-2 pt-1">
          <div class="text-xs font-semibold tracking-wider uppercase text-neutral-500 dark:text-neutral-400">
            {label}
          </div>
          <ActionGrid
            actions={actions}
            onSelect={props.onSelect}
            selectedAction={props.selectedAction}
          />
        </div>
      )}
    </For>
  </div>
);

const CategoryGrid: Component<{
  actions: PickerActions;
  onSelect: (action: KeyAction) => void;
  selectedAction?: KeyAction;
}> = (props) => {
  const actions = () => props.actions;
  return (
    <>
      {actions().kind === "flat" ? (
        <ActionGrid
          actions={
            (actions() as { kind: "flat"; actions: FlatPickerActions }).actions
          }
          onSelect={props.onSelect}
          selectedAction={props.selectedAction}
        />
      ) : (
        <GroupedActionGrid
          groups={
            (actions() as { kind: "group"; actions: GroupPickerActions })
              .actions
          }
          onSelect={props.onSelect}
          selectedAction={props.selectedAction}
        />
      )}
    </>
  );
};

const PickerActionList: Component<{
  actions: readonly KeyAction[];
  onSelect: (action: KeyAction, close: boolean) => void;
  selectedAction?: KeyAction;
}> = (props) => {
  const registry = useActionRegistry();

  const selectedAccordion = (): string[] => {
    const selected = props.selectedAction;
    if (!selected) {
      return [];
    }

    const definition = registry.byId(selected.actionId);
    if (!definition || definition.children(selected).length === 0) {
      return [];
    }

    const action = props.actions.find((action) => isSelected(selected, action));
    return action ? [definition.actionId.toString()] : [];
  };

  const [value, setValue] = createSignal<string[]>(selectedAccordion());

  createEffect(() => {
    const next = selectedAccordion();
    if (next.length === 0) {
      setValue([]);
      return;
    }

    setValue((current) =>
      next.every((label) => current.includes(label))
        ? current
        : [...new Set([...current, ...next])],
    );
  });

  return (
    <Accordion.Root
      class={`${styles.accordion} flex flex-col gap-2`}
      collapsible
      multiple
      value={value()}
      onValueChange={(details) => setValue(details.value)}
    >
      <For each={props.actions}>
        {(action) => {
          const selected = () => isSelected(props.selectedAction, action);
          const selectedOrDefaultAction = () =>
            selected() ? props.selectedAction! : action;
          return (
            <Accordion.Item
              class={`rounded-lg bg-neutral-200 dark:bg-neutral-800 ${
                selected() ? "shadow-[0_0_0_2px_var(--color-accent)]" : ""
              }`}
              value={action.actionId.toString()}
            >
              <ActionPickerRenderer
                action={selectedOrDefaultAction()}
                onSelect={props.onSelect}
              />
            </Accordion.Item>
          );
        }}
      </For>
    </Accordion.Root>
  );
};

export const KeyActionPicker: Component<KeyActionPickerProps> = (props) => {
  const { layout } = useKeyboardActions();

  const [activeTab, setActiveTab] = createSignal(
    tabForAction(props.selectedAction, layout()),
  );

  createEffect(() => {
    if (props.selectedAction) {
      setActiveTab(tabForAction(props.selectedAction, layout()));
    }
  });

  const listedBehaviourActions = createMemo<readonly KeyAction[]>(() => [
    new Layer(0),
    new HoldTap(new None(), new None()),
    new Ordered(new None(), new None()),
    new Unordered(new None(), new None()),
    new Sticky(new None()),
    new Toggle(new None()),
  ]);

  const simpleBehaviourActions = createMemo<readonly KeyAction[]>(() => [
    new None(),
    new Transparent(),
  ]);

  return (
    <Tabs.Root
      value={activeTab()}
      onValueChange={(details) => setActiveTab(details.value)}
      lazyMount
      unmountOnExit
      class="flex flex-col gap-3"
    >
      <Tabs.List class="flex flex-wrap gap-1">
        <For each={tabs}>
          {(tab) => (
            <Tabs.Trigger
              value={tab}
              class="
                cursor-pointer rounded-lg px-3 py-1.5 text-sm font-medium
                text-neutral-600 transition-colors duration-50
                hover:bg-neutral-200 hover:text-neutral-900
                data-selected:bg-neutral-200 data-selected:text-neutral-900
                dark:text-neutral-400 dark:hover:bg-neutral-800
                dark:hover:text-neutral-100
                dark:data-selected:bg-neutral-800
                dark:data-selected:text-neutral-100
              "
            >
              {tab}
            </Tabs.Trigger>
          )}
        </For>
      </Tabs.List>
      <div class="h-px w-full bg-neutral-200 dark:bg-neutral-800"></div>
      <Tabs.Content value="Behaviour" class="data-[state=closed]:hidden">
        <div class="flex flex-col gap-2">
          <ActionGrid
            actions={simpleBehaviourActions()}
            onSelect={(action) => props.onSelect(action, true)}
            selectedAction={props.selectedAction}
          />
          <PickerActionList
            actions={listedBehaviourActions()}
            onSelect={props.onSelect}
            selectedAction={props.selectedAction}
          />
        </div>
      </Tabs.Content>
      <For each={keyCodeCategoryOrder}>
        {(category) => (
          <Tabs.Content value={category} class="data-[state=closed]:hidden">
            <CategoryGrid
              actions={keyCodeActions(layout()).get(category)!}
              onSelect={(action) => props.onSelect(action, true)}
              selectedAction={props.selectedAction}
            />
          </Tabs.Content>
        )}
      </For>
    </Tabs.Root>
  );
};

export default KeyActionPicker;
