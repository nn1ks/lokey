import { Tooltip } from "@ark-ui/solid";
import type { KeyActionRender } from "../../keyAction";
import type { Component, JSX } from "solid-js";
import StyledTooltipContent from "../StyledTooltipContent";

export type KeyButtonProps = {
  keyAction: KeyActionRender;
  onClick?: JSX.EventHandler<HTMLButtonElement, MouseEvent>;
};

export const KeyButton: Component<KeyButtonProps> = (props) => {
  const selectedLayer = 0;
  const layerCount = 2;

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
        onClick={props.onClick}
      >
        {props.keyAction.render(selectedLayer, layerCount)}
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
