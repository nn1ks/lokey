import { Select, createListCollection } from "@ark-ui/solid/select";
import { For, Show, splitProps } from "solid-js";
import type { Component, JSX } from "solid-js";
import { ChevronsUpDownIcon } from "lucide-solid";
import type { ButtonProps } from "../Button";
import { useKeyboardActions, useToast } from "../../contexts";
import { isLayout, layouts } from "../../layouts";

const hostLayoutCollection = createListCollection({ items: [...layouts] });

const OptionsButtonList: Component<{ children: JSX.Element }> = (props) => (
  <ul
    class="
      m-0 list-inside list-disc flex flex-col gap-0.75
      text-neutral-600 dark:text-neutral-400
    "
  >
    {props.children}
  </ul>
);

const OptionsButton: Component<ButtonProps> = (props) => {
  const [local, rest] = splitProps(props, ["children", "class"]);

  return (
    <button
      {...rest}
      type="button"
      class={[
        "w-fit cursor-pointer bg-transparent",
        "underline decoration-dashed",
        "hover:text-neutral-900 dark:hover:text-neutral-100",
        local.class ?? "",
      ]
        .filter(Boolean)
        .join(" ")}
    >
      {local.children}
    </button>
  );
};

const OptionsSection: Component<{ title: string; children: JSX.Element }> = (
  props,
) => {
  return (
    <div class="flex flex-1 justify-start sm:justify-center">
      <div class="flex flex-col gap-2">
        <span class="font-medium italic text-neutral-600 dark:text-neutral-400">
          {props.title}
        </span>
        {props.children}
      </div>
    </div>
  );
};

export type OptionsProps = {
  onPrint?: () => void;
};

export const Options: Component<OptionsProps> = (props) => {
  const keyboardActions = useKeyboardActions();
  const toast = useToast();

  return (
    <div class="mt-6 flex flex-col gap-6 sm:flex-row">
      <OptionsSection title="Host Layout">
        <Select.Root
          class="relative w-40 min-w-40 max-w-40"
          collection={hostLayoutCollection}
          value={[keyboardActions.layout()]}
          onValueChange={(details) => {
            const value = details.value[0];
            if (value !== undefined && isLayout(value)) {
              void keyboardActions.setLayout(value).catch((error) => {
                // eslint-disable-next-line no-console
                console.error("Unable to change the host layout.", error);
              });
            }
          }}
        >
          <Select.Control>
            <Select.Trigger
              class="
                flex w-full items-center cursor-pointer
                text-neutral-600 dark:text-neutral-400
                hover:text-neutral-900 dark:hover:text-neutral-100
              "
              aria-label="Select host layout"
            >
              <Select.Indicator class="flex justify-center py-1 pr-1 [&_svg]:size-5 [&_svg]:stroke-2">
                <ChevronsUpDownIcon />
              </Select.Indicator>
              <Select.ValueText
                class="
                  min-w-0 flex-1 border-0 bg-transparent py-1 pr-2
                  text-start outline-none
                "
                placeholder="Select host layout"
              />
            </Select.Trigger>
          </Select.Control>
          <Select.Positioner>
            <Select.Content
              class="
                z-1000 min-w-(--reference-width) overflow-hidden rounded-xl border
                shadow-md
                border-neutral-200 bg-neutral-50 text-neutral-900
                dark:border-neutral-800 dark:bg-neutral-900 dark:text-neutral-100
              "
            >
              <For each={hostLayoutCollection.items}>
                {(item) => (
                  <Select.Item
                    class="
                      m-1 flex items-center justify-between gap-4 rounded-lg p-2
                      cursor-pointer transition-[background] duration-50 ease-[ease]
                      data-highlighted:bg-neutral-200 dark:data-highlighted:bg-neutral-800
                    "
                    item={item}
                  >
                    <Select.ItemText>{item.label}</Select.ItemText>
                    <Select.ItemIndicator>✓</Select.ItemIndicator>
                  </Select.Item>
                )}
              </For>
            </Select.Content>
          </Select.Positioner>
        </Select.Root>
      </OptionsSection>

      <OptionsSection title="Global Settings">
        <OptionsButtonList>
          <li>
            <OptionsButton
              onClick={() => {
                toast.createToast({
                  title: "Not supported yet",
                  description: "Combos are not supported yet.",
                  type: "info",
                });
              }}
            >
              Combos
            </OptionsButton>
          </li>
        </OptionsButtonList>
      </OptionsSection>

      <OptionsSection title="Layout Import/Export">
        <OptionsButtonList>
          <li>
            <OptionsButton>Import JSON</OptionsButton>
          </li>
          <li>
            <OptionsButton>Export JSON</OptionsButton>
          </li>
          <Show when={props.onPrint}>
            <li>
              <OptionsButton onClick={() => props.onPrint?.()}>
                Print
              </OptionsButton>
            </li>
          </Show>
        </OptionsButtonList>
      </OptionsSection>
    </div>
  );
};

export default Options;
