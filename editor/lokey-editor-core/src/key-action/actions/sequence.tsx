import * as capnp from "capnp-es";
import { Ellipsis } from "lucide-solid";
import { SequenceAction } from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import type { KeyAction } from "../action";
import { buildAction, buildActionEntry } from "../codec/entries";
import { defineAction } from "../definition";
import { Labelled, Split } from "../render";
import { KeyCode, keyCodesForLayout, reverseKeyCodes } from "./key-code";

export class Sequence<
  A1 extends KeyAction = KeyAction,
  A2 extends KeyAction = KeyAction,
> implements KeyAction {
  readonly actionId = ActionId.sequence;

  constructor(
    readonly action1: A1,
    readonly action2: A2,
  ) {}
}

export const sequenceActionDefinition = defineAction<Sequence>({
  actionId: ActionId.sequence,
  label: "Sequence",
  encode: (action, ctx) => {
    const action1EntryId = ctx.encodeChild(action.action1);
    const action2EntryId = ctx.encodeChild(action.action2);

    return {
      rootActionEntry: buildActionEntry(
        ctx.allocateEntryId(),
        buildAction(ActionId.sequence, SequenceAction, (sequenceAction) => {
          sequenceAction.action1EntryId = action1EntryId;
          sequenceAction.action2EntryId = action2EntryId;
        }),
      ),
      childActionEntries: [],
    };
  },
  decode: (payload, ctx) => {
    const message = new capnp.Message(payload, false);
    const sequenceAction = message.getRoot(SequenceAction);
    const action1 = ctx.decodeChild(sequenceAction.action1EntryId);
    const action2 = ctx.decodeChild(sequenceAction.action2EntryId);

    // A sequence that encodes a single character (e.g. German "=" as
    // Shift+0) collapses back into that key code.
    if (action1 instanceof KeyCode && action2 instanceof KeyCode) {
      const encoding1 = keyCodesForLayout(ctx.layout).get(action1);
      const encoding2 = keyCodesForLayout(ctx.layout).get(action2);
      if (encoding1?.kind === "simple" && encoding2?.kind === "simple") {
        const keyCode = reverseKeyCodes(ctx.layout).sequence.get(
          `${encoding1.keyCode}:${encoding2.keyCode}`,
        );
        if (keyCode) return keyCode;
      }
    }

    return new Sequence(action1, action2);
  },
  render: (action, level, ctx) => {
    if (level > 0) return <Ellipsis />;

    return (
      <Split
        action1={
          <Labelled label="1">
            {ctx.renderAction(action.action1, level + 1)}
          </Labelled>
        }
        action2={
          <Labelled label="2">
            {ctx.renderAction(action.action2, level + 1)}
          </Labelled>
        }
      />
    );
  },
  renderTooltip: (action, ctx) => (
    <div class="flex items-stretch gap-2">
      <div class="flex min-w-12 flex-col items-center gap-1">
        <div class="text-neutral-500 text-[0.65rem] font-semibold tracking-[0.08em] uppercase">
          1
        </div>
        <div class="flex min-h-6 items-center justify-center">
          {ctx.renderTooltip(action.action1)}
        </div>
      </div>
      <div class="w-px bg-neutral-600 dark:bg-neutral-400" />
      <div class="flex min-w-12 flex-col items-center gap-1">
        <div class="text-neutral-500 text-[0.65rem] font-semibold tracking-[0.08em] uppercase">
          2
        </div>
        <div class="flex min-h-6 items-center justify-center">
          {ctx.renderTooltip(action.action2)}
        </div>
      </div>
    </div>
  ),
  remap: (action, ctx) =>
    new Sequence(
      ctx.remapAction(action.action1),
      ctx.remapAction(action.action2),
    ),
  children: (action) => [action.action1, action.action2],
});
