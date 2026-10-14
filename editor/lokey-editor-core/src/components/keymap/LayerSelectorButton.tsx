import { Tooltip } from '@ark-ui/solid'
import { X } from 'lucide-solid'
import type { Component } from 'solid-js'
import StyledTooltipContent from '../StyledTooltipContent'

export type LayerSelectorButtonProps = {
  /** Index of the layer represented by this component. */
  layerIndex: number

  /** Index of the currently selected layer. */
  selected?: number

  /** Called with the layer index when its label is clicked. */
  onSelect?: (index: number) => void

  /** Called with the layer index when its remove button is clicked. */
  onRemove?: (index: number) => void
}

export const LayerSelectorButton: Component<LayerSelectorButtonProps> = props => {
  const label = () => `Layer ${props.layerIndex + 1}`
  const isSelected = () => props.selected === props.layerIndex

  return (
    <div
      class="
        inline-flex flex-none items-center rounded-full h-8
        bg-neutral-300 text-neutral-800 hover:bg-neutral-400
        dark:bg-neutral-800 dark:text-neutral-300 dark:hover:bg-neutral-700
        data-[selected=true]:bg-neutral-700 data-[selected=true]:text-neutral-50
        data-[selected=true]:hover:bg-neutral-700
        dark:data-[selected=true]:bg-neutral-300 dark:data-[selected=true]:text-neutral-950
        dark:data-[selected=true]:hover:bg-neutral-300
      "
      data-selected={isSelected()}
      role="listitem"
    >
      <button
        class="
          h-full px-3 text-inherit cursor-pointer
          data-[selected=true]:pr-1
          focus-visible:outline-2 focus-visible:outline-focused
          focus-visible:-outline-offset-2
        "
        data-selected={isSelected()}
        type="button"
        aria-pressed={isSelected()}
        onClick={() => props.onSelect?.(props.layerIndex)}
      >
        {label()}
      </button>
      {isSelected() && (
        <Tooltip.Root positioning={{ placement: 'bottom' }}>
          <Tooltip.Trigger
            class="
              mr-[0.2rem] inline-flex items-center justify-center rounded-full
              border-0 bg-transparent p-[0.325rem] cursor-pointer
              text-neutral-50 dark:text-neutral-950
              hover:bg-neutral-600 hover:text-neutral-50
              dark:hover:bg-neutral-400 dark:hover:text-neutral-950
              transition-[background-color] duration-50 ease-[ease]

              focus-visible:outline-2 focus-visible:outline-focused
              focus-visible:-outline-offset-2
            "
            type="button"
            aria-label={`Remove ${label()}`}
            onClick={event => {
              event.stopPropagation()
              props.onRemove?.(props.layerIndex)
            }}
          >
            <X size={14} strokeWidth={2} />
          </Tooltip.Trigger>
          <Tooltip.Positioner>
            <StyledTooltipContent>Remove layer</StyledTooltipContent>
          </Tooltip.Positioner>
        </Tooltip.Root>
      )}
    </div>
  )
}

export default LayerSelectorButton
