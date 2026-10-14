import { Progress } from '@ark-ui/solid/progress'
import { Component } from 'solid-js'
import styles from './LoadingIndicator.module.css'

export const LoadingIndicator: Component<{ label: string }> = props => (
  <Progress.Root
    class="flex w-full flex-col gap-2 text-neutral-950 dark:text-neutral-50"
    value={null}
  >
    <Progress.Label class="text-sm leading-5 font-medium text-neutral-950 dark:text-neutral-50">
      {props.label}
    </Progress.Label>
    <Progress.Track
      class="
        h-1.5 overflow-hidden rounded-full bg-neutral-300 dark:bg-neutral-800
        data-[orientation=vertical]:h-32 data-[orientation=vertical]:w-1.5
      "
    >
      <Progress.Range
        class={[
          'h-full rounded-full bg-accent transition-[width] duration-300 ease-out',
          'data-[orientation=vertical]:w-full',
          'data-[orientation=vertical]:transition-[height]',
          'data-[state=indeterminate]:w-1/2',
          styles.indeterminate_range,
        ].join(' ')}
      />
    </Progress.Track>
  </Progress.Root>
)

export default LoadingIndicator
