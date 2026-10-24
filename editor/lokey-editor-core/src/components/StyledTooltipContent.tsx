import { Tooltip } from "@ark-ui/solid";
import { splitProps } from "solid-js";
import type { Component } from "solid-js";

export type StyledTooltipContentProps = Tooltip.ContentProps & {
  withArrow?: boolean;
};

export const StyledTooltipContent: Component<StyledTooltipContentProps> = (
  props,
) => {
  const [local, contentProps] = splitProps(props, [
    "children",
    "class",
    "withArrow",
  ]);
  const className = [
    "relative z-1000 rounded-lg px-2.5 py-1.5",
    "bg-neutral-800 text-neutral-50 dark:bg-neutral-200 dark:text-neutral-950",
    "[--arrow-size:10px] [--arrow-background:var(--color-neutral-800)]",
    "dark:[--arrow-background:var(--color-neutral-200)]",
    local.class,
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <Tooltip.Content {...contentProps} class={className}>
      {local.withArrow !== false && (
        <Tooltip.Arrow>
          <Tooltip.ArrowTip />
        </Tooltip.Arrow>
      )}
      {local.children}
    </Tooltip.Content>
  );
};

export default StyledTooltipContent;
