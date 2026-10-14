import type { Component } from 'solid-js'
import { Usb } from 'lucide-solid'
import { useConnection } from '../../contexts'
import type { SerialPortLike } from '../../contexts'
import StartButton from './StartButton'
import type { StartButtonProps } from './StartButton'

type SerialLike = {
  requestPort: () => Promise<SerialPortLike>
}

export type SelectDeviceButtonProps = Omit<StartButtonProps, 'icon' | 'label' | 'onClick'>

export const SelectDeviceButton: Component<SelectDeviceButtonProps> = props => {
  const { connect } = useConnection()

  const selectDevice = async () => {
    const serial = (navigator as Navigator & { serial?: SerialLike }).serial
    if (!serial) {
      // eslint-disable-next-line no-console
      console.error('Web Serial API is not available in this browser.')
      return
    }

    try {
      const port = await serial.requestPort()
      await connect(port)
    } catch (cause) {
      if (cause instanceof DOMException && cause.name === 'NotFoundError') {
        // eslint-disable-next-line no-console
        console.info('No Web Serial port was selected.')
      } else {
        // eslint-disable-next-line no-console
        console.error('Unable to select or open a Web Serial port.', cause)
      }
    }
  }

  return <StartButton {...props} icon={Usb} label="Select Device" onClick={selectDevice} />
}

export default SelectDeviceButton
