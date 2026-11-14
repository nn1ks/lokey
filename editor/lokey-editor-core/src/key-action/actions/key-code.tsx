import * as capnp from "capnp-es";
import {
  ArrowBigUp,
  ArrowBigUpDash,
  ArrowDown,
  ArrowLeft,
  ArrowRight,
  ArrowRightToLine,
  ArrowUp,
  Command,
  CornerDownLeft,
  CornerDownRight,
  Delete,
  Option,
  Power,
  Space,
  Space as SpaceIcon,
  TriangleAlert,
  Volume1,
  Volume2,
  VolumeX,
} from "lucide-solid";
import {
  KeyboardActionEntry,
  KeyCodeAction,
  KeyCodeType,
  OrderedAction,
} from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import type { KeyAction } from "../action";
import { buildAction, buildActionEntry } from "../codec/build";
import type { RawKeyCode } from "../layouts/types";
import {
  IconSubtitleCombination,
  Small,
  SmallTextSubtitleCombination,
  TextSubtitleCombination,
} from "../render";
import { None } from "./none";
import { Component, createMemo, For, JSX, Match, Show, Switch } from "solid-js";
import {
  DecodeContext,
  EncodeContext,
  PickerContext,
  RemapLayersContext,
} from "../contexts";
import assertNever from "assert-never";
import { useKeymapState } from "../../contexts";
import {
  KeyCodeValue,
  keyCodeValues,
  reverseKeyCodeValues,
} from "../key-code-value";

export class KeyCode implements KeyAction {
  static readonly actionId = ActionId.keyCode;
  readonly actionId = ActionId.keyCode;

  readonly #combinedActionEntryIds: number[] = [];

  constructor(
    public actionEntryId: number | undefined,
    readonly value: KeyCodeValue,
  ) {}

  encode(ctx: EncodeContext): KeyboardActionEntry {
    const encoding = keyCodeValues(ctx.hostLayout).get(this.value);
    if (!encoding) {
      console.warn(
        `Invalid key code "${this.value.value}" for layout "${ctx.hostLayout}".`,
      );
      const entryId = this.actionEntryId ?? ctx.nextActionEntryId();
      this.actionEntryId = entryId;
      return new None(entryId).encode(ctx);
    }

    if (encoding.kind === "single") {
      const action = buildAction(ActionId.keyCode, KeyCodeAction, (action) => {
        action.keyCode = encoding.rawKeyCode.code;
        action.type = KeyCodeType.KEYBOARD;
      });
      const entryId = this.actionEntryId ?? ctx.nextActionEntryId();
      this.actionEntryId = entryId;
      return buildActionEntry(entryId, action);
    }

    if (encoding.kind === "combined") {
      const action1 = buildAction(ActionId.keyCode, KeyCodeAction, (action) => {
        action.keyCode = encoding.rawKeyCode1.code;
        action.type = KeyCodeType.KEYBOARD;
      });
      const actionEntryId1 =
        this.#combinedActionEntryIds[0] ?? ctx.nextActionEntryId();
      this.#combinedActionEntryIds[0] = actionEntryId1;
      const actionEntry1 = buildActionEntry(actionEntryId1, action1);

      const action2 = buildAction(ActionId.keyCode, KeyCodeAction, (action) => {
        action.keyCode = encoding.rawKeyCode2.code;
        action.type = KeyCodeType.KEYBOARD;
      });
      const actionEntryId2 =
        this.#combinedActionEntryIds[1] ?? ctx.nextActionEntryId();
      this.#combinedActionEntryIds[1] = actionEntryId2;
      const actionEntry2 = buildActionEntry(actionEntryId2, action2);

      ctx.pushActionEntry(actionEntry1);
      ctx.pushActionEntry(actionEntry2);

      const action = buildAction(ActionId.ordered, OrderedAction, (action) => {
        action.action1EntryId = actionEntry1.entryId;
        action.action2EntryId = actionEntry2.entryId;
      });
      const entryId = this.actionEntryId ?? ctx.nextActionEntryId();
      this.actionEntryId = entryId;
      return buildActionEntry(entryId, action);
    }

    assertNever(encoding);
  }

