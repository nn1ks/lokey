mod input;
mod output;

// Ideally this would only be included if running doc tests, but this is currently not possible (see
// https://github.com/rust-lang/rust/issues/67295).
#[doc(hidden)]
pub mod mock;

use core::cell::RefCell;
use core::marker::PhantomData;

/// Represents an input switch, such as a button or a switch
pub trait InputSwitch {
    type Error;

    /// Returns true if the switch has been activated, otherwise false
    /// i.e. if a button is currently pressed, returns true
    ///
    /// # Examples
    ///
    /// ```
    /// # use lokey_keyboard::switch::mock;
    /// use lokey_keyboard::switch::{InputSwitch, OutputSwitch, Switch, IntoSwitch};
    /// # let pin = mock::Pin::with_state(mock::State::High);
    /// # let mut status_led = mock::Pin::new().into_active_high_switch();
    /// let mut button = pin.into_active_low_switch();
    /// match button.is_active() {
    ///     Ok(true) => { status_led.on().ok(); }
    ///     Ok(false) => { status_led.off().ok(); }
    ///     Err(_) => { panic!("Failed to read button state"); }
    /// }
    /// ```
    fn is_active(&self) -> Result<bool, Self::Error>;
}

/// Represents an input switch that can be asynchronously waited for
pub trait WaitableInputSwitch {
    type Error;

    /// Waits until the switch becomes active. If the switch in already active, returns immediately
    fn wait_for_active(&mut self) -> impl Future<Output = Result<(), Self::Error>>;

    /// Waits until the switch becomes inactive. If the switch is already inactive, returns immediately
    fn wait_for_inactive(&mut self) -> impl Future<Output = Result<(), Self::Error>>;

    /// Waits until the switch changess from active to inactive, or from inactive to active
    fn wait_for_change(&mut self) -> impl Future<Output = Result<(), Self::Error>>;
}

/// Represents an output switch, such as a LED "switch" or transistor
pub trait OutputSwitch {
    type Error;

    /// Turns the switch on
    ///
    /// # Examples
    ///
    /// ```
    /// # use lokey_keyboard::switch::mock;
    /// use lokey_keyboard::switch::{OutputSwitch, Switch, IntoSwitch};
    /// # let pin = mock::Pin::new();
    /// let mut led = pin.into_active_high_switch();
    /// led.on().ok();
    /// ```
    fn on(&mut self) -> Result<(), Self::Error>;

    /// Turns the switch off
    ///
    /// # Examples
    ///
    /// ```
    /// # use lokey_keyboard::switch::mock;
    /// use lokey_keyboard::switch::{OutputSwitch, Switch, IntoSwitch};
    /// # let pin = mock::Pin::new();
    /// let mut led = pin.into_active_high_switch();
    /// led.off().ok();
    /// ```
    fn off(&mut self) -> Result<(), Self::Error>;
}

/// Toggles the switch from it's current state to it's opposite state.
///
/// # Notes
///
/// This is only available if the underlying hal has implemented
/// [`StatefulOutputPin`](embedded_hal::digital::StatefulOutputPin)
pub trait ToggleableOutputSwitch {
    type Error;

    /// Toggles the current state of the [`OutputSwitch`]
    ///
    /// # Examples
    ///
    /// ```
    /// # use lokey_keyboard::switch::mock;
    /// use lokey_keyboard::switch::{OutputSwitch, ToggleableOutputSwitch, Switch, IntoSwitch};
    /// # let pin = mock::Pin::new();
    /// let mut led = pin.into_active_high_switch();
    /// led.toggle().ok();
    /// ```
    fn toggle(&mut self) -> Result<(), Self::Error>;
}

/// Checks current switch state
///
/// # Notes
///
/// This is only available if the underlying hal has implemented
/// [`StatefulOutputPin`](embedded_hal::digital::StatefulOutputPin)
pub trait StatefulOutputSwitch {
    type Error;

    /// Checks whether the switch is on
    ///
    /// # Examples
    ///
    /// ```
    /// # use lokey_keyboard::switch::mock;
    /// use lokey_keyboard::switch::{OutputSwitch, Switch, StatefulOutputSwitch, IntoSwitch};
    /// # let pin = mock::Pin::new();
    /// let mut led = pin.into_active_high_switch();
    /// led.off().ok();
    /// assert_eq!(false, led.is_on().unwrap());
    /// ```
    fn is_on(&mut self) -> Result<bool, Self::Error>;

    /// Checks whether the switch is off
    ///
    /// # Examples
    ///
    /// ```
    /// # use lokey_keyboard::switch::mock;
    /// use lokey_keyboard::switch::{OutputSwitch, Switch, StatefulOutputSwitch, IntoSwitch};
    /// # let pin = mock::Pin::new();
    /// let mut led = pin.into_active_high_switch();
    /// led.off().ok();
    /// assert_eq!(true, led.is_off().unwrap());
    /// ```
    fn is_off(&mut self) -> Result<bool, Self::Error>;
}

/// Concrete implementation for [`InputSwitch`] and [`OutputSwitch`]
///
/// # Type Params
/// - `IoPin` must be a type that implements either of the
///   [`InputPin`](embedded_hal::digital::InputPin) or
///   [`OutputPin`](embedded_hal::digital::OutputPin) traits.
/// - `ActiveLevel` indicates whether the `Switch` is [`ActiveHigh`] or [`ActiveLow`].
///   `ActiveLevel` is not actually stored in the struct.
///   It's [`PhantomData`] used to indicate which implementation to use.
pub struct Switch<IoPin, ActiveLevel> {
    pin: RefCell<IoPin>,
    active: PhantomData<ActiveLevel>,
}

