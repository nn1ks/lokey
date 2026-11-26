use crate::odyn::{HookStateBitmaskEntry, OdynHookBundle, TupleHookBundle, TupleSize};
use crate::pwm::PwmChannel;
use crate::{LedArrayMessageHandler, default_gamma_correction};
use embassy_futures::join::join3;
use lokey::external::UnsupportedMessageType;
use lokey::storage::{ReadTransaction, Storage, WriteTransaction};
use lokey::util::error;
use lokey::{AnyState, Component, Context, Device};
use lokey_led_array_odyn::protocol;
use lokey_odyn::MAX_PAYLOAD_SIZE;

pub struct OdynLedArray<const NUM_LEDS: usize, Hooks: OdynHookBundle> {
    gamma_correction: fn(f32) -> f32,
    hook_bundle: Hooks,
}

impl<const NUM_LEDS: usize, Hooks: OdynHookBundle> Component for OdynLedArray<NUM_LEDS, Hooks> {}

impl<const NUM_LEDS: usize, Hooks> OdynLedArray<NUM_LEDS, TupleHookBundle<Hooks>>
where
    Hooks: TupleSize,
    TupleHookBundle<Hooks>: OdynHookBundle,
{
    /// Creates a new [`OdynLedArray`] instance from a tuple of hooks.
    ///
    /// # Example
    ///
    /// ```
    /// use lokey_led_array::{BootHook, BleProfileHook, OdynLedArray};
    ///
    /// let odyn_led_array = OdynLedArray::<4, _>::new((BootHook, BleProfileHook));
    /// ```
    pub fn new<T>(hook_bundle: T) -> Self
    where
        T: Into<TupleHookBundle<Hooks>>,
    {
        Self {
            gamma_correction: default_gamma_correction,
            hook_bundle: hook_bundle.into(),
        }
    }
}

impl<const NUM_LEDS: usize, Hooks: OdynHookBundle> OdynLedArray<NUM_LEDS, Hooks> {
    pub fn with_custom(hook_bundle: Hooks) -> Self {
        Self {
            gamma_correction: default_gamma_correction,
            hook_bundle,
        }
    }

    pub const fn gamma_correction(mut self, f: fn(f32) -> f32) -> Self {
        self.gamma_correction = f;
        self
    }

    async fn handle_get_led_array_hook_capabilities_request<T>(
        &self,
        message: &lokey_odyn::Message,
        external_channel: &lokey::external::Channel<T>,
    ) where
        T: lokey::external::Transport,
    {
        let mut buf = [0u8; MAX_PAYLOAD_SIZE];
        let result = lokey_odyn::decode::<protocol::get_led_array_hook_capabilities_request::Owned>(
            &message.payload,
            &mut buf,
        );
        let reader = match result {
            Ok(v) => v,
            Err(e) => {
                #[cfg(feature = "defmt")]
                let e = defmt::Display2Format(&e);
                error!("Failed to decode message: {}", e);
                // TODO: send error message
                return;
            }
        };
        match reader.get() {
            Ok(v) => v,
            Err(e) => {
                #[cfg(feature = "defmt")]
                let e = defmt::Display2Format(&e);
                error!("Failed to decode message: {}", e);
                // TODO: send error message
                return;
            }
        };
        let bitmask = self.hook_bundle.available_hooks();
        let response_result = lokey_odyn::encode::<
            protocol::get_led_array_hook_capabilities_response::Owned,
            MAX_PAYLOAD_SIZE,
        >(|v| {
            v.set_hooks(bitmask.value);
        });
        let response = match response_result {
            Ok(v) => v,
            Err(e) => {
                #[cfg(feature = "defmt")]
                let e = defmt::Display2Format(&e);
                error!("Failed to encode response: {}", e);
                return;
            }
        };
        let response_message = lokey_odyn::Message {
            sequence_number: message.sequence_number,
            id: protocol::message_id::GET_LED_ARRAY_HOOK_CAPABILITIES_RESPONSE,
            payload: response,
        };
        match external_channel.try_send(response_message).await {
            Ok(()) => {}
            Err(UnsupportedMessageType) => {
                error!("External transport does not support sending odyn message");
            }
        };
    }

    async fn handle_get_led_array_hooks_request<T>(
        &self,
        message: &lokey_odyn::Message,
        external_channel: &lokey::external::Channel<T>,
    ) where
        T: lokey::external::Transport,
    {
        let mut buf = [0u8; MAX_PAYLOAD_SIZE];
        let result = lokey_odyn::decode::<protocol::get_led_array_hooks_request::Owned>(
            &message.payload,
            &mut buf,
        );
        let reader = match result {
            Ok(v) => v,
            Err(e) => {
                #[cfg(feature = "defmt")]
                let e = defmt::Display2Format(&e);
                error!("Failed to decode message: {}", e);
                // TODO: send error message
                return;
            }
        };
        match reader.get() {
            Ok(v) => v,
            Err(e) => {
                #[cfg(feature = "defmt")]
                let e = defmt::Display2Format(&e);
                error!("Failed to decode message: {}", e);
                // TODO: send error message
                return;
            }
        };
        let bitmask = self.hook_bundle.enabled_hooks();
        let response_result = lokey_odyn::encode::<
            protocol::get_led_array_hooks_response::Owned,
            MAX_PAYLOAD_SIZE,
        >(|v| {
            v.set_hooks(bitmask.value);
        });
        let response = match response_result {
            Ok(v) => v,
            Err(e) => {
                #[cfg(feature = "defmt")]
                let e = defmt::Display2Format(&e);
                error!("Failed to encode response: {}", e);
                return;
            }
        };
        let response_message = lokey_odyn::Message {
            sequence_number: message.sequence_number,
            id: protocol::message_id::GET_LED_ARRAY_HOOKS_RESPONSE,
            payload: response,
        };
        match external_channel.try_send(response_message).await {
            Ok(()) => {}
            Err(UnsupportedMessageType) => {
                error!("External transport does not support sending odyn message");
            }
        };
    }

