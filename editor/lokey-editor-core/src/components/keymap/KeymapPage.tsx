import { Keyboard } from 'lucide-solid'
import { createSignal } from 'solid-js'
import type { Component, JSX } from 'solid-js'
import LayerSelectorBar from './LayerSelectorBar'
import Options from './Options'
import PageHeading from '../PageHeading'
import PoweredByLokey from '../PoweredByLokey'
import WithoutPagePadding from '../WithoutPagePadding'

export const KeymapPage: Component<{ children: JSX.Element; class?: string }> = props => {
  const [selectedLayer, setSelectedLayer] = createSignal(0)

  return (
    <main class={`w-full px-(--page-padding-x) mb-8 ${props.class ?? ''}`}>
      <PageHeading
        icon={<Keyboard />}
        title="Keymap"
        description="Configure key bindings and layers"
        actions={<></>}
      />
      <WithoutPagePadding>
        <LayerSelectorBar layerCount={3} selected={selectedLayer()} onSelect={setSelectedLayer} />
      </WithoutPagePadding>
      {props.children}
      <Options />
      <PoweredByLokey />
    </main>
  )
}

export default KeymapPage
