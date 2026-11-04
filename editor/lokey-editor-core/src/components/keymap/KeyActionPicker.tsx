import { Tabs, Tooltip } from "@ark-ui/solid";
import { For, createEffect, createSignal } from "solid-js";
import { Portal } from "solid-js/web";
import type { Component } from "solid-js";
import { HoldTap } from "../../key-action/actions/hold-tap";
import { KeyCode } from "../../key-action/actions/key-code";
import { Layer } from "../../key-action/actions/layer";
import { NoAction } from "../../key-action/actions/none";
import { Sequence } from "../../key-action/actions/sequence";
import { Transparent } from "../../key-action/actions/transparent";
import type { KeyAction } from "../../key-action/action";
import StyledTooltipContent from "../StyledTooltipContent";
import { ActionRenderer, ActionTooltipRenderer } from "./ActionRenderer";
import { ActionPickerRenderer } from "./ActionPickerRenderer";
import {
  keyCodeCategoryLabels,
  keyCodeCategoryOrder,
  keyCodeGroups,
} from "./key-code-groups";

export type KeyActionPickerProps = {
  onSelect: (action: KeyAction) => void;
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

function tabForAction(action: KeyAction | undefined): string {
  if (!action) {
    return "letters";
  }
  if (action instanceof KeyCode) {
    for (const category of keyCodeCategoryOrder) {
      if (
        keyCodeGroups[category].some(
          (keyCode) => keyCode.value === action.value,
        )
      ) {
        return category;
      }
    }
    return "letters";
  }
  return "behaviour";
}

type PickerEntry = {
  action: KeyAction;
  label: string;
};

const tabs = [
  ...keyCodeCategoryOrder.map((category) => ({
    value: category,
    label: keyCodeCategoryLabels[category],
  })),
  { value: "behaviour", label: "Behaviour" },
];

const ActionGrid: Component<{
  entries: readonly PickerEntry[];
  onSelect: (action: KeyAction) => void;
  selectedAction?: KeyAction;
}> = (props) => (
  <div class="grid grid-cols-[repeat(auto-fill,minmax(56px,1fr))] sm:grid-cols-[repeat(auto-fill,minmax(64px,1fr))] gap-2">
    <For each={props.entries}>
      {(entry) => (
        <Tooltip.Root positioning={{ placement: "bottom", flip: false }}>
          <Tooltip.Trigger
            type="button"
            aria-label={entry.label}
            class={`
              relative flex h-14 sm:h-16 w-full cursor-pointer items-center justify-center
              rounded-lg overflow
              text-neutral-900 bg-neutral-200 hover:bg-neutral-300
              dark:text-neutral-100 dark:bg-neutral-800 dark:hover:bg-neutral-700
              text-[16px] font-medium
              transition-colors duration-50
              [&_svg]:size-5
              ${
                isSelected(props.selectedAction, entry.action)
                  ? "shadow-[0_0_0_2px_var(--color-accent)]"
                  : ""
              }
            `}
            onClick={() => props.onSelect(entry.action)}
          >
            <ActionRenderer action={entry.action} level={0} />
          </Tooltip.Trigger>
          <Portal>
            <Tooltip.Positioner>
              <StyledTooltipContent>
                <ActionTooltipRenderer action={entry.action} />
              </StyledTooltipContent>
            </Tooltip.Positioner>
          </Portal>
        </Tooltip.Root>
      )}
    </For>
  </div>
);

const PickerActionList: Component<{
  entries: readonly PickerEntry[];
  onSelect: (action: KeyAction) => void;
  selectedAction?: KeyAction;
}> = (props) => (
  <div class="flex flex-col gap-4">
    <For each={props.entries}>
      {(entry) => {
        const selected = () => isSelected(props.selectedAction, entry.action);
        const action = () =>
          selected() ? props.selectedAction! : entry.action;
        return (
          <div
            class={`rounded-lg p-2 bg-neutral-200 dark:bg-neutral-800 ${
              selected() ? "shadow-[0_0_0_2px_var(--color-accent)]" : ""
            }`}
          >
            <ActionPickerRenderer action={action()} onSelect={props.onSelect} />
          </div>
        );
      }}
    </For>
  </div>
);

export const KeyActionPicker: Component<KeyActionPickerProps> = (props) => {
  const [activeTab, setActiveTab] = createSignal(
    tabForAction(props.selectedAction),
  );

  createEffect(() => {
    if (props.selectedAction) {
      setActiveTab(tabForAction(props.selectedAction));
    }
  });

  const pickerActionEntries: readonly PickerEntry[] = [
    { action: new Layer(0), label: "Layer" },
    { action: new HoldTap(new NoAction(), new NoAction()), label: "Hold/Tap" },
    { action: new Sequence(new NoAction(), new NoAction()), label: "Sequence" },
  ];

  const leafActionEntries: readonly PickerEntry[] = [
    { action: new NoAction(), label: "None" },
    { action: new Transparent(), label: "Transparent" },
  ];

  return (
    <Tabs.Root
      value={activeTab()}
      onValueChange={(details) => setActiveTab(details.value)}
      class="flex flex-col gap-3"
    >
      <Tabs.List class="flex flex-wrap gap-1">
        <For each={tabs}>
          {(tab) => (
            <Tabs.Trigger
              value={tab.value}
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
              {tab.label}
            </Tabs.Trigger>
          )}
        </For>
      </Tabs.List>
      <div class="h-px w-full bg-neutral-200 dark:bg-neutral-800"></div>
      <Tabs.Content value="behaviour" class="data-[state=closed]:hidden">
        <div class="flex flex-col gap-4">
          <ActionGrid
            entries={leafActionEntries}
            onSelect={props.onSelect}
            selectedAction={props.selectedAction}
          />
          <PickerActionList
            entries={pickerActionEntries}
            onSelect={props.onSelect}
            selectedAction={props.selectedAction}
          />
        </div>
      </Tabs.Content>
      <For each={keyCodeCategoryOrder}>
        {(category) => (
          <Tabs.Content value={category} class="data-[state=closed]:hidden">
            <ActionGrid
              entries={keyCodeGroups[category].map((keyCode) => ({
                action: keyCode,
                label: keyCode.displayValue(),
              }))}
              onSelect={props.onSelect}
              selectedAction={props.selectedAction}
            />
          </Tabs.Content>
        )}
      </For>
    </Tabs.Root>
  );
};

export default KeyActionPicker;
