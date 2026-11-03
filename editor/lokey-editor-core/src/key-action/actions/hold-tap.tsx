import * as capnp from "capnp-es";
import { Ellipsis } from "lucide-solid";
import { HoldTapAction } from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import type { KeyAction } from "../action";
import { buildAction, buildActionEntry } from "../codec/entries";
import { defineAction } from "../definition";
import { Labelled, Split } from "../render";

export class HoldTap<
  H extends KeyAction = KeyAction,
  T extends KeyAction = KeyAction,
> implements KeyAction {
  readonly actionId = ActionId.holdTap;

  constructor(
    readonly holdAction: H,
    readonly tapAction: T,
    /** Tapping term in milliseconds. */
    readonly tappingTerm: number = 200,
  ) {}
}

export const holdTapActionDefinition = defineAction<HoldTap>({
  actionId: ActionId.holdTap,
  label: "Hold/Tap",
  encode: (action, ctx) => {
    const holdActionEntryId = ctx.encodeChild(action.holdAction);
    const tapActionEntryId = ctx.encodeChild(action.tapAction);

    return {
      rootActionEntry: buildActionEntry(
        ctx.allocateEntryId(),
        buildAction(ActionId.holdTap, HoldTapAction, (holdTapAction) => {
          holdTapAction.holdActionEntryId = holdActionEntryId;
          holdTapAction.tapActionEntryId = tapActionEntryId;
          holdTapAction.tappingTerm = action.tappingTerm;
        }),
      ),
      childActionEntries: [],
    };
  },
  decode: (payload, ctx) => {
    const message = new capnp.Message(payload, false);
    const holdTap = message.getRoot(HoldTapAction);
    return new HoldTap(
      ctx.decodeChild(holdTap.holdActionEntryId),
      ctx.decodeChild(holdTap.tapActionEntryId),
      holdTap.tappingTerm,
    );
  },
  render: (action, level, ctx) => {
    if (level > 0) return <Ellipsis />;

    return (
      <Split
        action1={
          <Labelled label="Tap">
            {ctx.renderAction(action.tapAction, level + 1)}
          </Labelled>
        }
        action2={
          <Labelled label="Hold">
            {ctx.renderAction(action.holdAction, level + 1)}
          </Labelled>
        }
      />
    );
  },
  renderTooltip: (action, ctx) => (
    <div class="flex items-stretch gap-2">
      <div class="flex min-w-12 flex-col items-center gap-1">
        <div class="text-neutral-500 text-[0.65rem] font-semibold tracking-[0.08em] uppercase">
          Tap
        </div>
        <div class="flex min-h-6 items-center justify-center">
          {ctx.renderTooltip(action.tapAction)}
        </div>
      </div>
      <div class="w-px bg-neutral-600 dark:bg-neutral-400" />
      <div class="flex min-w-12 flex-col items-center gap-1">
        <div class="text-neutral-500 text-[0.65rem] font-semibold tracking-[0.08em] uppercase">
          Hold
        </div>
        <div class="flex min-h-6 items-center justify-center">
          {ctx.renderTooltip(action.holdAction)}
        </div>
      </div>
    </div>
  ),
  remap: (action, ctx) =>
    new HoldTap(
      ctx.remapAction(action.holdAction),
      ctx.remapAction(action.tapAction),
      action.tappingTerm,
    ),
  children: (action) => [action.holdAction, action.tapAction],
});
