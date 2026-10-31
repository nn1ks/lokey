import {
  ArrowBigUp,
  ArrowRightToLine,
  CircleChevronDown,
  CircleOff,
  Command,
  CornerDownLeft,
  Delete,
  Ellipsis,
  Layers2,
  Option,
  Space as SpaceIcon,
} from "lucide-solid";
import type { Component, JSX, ParentProps } from "solid-js";

export interface KeyActionRender {
  render(level: number, layerCount: number): JSX.Element;
  renderTooltip(): JSX.Element;
}

export const TextSubtitleCombination: Component<{
  text: string;
  subtitle: string;
}> = (props) => (
  <div class="relative mt-0.5 flex flex-col items-center">
    <div>{props.text}</div>
    <div class="text-[9px]">{props.subtitle}</div>
  </div>
);

export const IconSubtitleCombination: Component<{
  icon: JSX.Element;
  subtitle: string;
}> = (props) => (
  <div class="relative mt-0.5 flex flex-col items-center">
    <div>{props.icon}</div>
    <div class="-mt-1 text-[9px]">{props.subtitle}</div>
  </div>
);

export const Labelled: Component<ParentProps<{ label: string }>> = (props) => (
  <div class="relative flex h-full w-full items-center justify-center">
    <div
      class="
        absolute left-0 ml-0.5 h-full text-center text-[7px] font-semibold
        tracking-widest text-neutral-500 [text-orientation:sideways]
        [writing-mode:sideways-lr] uppercase
      "
    >
      {props.label}
    </div>
    {props.children}
  </div>
);

export const Split: Component<{
  action1: JSX.Element;
  action2: JSX.Element;
}> = (props) => (
  <div class="h-full w-full">
    <div class="flex h-1/2 min-h-1/2 items-center justify-center text-[0.8em]">
      {props.action1}
    </div>
    <div
      class="
        absolute top-1/2 w-full -translate-y-1/2 border-t-[1.8px]
        border-dashed border-neutral-400 dark:border-neutral-600
      "
    />
    <div class="flex h-1/2 min-h-1/2 items-center justify-center text-[0.8em]">
      {props.action2}
    </div>
  </div>
);

export class NoAction implements KeyActionRender {
  render(_level: number, _layerCount: number): JSX.Element {
    return <CircleOff />;
  }

  renderTooltip(): JSX.Element {
    return <>None</>;
  }
}

export class Transparent implements KeyActionRender {
  render(_level: number, _layerCount: number): JSX.Element {
    return <CircleChevronDown />;
  }

  renderTooltip(): JSX.Element {
    return <>Transparent</>;
  }
}

export class Layer implements KeyActionRender {
  constructor(public readonly index: number) {}

  render(_level: number, _layerCount: number): JSX.Element {
    const layerName = `Layer ${this.index + 1}`;
    return <IconSubtitleCombination icon={<Layers2 />} subtitle={layerName} />;
  }

  renderTooltip(): JSX.Element {
    return <>Layer {this.index + 1}</>;
  }
}

export class HoldTap<
  H extends KeyAction,
  T extends KeyAction,
> implements KeyActionRender {
  constructor(
    public readonly holdAction: H,
    public readonly tapAction: T,
    /** Tapping term in milliseconds. */
    public readonly tappingTerm: number = 200,
  ) {}

  render(level: number, layerCount: number): JSX.Element {
    if (level > 0) return <Ellipsis />;

    return (
      <Split
        action1={
          <Labelled label="Tap">
            {this.tapAction.render(level + 1, layerCount)}
          </Labelled>
        }
        action2={
          <Labelled label="Hold">
            {this.holdAction.render(level + 1, layerCount)}
          </Labelled>
        }
      />
    );
  }

  renderTooltip(): JSX.Element {
    return (
      <div class="flex items-stretch gap-2">
        <div class="flex min-w-12 flex-col items-center gap-1">
          <div class="text-neutral-500 text-[0.65rem] font-semibold tracking-[0.08em] uppercase">
            Tap
          </div>
          <div class="flex min-h-6 items-center justify-center">
            {this.tapAction.renderTooltip()}
          </div>
        </div>
        <div class="w-px bg-neutral-600 dark:bg-neutral-400" />
        <div class="flex min-w-12 flex-col items-center gap-1">
          <div class="text-neutral-500 text-[0.65rem] font-semibold tracking-[0.08em] uppercase">
            Hold
          </div>
          <div class="flex min-h-6 items-center justify-center">
            {this.holdAction.renderTooltip()}
          </div>
        </div>
      </div>
    );
  }
}

export class Sequence<
  A1 extends KeyAction,
  A2 extends KeyAction,
