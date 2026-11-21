import { Collapsible, RadioGroup } from "@ark-ui/solid";
import { Check, Keyboard, Monitor, Moon, Sun, Unplug } from "lucide-solid";
import { DeviceType } from "lokey-editor-odyn/core";
import {
  For,
  Show,
  createEffect,
  createSignal,
  onCleanup,
  useContext,
} from "solid-js";
import type { Component, JSX } from "solid-js";

import {
  DeviceCapabilitiesContext,
  useConnection,
  useTheme,
} from "../contexts";
import type { ThemePreference } from "../contexts";
import Button from "./Button";
import styles from "./Header.module.css";

type HeaderProps = {
  title: JSX.Element;
  class?: string;
};

const themeOptions: { value: ThemePreference; label: string }[] = [
  { value: "light", label: "Light" },
  { value: "dark", label: "Dark" },
  { value: "system", label: "System" },
];

const dropdownContentClass = `
  ${styles.content}
  sm:absolute sm:top-full sm:right-0 sm:mt-2
  sm:rounded-xl sm:border sm:border-neutral-300
  sm:bg-neutral-200/70 sm:shadow-md/5 sm:backdrop-blur-md
  sm:dark:border-neutral-750 sm:dark:bg-neutral-800/70
`;

const deviceTypeLabel = (type: DeviceType): string => {
  switch (type) {
    case DeviceType.KEYBOARD:
      return "Keyboard";
    case DeviceType.MOUSE:
      return "Mouse";
    case DeviceType.MIDI_CONTROLLER:
      return "MIDI Controller";
    default:
      return "Unknown";
  }
};

const DeviceIcon: Component<{ type: DeviceType }> = (props) => {
  switch (props.type) {
    case DeviceType.KEYBOARD:
      return <Keyboard aria-hidden="true" />;
    default:
      return (
        <span
          aria-hidden="true"
          class="inline-block size-2 rounded-full bg-current"
        />
      );
  }
};

const ThemePreferenceIcon: Component<{ value: ThemePreference }> = (props) => {
  switch (props.value) {
    case "light":
      return <Sun size="18" class="shrink-0" aria-hidden="true" />;
    case "dark":
      return <Moon size="18" class="shrink-0" aria-hidden="true" />;
    case "system":
      return <Monitor size="18" class="shrink-0" aria-hidden="true" />;
  }
};

const createDropdown = () => {
  const [open, setOpen] = createSignal(false);

  let triggerRef: HTMLElement | undefined;
  let contentRef: HTMLElement | undefined;

  createEffect(() => {
    if (!open()) {
      return;
    }

    const handlePointerDown = (event: PointerEvent) => {
      const target = event.target;
      if (!(target instanceof Node)) {
        return;
      }

      if (contentRef?.contains(target) || triggerRef?.contains(target)) {
        return;
      }

      setOpen(false);
    };

    document.addEventListener("pointerdown", handlePointerDown, true);
    onCleanup(() =>
      document.removeEventListener("pointerdown", handlePointerDown, true),
    );
  });

  return {
    open,
    setOpen,
    setTriggerRef: (element: HTMLElement) => {
      triggerRef = element;
    },
    setContentRef: (element: HTMLElement) => {
      contentRef = element;
    },
  };
};

