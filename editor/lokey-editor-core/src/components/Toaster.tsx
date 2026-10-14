import { Toast, Toaster as ArkToaster } from '@ark-ui/solid'
import { X } from 'lucide-solid'
import type { ToasterProps as ArkToasterProps } from '@ark-ui/solid'
import type { Component } from 'solid-js'

export type ToasterProps = Omit<ArkToasterProps, 'children'>

export const Toaster: Component<ToasterProps> = props => {
  return (
    <ArkToaster {...props}>
      {toast => (
        <Toast.Root>
          <Toast.Title>{toast().title}</Toast.Title>
          <Toast.Description>{toast().description}</Toast.Description>
          <Toast.CloseTrigger>
            <X />
          </Toast.CloseTrigger>
        </Toast.Root>
      )}
    </ArkToaster>
  )
}

export default Toaster