> implements KeyActionRender {
  constructor(
    public readonly action1: A1,
    public readonly action2: A2,
  ) {}

  render(level: number, layerCount: number): JSX.Element {
    if (level > 0) return <Ellipsis />;

    return (
      <Split
        action1={
          <Labelled label="1">
            {this.action1.render(level + 1, layerCount)}
          </Labelled>
        }
        action2={
          <Labelled label="2">
            {this.action2.render(level + 1, layerCount)}
          </Labelled>
        }
      />
    );
  }

  renderTooltip(): JSX.Element {
    return (
      <div class="flex items-stretch gap-2">
        <div class="flex min-w-12 flex-col items-center gap-1">
          <div class="text-neutral-500 text-[0.65rem] font-semibold tracking-[0.08em] uppercase">
            1
          </div>
          <div class="flex min-h-6 items-center justify-center">
            {this.action1.renderTooltip()}
          </div>
        </div>
        <div class="w-px bg-neutral-600 dark:bg-neutral-400" />
        <div class="flex min-w-12 flex-col items-center gap-1">
          <div class="text-neutral-500 text-[0.65rem] font-semibold tracking-[0.08em] uppercase">
            2
          </div>
          <div class="flex min-h-6 items-center justify-center">
            {this.action2.renderTooltip()}
          </div>
        </div>
      </div>
    );
  }
}

export class KeyCode implements KeyActionRender {
  private constructor(public readonly value: string) {}

  render(_level: number, _layerCount: number): JSX.Element {
    switch (this) {
      case KeyCode.LControl:
        return <TextSubtitleCombination text="Ctrl" subtitle="Left" />;
      case KeyCode.RControl:
        return <TextSubtitleCombination text="Ctrl" subtitle="Right" />;
      case KeyCode.LShift:
        return (
          <IconSubtitleCombination
            icon={<ArrowBigUp />}
            subtitle="Left Shift"
          />
        );
      case KeyCode.RShift:
        return (
          <IconSubtitleCombination
            icon={<ArrowBigUp />}
            subtitle="Right Shift"
          />
        );
      case KeyCode.LAlt:
        return (
          <IconSubtitleCombination icon={<Option />} subtitle="Left Alt" />
        );
      case KeyCode.RAlt:
        return (
          <IconSubtitleCombination icon={<Option />} subtitle="Right Alt" />
        );
      case KeyCode.LGui:
        return (
          <IconSubtitleCombination icon={<Command />} subtitle="Left Super" />
        );
      case KeyCode.RGui:
        return (
          <IconSubtitleCombination icon={<Command />} subtitle="Right Super" />
        );
      case KeyCode.Enter:
        return <CornerDownLeft />;
      case KeyCode.Escape:
        return <>ESC</>;
      case KeyCode.Backspace:
        return <Delete />;
      case KeyCode.Tab:
        return <ArrowRightToLine />;
      case KeyCode.Space:
        return <SpaceIcon />;
      default:
        return <>{this.displayValue()}</>;
    }
  }

  renderTooltip(): JSX.Element {
    const infoValue = this.additionalTooltipValue();

    return (
      <>
        {this.displayValue()}
        {infoValue !== null && <> ({infoValue})</>}
      </>
    );
  }

  private displayValue(): string {
    const displayValues: Record<string, string> = {
      LControl: "Left Control",
      RControl: "Right Control",
      LShift: "Left Shift",
      RShift: "Right Shift",
      LAlt: "Left Alt",
      RAlt: "Right Alt",
      LGui: "Left Gui",
      RGui: "Right Gui",
      N1: "1",
      N2: "2",
      N3: "3",
      N4: "4",
      N5: "5",
      N6: "6",
      N7: "7",
      N8: "8",
      N9: "9",
      N0: "0",
      Minus: "-",
      Equal: "=",
      LeftBracket: "[",
      RightBracket: "]",
      Backslash: "\\",
      Hash: "#",
      Semicolon: ";",
      Apostrophe: "'",
      Grave: "`",
      Comma: ",",
      Dot: ".",
      Slash: "/",
    };

    return displayValues[this.value] ?? this.value;
  }

  private additionalTooltipValue(): string | null {
    switch (this.value) {
      case "Minus":
        return "Minus";
      case "Equal":
        return "Equal";
      case "LeftBracket":
        return "Left Bracket";
      case "RightBracket":
        return "Right Bracket";
      case "Backslash":
        return "Backslash";
      case "Hash":
        return "Hash";
      case "Semicolon":
        return "Semicolon";
      case "Apostrophe":
        return "Apostrophe";
      case "Grave":
        return "Grave";
      case "Comma":
        return "Comma";
      case "Dot":
        return "Dot";
      case "Slash":
        return "Slash";
      default:
        return null;
    }
  }

  toString(): string {
    return this.displayValue();
  }