export const Header: Component<HeaderProps> = (props) => {
  const { connection, disconnect } = useConnection();
  const { theme, themePreference, setThemePreference } = useTheme();
  const deviceCapabilities = useContext(DeviceCapabilitiesContext);

  const deviceDropdown = createDropdown();
  const themeDropdown = createDropdown();

  const isConnected = () => connection().status === "connected";

  const device = () => deviceCapabilities?.device();

  const deviceName = () => {
    const state = device();
    return state?.status === "retrieved" ? state.deviceName : "Unknown Device";
  };

  const deviceType = (): DeviceType => {
    const state = device();
    return state?.status === "retrieved" ? state.deviceType : DeviceType.OTHER;
  };

  const deviceId = (): number | undefined => {
    const state = device();
    return state?.status === "retrieved" ? state.deviceId : undefined;
  };

  const deviceIdLabel = () => {
    const id = deviceId();
    return id === undefined ? "Unknown" : `0x${id.toString(16).toUpperCase()}`;
  };

  createEffect(() => {
    if (!isConnected()) {
      deviceDropdown.setOpen(false);
    }
  });

  return (
    <>
      <header
        class={`
          fixed top-0 inset-x-2 z-100 mx-auto mt-2
          before:absolute before:inset-0 before:-z-10 before:rounded-xl
          before:border before:border-neutral-300
          before:bg-neutral-200/80 before:backdrop-blur-md
          dark:before:border-neutral-750 dark:before:bg-neutral-800/80
          shadow-md/5 rounded-xl
          ${props.class ?? ""}
        `}
      >
        <Collapsible.Root
          open={deviceDropdown.open()}
          onOpenChange={(details) => {
            deviceDropdown.setOpen(details.open);
            if (details.open) {
              themeDropdown.setOpen(false);
            }
          }}
          lazyMount
          unmountOnExit
        >
          <div class="flex h-[calc(var(--header-height)-8px)] items-center justify-between px-2">
            {props.title}

            <div class="flex items-center gap-1">
              <Show when={isConnected()}>
                <Collapsible.Trigger
                  asChild={(triggerProps) => (
                    <Button
                      {...triggerProps()}
                      ref={deviceDropdown.setTriggerRef}
                      class="
                        py-0! px-2.5!
                        gap-2 bg-transparent! hover:bg-neutral-400/30!
                        data-[state=open]:bg-neutral-400/30!
                        dark:hover:bg-neutral-600/30!
                        dark:data-[state=open]:bg-neutral-600/30!
                        [&_svg]:size-[18px]
                      "
                    >
                      <DeviceIcon type={deviceType()} />
                      <span class="max-w-40 truncate">{deviceName()}</span>
                    </Button>
                  )}
                />
              </Show>
              <Button
                ref={themeDropdown.setTriggerRef}
                type="button"
                aria-label="Theme"
                title="Theme"
                aria-expanded={themeDropdown.open()}
                data-state={themeDropdown.open() ? "open" : "closed"}
                class="
                  py-0! px-2.5!
                  bg-transparent! hover:bg-neutral-400/30!
                  data-[state=open]:bg-neutral-400/30!
                  dark:hover:bg-neutral-600/30!
                  dark:data-[state=open]:bg-neutral-600/30!
                  [&_svg]:size-[18px]
                "
                onClick={() => {
                  const open = !themeDropdown.open();
                  themeDropdown.setOpen(open);
                  if (open) {
                    deviceDropdown.setOpen(false);
                  }
                }}
              >
                <Show when={theme() === "dark"} fallback={<Sun />}>
                  <Moon />
                </Show>
              </Button>
            </div>
          </div>

          <Show when={isConnected()}>
            <Collapsible.Content
              ref={deviceDropdown.setContentRef}
              class={`${dropdownContentClass} sm:w-72`}
            >
              <div
                class="
                  flex flex-col items-end
                  border-t border-neutral-300
                  dark:border-neutral-750
                  sm:border-t-0
                "
              >
                <dl class="grid grid-cols-[auto_auto] gap-x-4 gap-y-2 px-4 py-3 text-sm w-full">
                  <dt class="text-neutral-600 dark:text-neutral-400">
                    Device ID
                  </dt>
                  <dd class="text-right font-mono">{deviceIdLabel()}</dd>
                  <dt class="text-neutral-600 dark:text-neutral-400">
                    Device Type
                  </dt>
                  <dd class="text-right">{deviceTypeLabel(deviceType())}</dd>
                </dl>

                <div class="w-full h-px bg-neutral-300 dark:bg-neutral-750" />

                <Button
                  class="
                    w-[calc(100%-0.5rem)] m-1 flex gap-2 min-h-8.5! h-8.5!
                    bg-transparent! hover:bg-neutral-300! dark:hover:bg-neutral-750!
                  "
                  onClick={() => {
                    deviceDropdown.setOpen(false);
                    void disconnect();
                  }}
                >
                  <Unplug size="18" stroke-width="2" />
                  Disconnect
                </Button>
              </div>
            </Collapsible.Content>
          </Show>
        </Collapsible.Root>

        <Collapsible.Root
          open={themeDropdown.open()}
          onOpenChange={(details) => {
            themeDropdown.setOpen(details.open);
            if (details.open) {
              deviceDropdown.setOpen(false);
            }
          }}
          lazyMount
          unmountOnExit
        >
          <Collapsible.Content
            ref={themeDropdown.setContentRef}
            class={`${dropdownContentClass} sm:w-42`}
          >
            <div class="border-t border-neutral-300 dark:border-neutral-750 sm:border-t-0">
              <RadioGroup.Root
                value={themePreference()}
                onValueChange={(details) => {
                  const value = details.value;
                  if (
                    value === "light" ||
                    value === "dark" ||
                    value === "system"
                  ) {
                    setThemePreference(value);
                    themeDropdown.setOpen(false);
                  }
                }}
                onClick={(event) => {
                  const target = event.target;
                  if (
                    target instanceof HTMLInputElement &&
                    target.type === "radio"
                  ) {
                    themeDropdown.setOpen(false);
                  }
                }}
                class="flex flex-col gap-0.5 p-1"
              >
                <RadioGroup.Label class="sr-only">Theme</RadioGroup.Label>
                <For each={themeOptions}>
                  {(option) => (
                    <RadioGroup.Item
                      value={option.value}
                      class="
                        flex items-center gap-2.5 rounded-lg px-2.5 text-sm
                        h-8.5 min-h-8.5 cursor-pointer select-none
                        bg-transparent hover:bg-neutral-300 dark:hover:bg-neutral-750
                        data-focus-visible:outline-2
                        data-focus-visible:outline-focused
                        data-focus-visible:-outline-offset-2
                      "
                    >
                      <ThemePreferenceIcon value={option.value} />
                      <RadioGroup.ItemText class="flex-1">
                        {option.label}
                      </RadioGroup.ItemText>
                      <Show when={themePreference() === option.value}>
                        <Check size="18" class="shrink-0" aria-hidden="true" />
                      </Show>
                      <RadioGroup.ItemHiddenInput />
                    </RadioGroup.Item>
                  )}
                </For>
              </RadioGroup.Root>
            </div>
          </Collapsible.Content>
        </Collapsible.Root>
      </header>
      <div class="h-(--header-height)"></div>
    </>
  );
};

export default Header;
