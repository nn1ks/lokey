import { Dynamic } from 'solid-js/web'
import { splitProps } from 'solid-js'
import type { Component, JSX } from 'solid-js'

export type StartButtonProps = Omit<JSX.ButtonHTMLAttributes<HTMLButtonElement>, 'children'> & {
  icon: Component
  label: string
}

export const StartButton: Component<StartButtonProps> = props => {
  const [local, rest] = splitProps(props, ['icon', 'label', 'class'])

  return (
    <button
      {...rest}
      class={[
        'group flex w-full items-center gap-3 rounded-xl p-3 font-medium cursor-pointer',
        'bg-neutral-200 text-neutral-800 dark:bg-neutral-900 dark:text-neutral-200',
        'hover:bg-neutral-300 dark:hover:bg-neutral-800',
        'transition-[background-color] duration-50 ease-[ease]',
        'focus-visible:outline-2 focus-visible:outline-focused focus-visible:outline-offset-2',
        local.class ?? '',
      ]
        .filter(Boolean)
        .join(' ')}
    >
      <div>
        <Dynamic component={local.icon} size="22" />
      </div>
      <span>{local.label}</span>
    </button>
  )
}

export default StartButton
