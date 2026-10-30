import { Lightbulb } from "lucide-solid";
import { Show } from "solid-js";
import type { Component } from "solid-js";
import PageHeading from "../PageHeading";
import ToggleRow from "./ToggleRow";
import PoweredByLokey from "../PoweredByLokey";
import { LedArrayContextProvider, useLedArray } from "../../contexts";

const LedArrayContent: Component = () => {
  const ledArray = useLedArray();

  return (
    <main class="w-full max-w-200 px-(--page-padding-x) mb-8">
      <PageHeading
        icon={<Lightbulb />}
        title="LED Array"
        description="Choose which events trigger LED animations"
        actions={<></>}
      />
      <div
        class="
          mt-2 flex flex-col
          [&>*:first-child]:rounded-t-2xl
          [&>*:last-child]:rounded-b-2xl
          [&>*:not(:last-child)]:border-b
          [&>*:not(:last-child)]:border-neutral-300
          dark:[&>*:not(:last-child)]:border-neutral-800
        "
      >
        <Show when={ledArray.capabilities()?.boot}>
          <ToggleRow
            label="Boot"
            description="Shows a brief LED animation during device startup."
            enabled={ledArray.enabled()?.boot ?? false}
            onCheckedChange={(checked) =>
              void ledArray.setHook("boot", checked)
            }
          />
        </Show>
        <Show when={ledArray.capabilities()?.layerChange}>
          <ToggleRow
            label="Layer Change"
            description="Briefly lights an LED whenever the currently active layer changes."
            enabled={ledArray.enabled()?.layerChange ?? false}
            onCheckedChange={(checked) =>
              void ledArray.setHook("layerChange", checked)
            }
          />
        </Show>
        <Show when={ledArray.capabilities()?.blePairing}>
          <ToggleRow
            label="Bluetooth Pairing"
            description="Shows an LED animation while the device is in pairing mode."
            enabled={ledArray.enabled()?.blePairing ?? false}
            onCheckedChange={(checked) =>
              void ledArray.setHook("blePairing", checked)
            }
          />
        </Show>
        <Show when={ledArray.capabilities()?.bleProfile}>
          <ToggleRow
            label="Bluetooth Profile"
            description="Briefly lights an LED whenever the active Bluetooth profile changes."
            enabled={ledArray.enabled()?.bleProfile ?? false}
            onCheckedChange={(checked) =>
              void ledArray.setHook("bleProfile", checked)
            }
          />
        </Show>
      </div>
      <PoweredByLokey />
    </main>
  );
};

export const LedArrayPage: Component = () => {
  return (
    <LedArrayContextProvider>
      <LedArrayContent />
    </LedArrayContextProvider>
  );
};

export default LedArrayPage;
