import { splitProps } from 'solid-js'
import type { Component, JSX } from 'solid-js'

export type ButtonProps = JSX.ButtonHTMLAttributes<HTMLButtonElement>

export const Button: Component<ButtonProps> = props => {
  const [local, rest] = splitProps(props, ['children', 'class'])

  return (
    <button
      {...rest}
      class={[
        'inline-flex min-h-9 items-center justify-center px-4 py-2 rounded-lg cursor-pointer',
        'bg-neutral-200 text-neutral-800 dark:bg-neutral-900 dark:text-neutral-200',
        'enabled:hover:bg-neutral-300 dark:enabled:hover:bg-neutral-800',
        'transition-[background-color] duration-50 ease-[ease]',
        'focus-visible:outline-2 focus-visible:outline-focused focus-visible:outline-offset-2',
        'disabled:cursor-not-allowed disabled:opacity-50',
        local.class ?? '',
      ]
        .filter(Boolean)
        .join(' ')}
    >
      {local.children}
    </button>
  )
}

export default Button