    async fn handle_set_led_array_hooks_request<T>(
        &self,
        message: &lokey_odyn::Message,
        external_channel: &lokey::external::Channel<T>,
    ) where
        T: lokey::external::Transport,
    {
        let mut buf = [0u8; MAX_PAYLOAD_SIZE];
        let result = lokey_odyn::decode::<protocol::set_led_array_hooks_request::Owned>(
            &message.payload,
            &mut buf,
        );
        let reader = match result {
            Ok(v) => v,
            Err(e) => {
                #[cfg(feature = "defmt")]
                let e = defmt::Display2Format(&e);
                error!("Failed to decode message: {}", e);
                // TODO: send error message
                return;
            }
        };
        let reader = match reader.get() {
            Ok(v) => v,
            Err(e) => {
                #[cfg(feature = "defmt")]
                let e = defmt::Display2Format(&e);
                error!("Failed to decode message: {}", e);
                // TODO: send error message
                return;
            }
        };
        let hooks_bitmask = reader.get_hooks();
        self.hook_bundle.set_enabled_hooks(hooks_bitmask.into());
        let response = match lokey_odyn::encode::<
            lokey_odyn::protocol::success_response::Owned,
            MAX_PAYLOAD_SIZE,
        >(|_| {})
        {
            Ok(v) => v,
            Err(e) => {
                #[cfg(feature = "defmt")]
                let e = defmt::Display2Format(&e);
                error!("Failed to encode response: {}", e);
                return;
            }
        };
        let response_message = lokey_odyn::Message {
            sequence_number: message.sequence_number,
            id: lokey_odyn::protocol::message_id::SUCCESS_RESPONSE,
            payload: response,
        };
        match external_channel.try_send(response_message).await {
            Ok(()) => {}
            Err(UnsupportedMessageType) => {
                error!("External transport does not support sending odyn message");
            }
        };
    }

    pub async fn run<D, S>(
        self,
        context: Context<D, S>,
        pwm_channels: [&mut dyn PwmChannel; NUM_LEDS],
    ) where
        D: Device,
        S: AnyState,
    {
        let run_hooks = self.hook_bundle.run_all::<NUM_LEDS>(context.as_dyn());

        let mut message_handler =
            LedArrayMessageHandler::new(context.as_dyn(), self.gamma_correction);

        let handle_odyn_messages = async {
            let mut odyn_receiver = match context
                .external_channel
                .try_receiver::<lokey_odyn::Message>()
            {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to create receiver: {}", e);
                    return;
                }
            };
            loop {
                let message = odyn_receiver.next().await;
                match message.id {
                    protocol::message_id::GET_LED_ARRAY_HOOK_CAPABILITIES_REQUEST => {
                        self.handle_get_led_array_hook_capabilities_request(
                            &message,
                            context.external_channel,
                        )
                        .await
                    }
                    protocol::message_id::GET_LED_ARRAY_HOOKS_REQUEST => {
                        self.handle_get_led_array_hooks_request(&message, context.external_channel)
                            .await
                    }
                    protocol::message_id::SET_LED_ARRAY_HOOKS_REQUEST => {
                        self.handle_set_led_array_hooks_request(&message, context.external_channel)
                            .await;
                        let bitmask = self.hook_bundle.enabled_hooks();
                        let mut write_transaction = context.storage.write_transaction().await;
                        if let Err(e) = write_transaction
                            .write((), &HookStateBitmaskEntry(bitmask.value))
                            .await
                        {
                            #[cfg(feature = "defmt")]
                            let e = defmt::Debug2Format(&e);
                            error!("Failed to write hook state to storage: {}", e);
                        };
                        if let Err(e) = write_transaction.commit().await {
                            #[cfg(feature = "defmt")]
                            let e = defmt::Debug2Format(&e);
                            error!("Failed to commit to storage: {}", e);
                        };
                    }
                    _ => {}
                }
            }
        };

        {
            let read_transaction = context.storage.read_transaction().await;
            match read_transaction.read::<HookStateBitmaskEntry>(()).await {
                Ok(bitmask) => {
                    self.hook_bundle.set_enabled_hooks(bitmask.0.into());
                }
                Err(e) => {
                    #[cfg(feature = "defmt")]
                    let e = defmt::Debug2Format(&e);
                    error!("Failed to read hook state from storage: {}", e);
                }
            };
        }

        join3(
            message_handler.run(pwm_channels),
            run_hooks,
            handle_odyn_messages,
        )
        .await;
    }
}
