import { Component } from "solid-js";
import { Keyboard, Lightbulb } from "lucide-solid";
import {
  HoldTap,
  KeyAction,
  KeyCode,
  Layer,
  LayerId,
  NoAction,
  Transparent,
} from "lokey-editor-core";
import {
  PageTabTriggerContent,
  PageTabs,
  Tabs,
} from "lokey-editor-core/components";
import { LedArrayPage } from "lokey-editor-core/components/led-array";
import { KeymapPage } from "lokey-editor-core/components/keymap";
import FinKeymapLayout from "./FinKeymapLayout";
import {
  FinKeyActions,
  FinKeyActionsContext,
} from "../contexts/FinKeyActionsContext";

// TODO: remove example key actions
const layer1Actions = (): KeyAction[] => [
  KeyCode.Q,
  KeyCode.W,
  KeyCode.F,
  KeyCode.P,
  KeyCode.B,
  KeyCode.J,
  KeyCode.L,
  KeyCode.U,
  KeyCode.Y,
  KeyCode.Slash,
  KeyCode.A,
  KeyCode.R,
  KeyCode.S,
  KeyCode.T,
  KeyCode.G,
  KeyCode.M,
  KeyCode.N,
  KeyCode.E,
  KeyCode.I,
  KeyCode.O,
  KeyCode.Z,
  KeyCode.X,
  KeyCode.C,
  KeyCode.D,
  KeyCode.V,
  KeyCode.K,
  KeyCode.H,
  KeyCode.Comma,
  KeyCode.Dot,
  KeyCode.Minus,
  new HoldTap(new Layer(1), KeyCode.Escape),
  KeyCode.LShift,
  KeyCode.Space,
  KeyCode.Enter,
  new Layer(3),
  new HoldTap(new HoldTap(KeyCode.A, KeyCode.B), KeyCode.Backspace),
];

const layer2Actions = (): KeyAction[] => [
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new NoAction(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
  new Transparent(),
];

const finKeyActions: FinKeyActions = [
  [new LayerId(), layer1Actions()],
  [new LayerId(), layer2Actions()],
];

export const FinPage: Component = () => {
  return (
    <FinKeyActionsContext.Provider value={finKeyActions}>
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
          <KeymapPage class="max-w-[calc(1130px+2*var(--page-padding-x)+1px)]">
            <FinKeymapLayout />
          </KeymapPage>
        </Tabs.Content>

        <Tabs.Content value="led-array">
          <LedArrayPage />
        </Tabs.Content>
      </PageTabs>
    </FinKeyActionsContext.Provider>
  );
};