  static readonly LControl = new KeyCode("LControl");
  static readonly RControl = new KeyCode("RControl");
  static readonly LShift = new KeyCode("LShift");
  static readonly RShift = new KeyCode("RShift");
  static readonly LAlt = new KeyCode("LAlt");
  static readonly RAlt = new KeyCode("RAlt");
  static readonly LGui = new KeyCode("LGui");
  static readonly RGui = new KeyCode("RGui");
  static readonly A = new KeyCode("A");
  static readonly B = new KeyCode("B");
  static readonly C = new KeyCode("C");
  static readonly D = new KeyCode("D");
  static readonly E = new KeyCode("E");
  static readonly F = new KeyCode("F");
  static readonly G = new KeyCode("G");
  static readonly H = new KeyCode("H");
  static readonly I = new KeyCode("I");
  static readonly J = new KeyCode("J");
  static readonly K = new KeyCode("K");
  static readonly L = new KeyCode("L");
  static readonly M = new KeyCode("M");
  static readonly N = new KeyCode("N");
  static readonly O = new KeyCode("O");
  static readonly P = new KeyCode("P");
  static readonly Q = new KeyCode("Q");
  static readonly R = new KeyCode("R");
  static readonly S = new KeyCode("S");
  static readonly T = new KeyCode("T");
  static readonly U = new KeyCode("U");
  static readonly V = new KeyCode("V");
  static readonly W = new KeyCode("W");
  static readonly X = new KeyCode("X");
  static readonly Y = new KeyCode("Y");
  static readonly Z = new KeyCode("Z");
  static readonly N1 = new KeyCode("N1");
  static readonly N2 = new KeyCode("N2");
  static readonly N3 = new KeyCode("N3");
  static readonly N4 = new KeyCode("N4");
  static readonly N5 = new KeyCode("N5");
  static readonly N6 = new KeyCode("N6");
  static readonly N7 = new KeyCode("N7");
  static readonly N8 = new KeyCode("N8");
  static readonly N9 = new KeyCode("N9");
  static readonly N0 = new KeyCode("N0");
  static readonly Enter = new KeyCode("Enter");
  static readonly Escape = new KeyCode("Escape");
  static readonly Backspace = new KeyCode("Backspace");
  static readonly Tab = new KeyCode("Tab");
  static readonly Space = new KeyCode("Space");
  static readonly Minus = new KeyCode("Minus");
  static readonly Equal = new KeyCode("Equal");
  static readonly LeftBracket = new KeyCode("LeftBracket");
  static readonly RightBracket = new KeyCode("RightBracket");
  static readonly Backslash = new KeyCode("Backslash");
  static readonly Hash = new KeyCode("Hash");
  static readonly Semicolon = new KeyCode("Semicolon");
  static readonly Apostrophe = new KeyCode("Apostrophe");
  static readonly Grave = new KeyCode("Grave");
  static readonly Comma = new KeyCode("Comma");
  static readonly Dot = new KeyCode("Dot");
  static readonly Slash = new KeyCode("Slash");
  static readonly CapsLock = new KeyCode("CapsLock");
  static readonly F1 = new KeyCode("F1");
  static readonly F2 = new KeyCode("F2");
  static readonly F3 = new KeyCode("F3");
  static readonly F4 = new KeyCode("F4");
  static readonly F5 = new KeyCode("F5");
  static readonly F6 = new KeyCode("F6");
  static readonly F7 = new KeyCode("F7");
  static readonly F8 = new KeyCode("F8");
  static readonly F9 = new KeyCode("F9");
  static readonly F10 = new KeyCode("F10");
  static readonly F11 = new KeyCode("F11");
  static readonly F12 = new KeyCode("F12");
  static readonly F13 = new KeyCode("F13");
  static readonly F14 = new KeyCode("F14");
  static readonly F15 = new KeyCode("F15");
  static readonly F16 = new KeyCode("F16");
  static readonly F17 = new KeyCode("F17");
  static readonly F18 = new KeyCode("F18");
  static readonly F19 = new KeyCode("F19");
  static readonly F20 = new KeyCode("F20");
  static readonly F21 = new KeyCode("F21");
  static readonly F22 = new KeyCode("F22");
  static readonly F23 = new KeyCode("F23");
  static readonly F24 = new KeyCode("F24");
  static readonly PrintScreen = new KeyCode("PrintScreen");
  static readonly ScrollLock = new KeyCode("ScrollLock");
  static readonly Pause = new KeyCode("Pause");
  static readonly Insert = new KeyCode("Insert");
  static readonly Home = new KeyCode("Home");
  static readonly PageUp = new KeyCode("PageUp");
  static readonly Delete = new KeyCode("Delete");
  static readonly End = new KeyCode("PageDown");
  static readonly Right = new KeyCode("Right");
  static readonly Left = new KeyCode("Left");
  static readonly Down = new KeyCode("Down");
}

export type KeyAction =
  | NoAction
  | Transparent
  | Layer
  | HoldTap<KeyAction, KeyAction>
  | Sequence<KeyAction, KeyAction>
  | KeyCode;

/**
 * The actions of a single key: either one action shared across all layers, or
 * one action per layer.
 */
export type RootKeyAction =
  | { kind: "single"; action: KeyAction }
  | { kind: "perLayer"; actions: KeyAction[] };
