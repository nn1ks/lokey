import { Accordion } from "@ark-ui/solid";
import * as capnp from "capnp-es";
import { Ellipsis } from "lucide-solid";
import { createSignal } from "solid-js";
import type { Component } from "solid-js";
import { StickyAction } from "lokey-editor-odyn/keyboard";
import * as ActionId from "lokey-editor-odyn/keyboard/action-id";
import { ActionPickerChildRow } from "../../components/keymap/ActionPickerChildRow";
import { ActionPickerDurationRow } from "../../components/keymap/ActionPickerDurationRow";
import { ActionPickerHeader } from "../../components/keymap/ActionPickerHeader";
import { ActionPickerSwitchRow } from "../../components/keymap/ActionPickerSwitchRow";
import type { KeyAction } from "../action";
import { buildAction, buildActionEntry } from "../codec/entries";
import { defineAction } from "../definition";
import type { PickerContext } from "../definition";
import { Labelled } from "../render";

export class Sticky<A extends KeyAction = KeyAction> implements KeyAction {
  readonly actionId = ActionId.sticky;

  constructor(
    readonly action: A,
    /** Timeout in milliseconds. */
    readonly timeout: number = 1000,
    readonly ignoreModifiers: boolean = true,
  ) {}
}

type StickyChanges = Partial<{
  action: KeyAction;
  timeout: number;
  ignoreModifiers: boolean;
}>;

const withSticky = (action: Sticky, changes: StickyChanges): Sticky =>
  new Sticky(
    changes.action ?? action.action,
    changes.timeout ?? action.timeout,
    changes.ignoreModifiers ?? action.ignoreModifiers,
  );

const withStickyChildren = (action: Sticky, children: KeyAction[]): Sticky =>
  withSticky(action, { action: children[0]! });

const StickyPicker: Component<{
  action: Sticky;
  ctx: PickerContext;
}> = (props) => {
  const [currentAction, setCurrentAction] = createSignal(props.action);

  const update = (changes: StickyChanges) => {
    const next = withSticky(currentAction(), changes);
    setCurrentAction(next);
    props.ctx.onSelect(next, false);
  };

  return (
    <>
      <ActionPickerHeader
        label="Sticky"
        withAccordionTrigger={true}
        onSelect={() => props.ctx.onSelect(currentAction(), true)}
      />
      <Accordion.ItemContent class="data-[state=closed]:hidden">
        <div class="flex flex-col gap-2 p-2 pt-0">
          <ActionPickerDurationRow
            label="Timeout"
            value={currentAction().timeout}
            min={0}
            max={10000}
            step={50}
            unit="ms"
            onValueCommit={() => props.ctx.onSelect(currentAction(), false)}
            onValueChange={(timeout) => {
              setCurrentAction(withSticky(currentAction(), { timeout }));
            }}
          />
          <ActionPickerSwitchRow
            label="Ignore Modifiers"
            checked={currentAction().ignoreModifiers}
            onCheckedChange={(ignoreModifiers) => update({ ignoreModifiers })}
          />
          <ActionPickerChildRow
            label="Action"
            action={currentAction().action}
            onSelect={(action) => update({ action })}
          />
        </div>
      </Accordion.ItemContent>
    </>
  );
};

export const stickyActionDefinition = defineAction<Sticky>({
  actionId: ActionId.sticky,
  label: "Sticky",
  encode: (action, ctx) => {
    const actionEntryId = ctx.encodeChild(action.action);

    return {
      rootActionEntry: buildActionEntry(
        ctx.allocateEntryId(),
        buildAction(ActionId.sticky, StickyAction, (stickyAction) => {
          stickyAction.actionEntryId = actionEntryId;
          stickyAction.timeout = action.timeout;
          stickyAction.ignoreModifiers = action.ignoreModifiers;
        }),
      ),
      childActionEntries: [],
    };
  },
  decode: (payload, ctx) => {
    const message = new capnp.Message(payload, false);
    const sticky = message.getRoot(StickyAction);
    return new Sticky(
      ctx.decodeChild(sticky.actionEntryId),
      sticky.timeout,
      sticky.ignoreModifiers,
    );
  },
  render: (action, level, ctx) => {
    if (level > 0) return <Ellipsis />;

    return (
      <Labelled label="Sticky">
        {ctx.renderAction(action.action, level + 1)}
      </Labelled>
    );
  },
  renderTooltip: (action, ctx) => (
    <div class="flex flex-col items-center gap-1">
      <div class="text-neutral-500 text-[0.65rem] font-semibold tracking-[0.08em] uppercase">
        Sticky
      </div>
      <div class="flex min-h-6 items-center justify-center">
        {ctx.renderTooltip(action.action)}
      </div>
    </div>
  ),
  remap: (action, ctx) =>
    withSticky(action, { action: ctx.remapAction(action.action) }),
  children: (action) => [action.action],
  renderPicker: (action, ctx) => <StickyPicker action={action} ctx={ctx} />,
  withChildren: withStickyChildren,
});