  static decode(
    actionEntryId: number | undefined,
    actionPayload: Uint8Array,
    ctx: DecodeContext,
  ): KeyCode {
    const message = new capnp.Message(actionPayload, false);
    const keyCodeAction = message.getRoot(KeyCodeAction);

    if (keyCodeAction.type === KeyCodeType.KEYBOARD) {
      const keyCodeValue = reverseKeyCodeValues(ctx.hostLayout).get(
        keyCodeAction.keyCode,
      );
      if (!keyCodeValue) {
        throw new Error(
          `Unknown key code 0x${keyCodeAction.keyCode.toString(16)} for layout ${ctx.hostLayout}.`,
        );
      }

      return new KeyCode(actionEntryId, keyCodeValue);
    }

    if (keyCodeAction.type === KeyCodeType.CONSUMER) {
      throw new Error("Consumer key codes are not supported yet.");
    }

    throw new Error(`Unknown key code type "${keyCodeAction.type}".`);
  }

  renderButtonContent(level: number): JSX.Element {
    return <KeyCodeButtonContent keyCode={this} level={level} />;
  }

  renderTooltip(): JSX.Element {
    return <KeyCodeTooltip keyCode={this} />;
  }

  renderPicker(_ctx: PickerContext): JSX.Element {
    return this.renderButtonContent(0);
  }

  remapLayers(_ctx: RemapLayersContext): void {}
}

const KeyCodeButtonContent: Component<{ keyCode: KeyCode; level: number }> = (
  props,
) => {
  const { hostLayout } = useKeymapState();

  const buttonContent = renderButtonContent(props.keyCode.value, props.level);

  const isValid = createMemo(() => {
    const hostLayoutValue = hostLayout();
    if (!hostLayoutValue) {
      return false;
    }
    return (
      keyCodeValues(hostLayoutValue).get(props.keyCode.value) !== undefined
    );
  });

  return (
    <Switch>
      <Match when={isValid()}>{buttonContent}</Match>
      <Match when={!isValid()}>
        <div class="relative w-full h-full">
          <div class="w-full h-full flex justify-center items-center">
            {buttonContent}
          </div>
          <div class="absolute bottom-1 right-1 [&_svg]:size-[1em]! text-yellow-800 dark:text-yellow-200">
            <TriangleAlert />
          </div>
        </div>
      </Match>
    </Switch>
  );
};

