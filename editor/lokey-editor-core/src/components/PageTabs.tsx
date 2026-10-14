import { Tabs } from '@ark-ui/solid'

import type { Component, JSX } from 'solid-js'
import styles from './PageTabs.module.css'

export const PageTabs: Component<Tabs.RootProps> = props => {
  return <Tabs.Root {...props} class={styles.root} />
}

export type PageTabTriggerContentProps = {
  icon: JSX.Element
  title: JSX.Element
}

export const PageTabTriggerContent: Component<PageTabTriggerContentProps> = props => {
  return (
    <div class="flex items-center gap-1.5 text-[0.95em] mt-[1px] [&_svg]:size-4.5">
      {props.icon}
      <span>{props.title}</span>
    </div>
  )
}

export { Tabs }
export default PageTabs