impl<IoPin, ActiveLevel> Switch<IoPin, ActiveLevel> {
    /// Constructs a new [`Switch`] from a concrete implementation of an
    /// [`InputPin`](embedded_hal::digital::InputPin) or
    /// [`OutputPin`](embedded_hal::digital::OutputPin)
    ///
    /// **Prefer the [`IntoSwitch`] trait over calling [`new`](Self::new) directly.**
    ///
    /// # Examples
    ///
    /// Active High
    ///
    /// ```
    /// # use lokey_keyboard::switch::mock;
    /// use lokey_keyboard::switch::{ActiveHigh, OutputSwitch, Switch};
    /// # let pin = mock::Pin::new();
    /// let mut led = Switch::<_, ActiveHigh>::new(pin);
    /// ```
    ///
    /// ActiveLow
    ///
    /// ```
    /// # use lokey_keyboard::switch::mock;
    /// use lokey_keyboard::switch::{ActiveLow, OutputSwitch, Switch};
    /// # let pin = mock::Pin::new();
    /// let mut led = Switch::<_, ActiveLow>::new(pin);
    /// ```
    pub fn new(pin: IoPin) -> Self {
        Switch {
            pin: RefCell::new(pin),
            active: PhantomData::<ActiveLevel>,
        }
    }

    /// Consumes the [`Switch`] and returns the underlying
    /// [`InputPin`](embedded_hal::digital::InputPin) or
    /// [`OutputPin`](embedded_hal::digital::OutputPin).
    ///
    /// This is useful for retrieving the underlying pin to use it for a different purpose.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lokey_keyboard::switch::mock;
    /// use lokey_keyboard::switch::{OutputSwitch, Switch, IntoSwitch};
    /// # let pin = mock::Pin::new();
    /// let mut led = pin.into_active_high_switch();
    /// led.on().ok();
    /// let mut pin = led.into_pin();
    /// // do something else with the pin
    /// ```
    pub fn into_pin(self) -> IoPin {
        self.pin.into_inner()
    }
}

/// Zero sized struct for signaling to [`Switch`] that it is active high
pub struct ActiveHigh;
/// Zero sized struct for signaling to [`Switch`] that it is active low
pub struct ActiveLow;

/// Convenience functions for converting [`InputPin`](embedded_hal::digital::InputPin) and
/// [`OutputPin`](embedded_hal::digital::OutputPin) to a [`Switch`].
///
/// The type of [`Switch`] returned, [`InputSwitch`] or [`OutputSwitch`] is determined by whether
/// the `IoPin` being consumed is an [`InputPin`](embedded_hal::digital::InputPin) or
/// [`OutputPin`](embedded_hal::digital::OutputPin).
pub trait IntoSwitch {
    /// Consumes the `IoPin` returning a [`Switch`] of the appropriate `ActiveLevel`.
    ///
    /// This method exists so other, more convenient functions, can have blanket implementations.  
    /// Prefer [`into_active_low_switch`](Self::into_active_low_switch) and
    /// [`into_active_high_switch`](Self::into_active_high_switch).
    ///
    /// # Examples
    ///
    /// ## Active High
    ///
    /// ```
    /// # use lokey_keyboard::switch::mock;
    /// use lokey_keyboard::switch::{ActiveHigh, OutputSwitch, IntoSwitch};
    /// # let pin = mock::Pin::new();
    /// let led = pin.into_switch::<ActiveHigh>();
    /// ```
    ///
    /// ## Active Low
    ///
    /// ```
    /// # use lokey_keyboard::switch::mock;
    /// use lokey_keyboard::switch::{ActiveLow, InputSwitch, IntoSwitch};
    /// # let pin = mock::Pin::new();
    /// let button = pin.into_switch::<ActiveLow>();
    /// ```
    fn into_switch<ActiveLevel>(self) -> Switch<Self, ActiveLevel>
    where
        Self: core::marker::Sized;

    /// Consumes the `IoPin` returning a `Switch<IoPin, ActiveLow>`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lokey_keyboard::switch::mock;
    /// use lokey_keyboard::switch::IntoSwitch;
    /// # let pin = mock::Pin::new();
    /// let led = pin.into_active_low_switch();
    /// ```
    fn into_active_low_switch(self) -> Switch<Self, ActiveLow>
    where
        Self: core::marker::Sized,
    {
        self.into_switch::<ActiveLow>()
    }

    /// Consumes the `IoPin` returning a `Switch<IoPin, ActiveHigh>`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lokey_keyboard::switch::mock;
    /// use lokey_keyboard::switch::IntoSwitch;
    /// # let pin = mock::Pin::new();
    /// let button = pin.into_active_high_switch();
    /// ```
    fn into_active_high_switch(self) -> Switch<Self, ActiveHigh>
    where
        Self: core::marker::Sized,
    {
        self.into_switch::<ActiveHigh>()
    }
}

impl<T> IntoSwitch for T {
    fn into_switch<ActiveLevel>(self) -> Switch<Self, ActiveLevel> {
        Switch::<Self, ActiveLevel>::new(self)
    }
}