function renderButtonContent(
  keyCodeValue: KeyCodeValue,
  level: number,
): JSX.Element {
  switch (keyCodeValue) {
    case KeyCodeValue.LControl:
      return level == 0 ? (
        <TextSubtitleCombination text="Ctrl" subtitle="Left" />
      ) : (
        <>Ctrl</>
      );
    case KeyCodeValue.RControl:
      return level == 0 ? (
        <TextSubtitleCombination text="Ctrl" subtitle="Right" />
      ) : (
        <>Ctrl</>
      );
    case KeyCodeValue.LShift:
      return level == 0 ? (
        <IconSubtitleCombination icon={<ArrowBigUp />} subtitle="Left Shift" />
      ) : (
        <ArrowBigUp />
      );
    case KeyCodeValue.RShift:
      return level == 0 ? (
        <IconSubtitleCombination icon={<ArrowBigUp />} subtitle="Right Shift" />
      ) : (
        <ArrowBigUp />
      );
    case KeyCodeValue.LAlt:
      return level == 0 ? (
        <IconSubtitleCombination icon={<Option />} subtitle="Left Alt" />
      ) : (
        <Option />
      );
    case KeyCodeValue.RAlt:
      return level == 0 ? (
        <IconSubtitleCombination icon={<Option />} subtitle="Right Alt" />
      ) : (
        <Option />
      );
    case KeyCodeValue.LGui:
      return level == 0 ? (
        <IconSubtitleCombination icon={<Command />} subtitle="Left Super" />
      ) : (
        <Command />
      );
    case KeyCodeValue.RGui:
      return level == 0 ? (
        <IconSubtitleCombination icon={<Command />} subtitle="Right Super" />
      ) : (
        <Command />
      );
    case KeyCodeValue.AE:
      return <>Ä</>;
    case KeyCodeValue.OE:
      return <>Ö</>;
    case KeyCodeValue.UE:
      return <>Ü</>;
    case KeyCodeValue.SharpS:
      return <>ß</>;
    case KeyCodeValue.N1:
      return <>1</>;
    case KeyCodeValue.N2:
      return <>2</>;
    case KeyCodeValue.N3:
      return <>3</>;
    case KeyCodeValue.N4:
      return <>4</>;
    case KeyCodeValue.N5:
      return <>5</>;
    case KeyCodeValue.N6:
      return <>6</>;
    case KeyCodeValue.N7:
      return <>7</>;
    case KeyCodeValue.N8:
      return <>8</>;
    case KeyCodeValue.N9:
      return <>9</>;
    case KeyCodeValue.N0:
      return <>0</>;
    case KeyCodeValue.Enter:
      return <CornerDownLeft />;
    case KeyCodeValue.Escape:
      return <>ESC</>;
    case KeyCodeValue.Backspace:
      return <Delete />;
    case KeyCodeValue.Tab:
      return <ArrowRightToLine />;
    case KeyCodeValue.Space:
      return <SpaceIcon />;
    case KeyCodeValue.CapsLock:
      return <ArrowBigUpDash />;
    case KeyCodeValue.Minus:
      return <>-</>;
    case KeyCodeValue.Equal:
      return <>=</>;
    case KeyCodeValue.LeftParen:
      return <>(</>;
    case KeyCodeValue.RightParen:
      return <>)</>;
    case KeyCodeValue.LeftBracket:
      return <>[</>;
    case KeyCodeValue.RightBracket:
      return <>]</>;
    case KeyCodeValue.LeftBrace:
      return <>{"{"}</>;
    case KeyCodeValue.RightBrace:
      return <>{"}"}</>;
    case KeyCodeValue.Backslash:
      return <>{"\\"}</>;
    case KeyCodeValue.Hash:
      return <>#</>;
    case KeyCodeValue.Semicolon:
      return <>;</>;
    case KeyCodeValue.Apostrophe:
      return <>'</>;
    case KeyCodeValue.Grave:
      return <>`</>;
    case KeyCodeValue.Comma:
      return <>,</>;
    case KeyCodeValue.Dot:
      return <>.</>;
    case KeyCodeValue.Slash:
      return <>/</>;
    case KeyCodeValue.Tilde:
      return <>~</>;
    case KeyCodeValue.ExclamationMark:
      return <>!</>;
    case KeyCodeValue.At:
      return <>@</>;
    case KeyCodeValue.Dollar:
      return <>$</>;
    case KeyCodeValue.Euro:
      return <>€</>;
    case KeyCodeValue.Percent:
      return <>%</>;
    case KeyCodeValue.Caret:
      return <>^</>;
    case KeyCodeValue.Ampersand:
      return <>{"&"}</>;
    case KeyCodeValue.Asterisk:
      return <>*</>;
    case KeyCodeValue.Underscore:
      return <>_</>;
    case KeyCodeValue.Plus:
      return <>+</>;
    case KeyCodeValue.Pipe:
      return <>|</>;
    case KeyCodeValue.Colon:
      return <>:</>;
    case KeyCodeValue.DoubleQuote:
      return <>{'"'}</>;
    case KeyCodeValue.GreaterThan:
      return <>{">"}</>;
    case KeyCodeValue.LessThan:
      return <>{"<"}</>;
    case KeyCodeValue.QuestionMark:
      return <>?</>;
    case KeyCodeValue.SectionSign:
      return <>§</>;
    case KeyCodeValue.Degree:
      return <>°</>;
    case KeyCodeValue.Acute:
      return <>`</>;
    case KeyCodeValue.PrintScreen:
      return <Small>Print Screen</Small>;
    case KeyCodeValue.ScrollLock:
      return <Small>Scroll Lock</Small>;
    case KeyCodeValue.PageUp:
      return <Small>Page Up</Small>;
    case KeyCodeValue.PageDown:
      return <Small>Page Down</Small>;
    case KeyCodeValue.Pause:
      return <Small>Pause</Small>;
    case KeyCodeValue.Insert:
      return <Small>Insert</Small>;
    case KeyCodeValue.Home:
      return <Small>Home</Small>;
    case KeyCodeValue.Delete:
      return <Small>Delete</Small>;
    case KeyCodeValue.End:
      return <Small>End</Small>;
    case KeyCodeValue.Left:
      return <ArrowLeft />;
    case KeyCodeValue.Right:
      return <ArrowRight />;
    case KeyCodeValue.Up:
      return <ArrowUp />;
    case KeyCodeValue.Down:
      return <ArrowDown />;
    case KeyCodeValue.Application:
      return <Small>Application</Small>;
    case KeyCodeValue.Power:
      return <Power />;
    case KeyCodeValue.Execute:
      return <Small>Execute</Small>;
    case KeyCodeValue.Help:
      return <Small>Help</Small>;
    case KeyCodeValue.Menu:
      return <Small>Menu</Small>;
    case KeyCodeValue.Select:
      return <Small>Select</Small>;
    case KeyCodeValue.Stop:
      return <Small>Stop</Small>;
    case KeyCodeValue.Again:
      return <Small>Again</Small>;
    case KeyCodeValue.Undo:
      return <Small>Undo</Small>;
    case KeyCodeValue.Cut:
      return <Small>Cut</Small>;
    case KeyCodeValue.Copy:
      return <Small>Copy</Small>;
    case KeyCodeValue.Paste:
      return <Small>Paste</Small>;
    case KeyCodeValue.Find:
      return <Small>Find</Small>;
    case KeyCodeValue.LockingCapsLock:
      return <Small>Locking Caps Lock</Small>;
    case KeyCodeValue.LockingNumLock:
      return <Small>Locking Num Lock</Small>;
    case KeyCodeValue.LockingScrollLock:
      return <Small>Locking Scroll Lock</Small>;
    case KeyCodeValue.Mute:
      return <VolumeX />;
    case KeyCodeValue.VolumeDown:
      return <Volume1 />;
    case KeyCodeValue.VolumeUp:
      return <Volume2 />;
    case KeyCodeValue.NonUsBackslash:
      return <Small>Non-US Backslash</Small>;
    case KeyCodeValue.International1:
      return <Small>Int 1</Small>;
    case KeyCodeValue.International2:
      return <Small>Int 2</Small>;
    case KeyCodeValue.International3:
      return <Small>Int 3</Small>;
    case KeyCodeValue.International4:
      return <Small>Int 4</Small>;
    case KeyCodeValue.International5:
      return <Small>Int 5</Small>;
    case KeyCodeValue.International6:
      return <Small>Int 6</Small>;
    case KeyCodeValue.International7:
      return <Small>Int 7</Small>;
    case KeyCodeValue.International8:
      return <Small>Int 8</Small>;
    case KeyCodeValue.International9:
      return <Small>Int 9</Small>;
    case KeyCodeValue.Lang1:
      return <Small>Lang 1</Small>;
    case KeyCodeValue.Lang2:
      return <Small>Lang 2</Small>;
    case KeyCodeValue.Lang3:
      return <Small>Lang 3</Small>;
    case KeyCodeValue.Lang4:
      return <Small>Lang 4</Small>;
    case KeyCodeValue.Lang5:
      return <Small>Lang 5</Small>;
    case KeyCodeValue.Lang6:
      return <Small>Lang 6</Small>;
    case KeyCodeValue.Lang7:
      return <Small>Lang 7</Small>;
    case KeyCodeValue.Lang8:
      return <Small>Lang 8</Small>;
    case KeyCodeValue.Lang9:
      return <Small>Lang 9</Small>;
    case KeyCodeValue.Kp1:
      return <TextSubtitleCombination text="1" subtitle="Keypad" />;
    case KeyCodeValue.Kp2:
      return <TextSubtitleCombination text="2" subtitle="Keypad" />;
    case KeyCodeValue.Kp3:
      return <TextSubtitleCombination text="3" subtitle="Keypad" />;
    case KeyCodeValue.Kp4:
      return <TextSubtitleCombination text="4" subtitle="Keypad" />;
    case KeyCodeValue.Kp5:
      return <TextSubtitleCombination text="5" subtitle="Keypad" />;
    case KeyCodeValue.Kp6:
      return <TextSubtitleCombination text="6" subtitle="Keypad" />;
    case KeyCodeValue.Kp7:
      return <TextSubtitleCombination text="7" subtitle="Keypad" />;
    case KeyCodeValue.Kp8:
      return <TextSubtitleCombination text="8" subtitle="Keypad" />;
    case KeyCodeValue.Kp9:
      return <TextSubtitleCombination text="9" subtitle="Keypad" />;
    case KeyCodeValue.Kp0:
      return <TextSubtitleCombination text="0" subtitle="Keypad" />;
    case KeyCodeValue.Kp00:
      return <TextSubtitleCombination text="00" subtitle="Keypad" />;
    case KeyCodeValue.Kp000:
      return <TextSubtitleCombination text="000" subtitle="Keypad" />;
    case KeyCodeValue.NumLock:
      return <Small>Num Lock</Small>;
    case KeyCodeValue.KpEnter:
      return (
        <IconSubtitleCombination icon={<CornerDownLeft />} subtitle="Keypad" />
      );
    case KeyCodeValue.KpTab:
      return (
        <IconSubtitleCombination
          icon={<ArrowRightToLine />}
          subtitle="Keypad"
        />
      );
    case KeyCodeValue.KpBackspace:
      return <IconSubtitleCombination icon={<Delete />} subtitle="Keypad" />;
    case KeyCodeValue.KpA:
      return <TextSubtitleCombination text="A" subtitle="Keypad" />;
    case KeyCodeValue.KpB:
      return <TextSubtitleCombination text="B" subtitle="Keypad" />;
    case KeyCodeValue.KpC:
      return <TextSubtitleCombination text="C" subtitle="Keypad" />;
    case KeyCodeValue.KpD:
      return <TextSubtitleCombination text="D" subtitle="Keypad" />;
    case KeyCodeValue.KpE:
      return <TextSubtitleCombination text="E" subtitle="Keypad" />;
    case KeyCodeValue.KpF:
      return <TextSubtitleCombination text="F" subtitle="Keypad" />;
    case KeyCodeValue.KpMemoryStore:
      return (
        <SmallTextSubtitleCombination text="Memory Store" subtitle="Keypad" />
      );
    case KeyCodeValue.KpMemoryRecall:
      return (
        <SmallTextSubtitleCombination text="Memory Recall" subtitle="Keypad" />
      );
    case KeyCodeValue.KpMemoryClear:
      return (
        <SmallTextSubtitleCombination text="Memory Clear" subtitle="Keypad" />
      );
    case KeyCodeValue.KpMemoryAdd:
      return (
        <SmallTextSubtitleCombination text="Memory Add" subtitle="Keypad" />
      );
    case KeyCodeValue.KpMemorySubtract:
      return (
        <SmallTextSubtitleCombination
          text="Memory Subtract"
          subtitle="Keypad"
        />
      );
    case KeyCodeValue.KpMemoryMultiply:
      return (
        <SmallTextSubtitleCombination
          text="Memory Multiply"
          subtitle="Keypad"
        />
      );
    case KeyCodeValue.KpMemoryDivide:
      return (
        <SmallTextSubtitleCombination text="Memory Divide" subtitle="Keypad" />
      );
    case KeyCodeValue.KpClear:
      return <SmallTextSubtitleCombination text="Clear" subtitle="Keypad" />;
    case KeyCodeValue.KpClearEntry:
      return (
        <SmallTextSubtitleCombination text="Clear Entry" subtitle="Keypad" />
      );
    case KeyCodeValue.KpBinary:
      return <SmallTextSubtitleCombination text="Binary" subtitle="Keypad" />;
    case KeyCodeValue.KpOctal:
      return <SmallTextSubtitleCombination text="Octal" subtitle="Keypad" />;
    case KeyCodeValue.KpDecimal:
      return <SmallTextSubtitleCombination text="Decimal" subtitle="Keypad" />;
    case KeyCodeValue.KpHexadecimal:
      return (
        <SmallTextSubtitleCombination text="Hexadecimal" subtitle="Keypad" />
      );
    case KeyCodeValue.KpSlash:
      return <TextSubtitleCombination text="/" subtitle="Keypad" />;
    case KeyCodeValue.KpAsterisk:
      return <TextSubtitleCombination text="*" subtitle="Keypad" />;
    case KeyCodeValue.KpMinus:
      return <TextSubtitleCombination text="-" subtitle="Keypad" />;
    case KeyCodeValue.KpPlus:
      return <TextSubtitleCombination text="+" subtitle="Keypad" />;
    case KeyCodeValue.KpDot:
      return <TextSubtitleCombination text="." subtitle="Keypad" />;
    case KeyCodeValue.KpEqual:
      return <TextSubtitleCombination text="=" subtitle="Keypad" />;
    case KeyCodeValue.KpComma:
      return <TextSubtitleCombination text="," subtitle="Keypad" />;
    case KeyCodeValue.KpEqualSign:
      return <TextSubtitleCombination text="=" subtitle="Keypad" />;
    case KeyCodeValue.KpThousandsSeparator:
      return (
        <SmallTextSubtitleCombination
          text="Thousands Separator"
          subtitle="Keypad"
        />
      );
    case KeyCodeValue.KpDecimalSeparator:
      return (
        <SmallTextSubtitleCombination
          text="Decimal Separator"
          subtitle="Keypad"
        />
      );
    case KeyCodeValue.KpCurrencyUnit:
      return (
        <SmallTextSubtitleCombination text="Currency Unit" subtitle="Keypad" />
      );
    case KeyCodeValue.KpCurrencySubunit:
      return (
        <SmallTextSubtitleCombination
          text="Currency Subunit"
          subtitle="Keypad"
        />
      );
    case KeyCodeValue.KpLeftParen:
      return <TextSubtitleCombination text="(" subtitle="Keypad" />;
    case KeyCodeValue.KpRightParen:
      return <TextSubtitleCombination text="(" subtitle="Keypad" />;
    case KeyCodeValue.KpLeftBrace:
      return <TextSubtitleCombination text="{" subtitle="Keypad" />;
    case KeyCodeValue.KpRightBrace:
      return <TextSubtitleCombination text="}" subtitle="Keypad" />;
    case KeyCodeValue.KpXor:
      return <SmallTextSubtitleCombination text="XOR" subtitle="Keypad" />;
    case KeyCodeValue.KpCaret:
      return <TextSubtitleCombination text="^" subtitle="Keypad" />;
    case KeyCodeValue.KpPercent:
      return <TextSubtitleCombination text="%" subtitle="Keypad" />;
    case KeyCodeValue.KpLessThan:
      return <TextSubtitleCombination text="<" subtitle="Keypad" />;
    case KeyCodeValue.KpGreaterThan:
      return <TextSubtitleCombination text=">" subtitle="Keypad" />;
    case KeyCodeValue.KpAmpersand:
      return <TextSubtitleCombination text={"&"} subtitle="Keypad" />;
    case KeyCodeValue.KpDoubleAmpersand:
      return <TextSubtitleCombination text="&&" subtitle="Keypad" />;
    case KeyCodeValue.KpPipe:
      return <TextSubtitleCombination text="|" subtitle="Keypad" />;
    case KeyCodeValue.KpDoublePipe:
      return <TextSubtitleCombination text="||" subtitle="Keypad" />;
    case KeyCodeValue.KpColon:
      return <TextSubtitleCombination text=":" subtitle="Keypad" />;
    case KeyCodeValue.KpHash:
      return <TextSubtitleCombination text="#" subtitle="Keypad" />;
    case KeyCodeValue.KpSpace:
      return <IconSubtitleCombination icon={<Space />} subtitle="Keypad" />;
    case KeyCodeValue.KpAt:
      return <TextSubtitleCombination text="@" subtitle="Keypad" />;
    case KeyCodeValue.KpExclamation:
      return <TextSubtitleCombination text="!" subtitle="Keypad" />;
    case KeyCodeValue.KpPlusMinus:
      return <TextSubtitleCombination text="+-" subtitle="Keypad" />;
    default:
      return <>{keyCodeValue.value}</>;
  }
}

