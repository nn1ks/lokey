import { Select, createListCollection } from "@ark-ui/solid/select";
import { createSignal, For } from "solid-js";
import type { Component, JSX } from "solid-js";
import { ChevronsUpDownIcon } from "lucide-solid";

const hostLayoutOptions = [
  { label: "English (US)", value: "english-us" },
  { label: "German", value: "german" },
];

const hostLayoutCollection = createListCollection({ items: hostLayoutOptions });

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

const OptionsButton: Component<{ children: JSX.Element }> = (props) => (
  <button
    class="
      w-fit cursor-pointer bg-transparent
      underline decoration-dashed
      hover:text-neutral-900 dark:hover:text-neutral-100
    "
  >
    {props.children}
  </button>
);

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

export const Options: Component = () => {
  const [hostLayout, setHostLayout] = createSignal<string[]>(["english-us"]);

  return (
    <div class="mt-6 flex flex-col gap-6 sm:flex-row">
      <OptionsSection title="Host Layout">
        <Select.Root
          class="relative w-40 min-w-40 max-w-40"
          collection={hostLayoutCollection}
          value={hostLayout()}
          onValueChange={(details) => setHostLayout(details.value)}
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
                shadow-[0_4px_8px_rgb(0_0_0/10%)]
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
            <OptionsButton>Key Behaviour</OptionsButton>
          </li>
          <li>
            <OptionsButton>Macros</OptionsButton>
          </li>
          <li>
            <OptionsButton>Combos</OptionsButton>
          </li>
        </OptionsButtonList>
      </OptionsSection>

      <OptionsSection title="Layout Export">
        <OptionsButtonList>
          <li>
            <OptionsButton>Export JSON</OptionsButton>
          </li>
          <li>
            <OptionsButton>Export SVG</OptionsButton>
          </li>
          <li>
            <OptionsButton>Print</OptionsButton>
          </li>
        </OptionsButtonList>
      </OptionsSection>
    </div>
  );
};

export default Options;
