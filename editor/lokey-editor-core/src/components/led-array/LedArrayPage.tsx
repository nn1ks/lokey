import { Lightbulb } from 'lucide-solid'
import { createSignal } from 'solid-js'
import type { Component } from 'solid-js'
import PageHeading from '../PageHeading'
import ToggleRow from './ToggleRow'
import PoweredByLokey from '../PoweredByLokey'

export const LedArrayPage: Component = () => {
  const [boot, setBoot] = createSignal(false)
  const [bluetoothProfile, setBluetoothProfile] = createSignal(false)
  const [bluetoothPairing, setBluetoothPairing] = createSignal(false)
  const [layer, setLayer] = createSignal(false)

  return (
    <main class="w-full max-w-200 px-(--page-padding-x) mb-8">
      <PageHeading
        icon={<Lightbulb />}
        title="LED Array"
        description="Choose which events trigger LED animations"
        actions={<></>}
      />
      <div
        class="
            mt-2 flex flex-col
            [&>*:first-child]:rounded-t-2xl
            [&>*:last-child]:rounded-b-2xl
            [&>*:not(:last-child)]:border-b
            [&>*:not(:last-child)]:border-neutral-300
            dark:[&>*:not(:last-child)]:border-neutral-800
          "
      >
        <ToggleRow
          label="Boot"
          description="Shows a brief LED animation during device startup."
          enabled={boot()}
          onCheckedChange={setBoot}
        />
        <ToggleRow
          label="Layer Change"
          description="Briefly lights an LED whenever the currently active layer changes."
          enabled={layer()}
          onCheckedChange={setLayer}
        />
        <ToggleRow
          label="Bluetooth Profile"
          description="Briefly lights an LED whenever the active Bluetooth profile changes."
          enabled={bluetoothProfile()}
          onCheckedChange={setBluetoothProfile}
        />
        <ToggleRow
          label="Bluetooth Pairing"
          description="Shows an LED animation while the device is in pairing mode."
          enabled={bluetoothPairing()}
          onCheckedChange={setBluetoothPairing}
        />
      </div>
      <PoweredByLokey />
    </main>
  )
}

export default LedArrayPage
