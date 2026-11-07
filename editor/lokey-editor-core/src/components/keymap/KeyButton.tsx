import { Tooltip } from "@ark-ui/solid";
import type { KeyAction } from "../../key-action";
import type { Component } from "solid-js";
import StyledTooltipContent from "../StyledTooltipContent";

export type KeyButtonProps = {
  keyAction: KeyAction;
  onClick?: () => void;
};

export const KeyButton: Component<KeyButtonProps> = (props) => {
  return (
    <Tooltip.Root positioning={{ placement: "bottom", flip: false }}>
      <Tooltip.Trigger
        class="absolute flex items-center justify-center
          w-[64px] h-[60px] rounded-lg
          bg-neutral-300 hover:bg-neutral-350
          dark:bg-neutral-750 dark:hover:bg-neutral-700
          transition-colors duration-50
          text-neutral-900 dark:text-neutral-100
          text-[16px] font-medium
          cursor-pointer select-none
          [&_svg]:size-[1.4em] [&_svg]:stroke-[1.8]"
        type="button"
        aria-label="Key action"
        onClick={() => props.onClick?.()}
      >
        {props.keyAction.renderButtonContent(0)}
      </Tooltip.Trigger>
      <Tooltip.Positioner>
        <StyledTooltipContent>
          {props.keyAction.renderTooltip()}
        </StyledTooltipContent>
      </Tooltip.Positioner>
    </Tooltip.Root>
  );
};

export default KeyButton;
