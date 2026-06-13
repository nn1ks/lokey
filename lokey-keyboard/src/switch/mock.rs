#![allow(clippy::bool_assert_comparison)]

use core::future::poll_fn;
use core::task::Poll;
use embedded_hal::digital::{ErrorType, InputPin, OutputPin, StatefulOutputPin};
use embedded_hal_async::digital::Wait;

#[derive(PartialEq, Eq, Debug)]
pub enum State {
    Low,
    High,
}

pub struct Pin {
    state: Option<State>,
}

impl Default for Pin {
    fn default() -> Self {
        Self::new()
    }
}

impl Pin {
    pub fn new() -> Self {
        Pin { state: None }
    }

    pub fn with_state(state: State) -> Self {
        Pin { state: Some(state) }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct MockError(&'static str);

impl embedded_hal::digital::Error for MockError {
    fn kind(&self) -> embedded_hal::digital::ErrorKind {
        embedded_hal::digital::ErrorKind::Other
    }
}

impl ErrorType for Pin {
    type Error = MockError;
}

impl InputPin for Pin {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        match self.state {
            Some(State::High) => Ok(true),
            Some(State::Low) => Ok(false),
            None => Err(MockError("state not set")),
        }
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        match self.is_high() {
            Ok(v) => Ok(!v),
            Err(e) => Err(e),
        }
    }
}

impl OutputPin for Pin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.state = Some(State::Low);
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.state = Some(State::High);
        Ok(())
    }
}

impl StatefulOutputPin for Pin {
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        self.is_low()
    }

    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        self.is_high()
    }
}

impl Wait for Pin {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        poll_fn(|_cx| {
            if self.state == Some(State::High) {
                Poll::Ready(Ok(()))
            } else {
                Poll::Pending
            }
        })
        .await
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        poll_fn(|_cx| {
            if self.state == Some(State::Low) {
                Poll::Ready(Ok(()))
            } else {
                Poll::Pending
            }
        })
        .await
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_low().await?;
        self.wait_for_high().await
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_high().await?;
        self.wait_for_low().await
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        if self.is_high()? {
            self.wait_for_falling_edge().await
        } else {
            self.wait_for_rising_edge().await
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod new {
        use super::*;

        #[test]
        fn state_is_uninitialized() {
            let mut pin = Pin::new();
            assert_eq!(None, pin.state);
            pin.is_low().expect_err("Expected uninitialized pin");
        }
    }

    mod input_pin {
        use super::*;

        #[test]
        fn error_when_uninitialized() {
            let mut pin = Pin { state: None };
            pin.is_high().expect_err("Expected uninitialized pin");
        }

        mod is_high {
            use super::*;

            #[test]
            fn returns_true_when_state_is_high() {
                let mut pin = Pin::with_state(State::High);
                assert_eq!(true, pin.is_high().unwrap());
            }

            #[test]
            fn returns_false_when_state_is_low() {
                let mut pin = Pin::with_state(State::Low);
                assert_eq!(false, pin.is_high().unwrap());
            }
        }

        mod is_low {
            use super::*;

            #[test]
            fn returns_false_when_state_is_high() {
                let mut pin = Pin::with_state(State::High);
                assert_eq!(false, pin.is_low().unwrap());
            }

            #[test]
            fn returns_true_when_state_is_high() {
                let mut pin = Pin::with_state(State::Low);
                assert_eq!(true, pin.is_low().unwrap());
            }
        }

        mod asynch {
            use super::*;
            use core::future::Future;
            use core::pin::pin;
            use core::task::{Context, Poll};
            use noop_waker::noop_waker;

            #[test]
            fn wait_for_high_while_high() {
                let mut pin = Pin::with_state(State::High);
                let mut future = pin!(pin.wait_for_high());
                let waker = noop_waker();
                let mut cx = Context::from_waker(&waker);

                assert_eq!(
                    true,
                    matches!(future.as_mut().poll(&mut cx), Poll::Ready(_))
                )
            }

            #[test]
            fn wait_for_high_while_low() {
                let mut pin = Pin::with_state(State::Low);
                let mut future = pin!(pin.wait_for_high());
                let waker = noop_waker();
                let mut cx = Context::from_waker(&waker);

                assert_eq!(true, matches!(future.as_mut().poll(&mut cx), Poll::Pending));
            }

            #[test]
            fn wait_for_low_while_low() {
                let mut pin = Pin::with_state(State::Low);
                let mut future = pin!(pin.wait_for_low());
                let waker = noop_waker();
                let mut cx = Context::from_waker(&waker);

                assert_eq!(
                    true,
                    matches!(future.as_mut().poll(&mut cx), Poll::Ready(_))
                );
            }

            #[test]
            fn wait_for_low_while_high() {
                let mut pin = Pin::with_state(State::High);
                let mut future = pin!(pin.wait_for_low());
                let waker = noop_waker();
                let mut cx = Context::from_waker(&waker);

                assert_eq!(true, matches!(future.as_mut().poll(&mut cx), Poll::Pending));
            }

            #[test]
            fn wait_for_any_edge_while_high() {
                let mut pin = Pin::with_state(State::High);
                let mut future = pin!(pin.wait_for_any_edge());
                let waker = noop_waker();
                let mut cx = Context::from_waker(&waker);

                assert_eq!(true, matches!(future.as_mut().poll(&mut cx), Poll::Pending));
            }

            #[test]
            fn wait_for_any_edge_while_low() {
                let mut pin = Pin::with_state(State::Low);
                let mut future = pin!(pin.wait_for_any_edge());
                let waker = noop_waker();
                let mut cx = Context::from_waker(&waker);

                assert_eq!(true, matches!(future.as_mut().poll(&mut cx), Poll::Pending));
            }
        }
    }

    mod output_pin {
        use super::*;

        #[test]
        fn set_low() {
            let mut pin = Pin::new();
            pin.set_low().unwrap();

            assert_eq!(true, pin.is_low().unwrap());
        }

        #[test]
        fn set_high() {
            let mut pin = Pin::new();
            pin.set_high().unwrap();

            assert_eq!(true, pin.is_high().unwrap());
        }
    }

    mod stateful_output_pin {
        use super::*;

        #[test]
        fn error_when_uninitialized() {
            let mut pin = Pin { state: None };
            pin.is_set_high().expect_err("Expected uninitialized pin");
        }

        mod is_set_low {
            use super::*;

            #[test]
            fn returns_false_when_state_is_high() {
                let mut pin = Pin::with_state(State::High);
                assert_eq!(false, pin.is_set_low().unwrap());
            }

            #[test]
            fn returns_true_when_state_is_high() {
                let mut pin = Pin::with_state(State::Low);
                assert_eq!(true, pin.is_set_low().unwrap());
            }
        }

        mod is_set_high {
            use super::*;

            #[test]
            fn returns_true_when_state_is_high() {
                let mut pin = Pin::with_state(State::High);
                assert_eq!(true, pin.is_set_high().unwrap());
            }

            #[test]
            fn returns_false_when_state_is_low() {
                let mut pin = Pin::with_state(State::Low);
                assert_eq!(false, pin.is_set_high().unwrap());
            }
        }

        mod toggleable {
            use super::*;
            use embedded_hal::digital::StatefulOutputPin;

            #[test]
            fn default_toggleable_impl() {
                let mut pin = Pin::with_state(State::Low);
                pin.toggle().unwrap();
                assert_eq!(true, pin.is_set_high().unwrap());
            }
        }
    }
}