const KeyCodeTooltip: Component<{ keyCode: KeyCode }> = (props) => {
  const { hostLayout } = useKeymapState();

  const content = (rawKeyCodes: RawKeyCode[]) => {
    return (
      <div class="flex flex-col">
        <div class="flex gap-1">
          <span>Key:</span>
          <span>{props.keyCode.value.value}</span>
        </div>
        <div class="flex gap-1">
          <CornerDownRight size="20" />
          <For each={rawKeyCodes} fallback={"Unknown"}>
            {(rawKeyCode, index) => (
              <>
                <Show when={index() !== 0}>
                  <span>+</span>
                </Show>
                <span>{rawKeyCode.name}</span>
                <span class="text-neutral-400 dark:text-neutral-600">
                  ({`0x${rawKeyCode.code.toString(16)}`})
                </span>
              </>
            )}
          </For>
        </div>
      </div>
    );
  };

  return (
    <Show when={hostLayout()} keyed>
      {(hostLayout) => {
        const keyCodes = keyCodeValues(hostLayout);
        const keyCodeEncoding = keyCodes.get(props.keyCode.value);

        if (!keyCodeEncoding) {
          return content([]);
        }

        if (keyCodeEncoding.kind === "single") {
          return content([keyCodeEncoding.rawKeyCode]);
        }

        if (keyCodeEncoding.kind === "combined") {
          return content([
            keyCodeEncoding.rawKeyCode1,
            keyCodeEncoding.rawKeyCode2,
          ]);
        }

        assertNever(keyCodeEncoding);
      }}
    </Show>
  );
};
