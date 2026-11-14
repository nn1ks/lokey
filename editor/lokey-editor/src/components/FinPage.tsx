import { Component } from "solid-js";
import { Keyboard, Lightbulb } from "lucide-solid";
import {
  PageTabTriggerContent,
  PageTabs,
  Tabs,
} from "lokey-editor-core/components";
import { LedArrayPage } from "lokey-editor-core/components/led-array";
import { KeymapPage } from "lokey-editor-core/components/keymap";
import { createDefaultActionRegistry } from "lokey-editor-core/key-action";
import FinKeymapLayout from "./FinKeymapLayout";
import FinKeymapLayoutPrint from "./FinKeymapLayoutPrint";

export const FinPage: Component = () => {
  const actionRegistry = createDefaultActionRegistry();
  return (
    <PageTabs defaultValue="keymap">
      <Tabs.List>
        <Tabs.Trigger value="keymap">
          <PageTabTriggerContent icon={<Keyboard />} title="Keymap" />
        </Tabs.Trigger>
        <Tabs.Trigger value="led-array">
          <PageTabTriggerContent icon={<Lightbulb />} title="LED Array" />
        </Tabs.Trigger>
      </Tabs.List>

      <Tabs.Content value="keymap">
        <KeymapPage
          class="max-w-[calc(1130px+2*var(--page-padding-x)+1px)]"
          actionRegistry={actionRegistry}
          printView={FinKeymapLayoutPrint}
        >
          <FinKeymapLayout />
        </KeymapPage>
      </Tabs.Content>

      <Tabs.Content value="led-array">
        <LedArrayPage />
      </Tabs.Content>
    </PageTabs>
  );
};
