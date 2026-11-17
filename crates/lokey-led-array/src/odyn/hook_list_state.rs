use core::cell::Cell;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use generic_array::sequence::GenericSequence;
use generic_array::{ArrayLength, GenericArray};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HookIndexOutOfRange;

pub(crate) struct HookListState<N: ArrayLength> {
    signals: GenericArray<Signal<CriticalSectionRawMutex, ()>, N>,
    state: GenericArray<Mutex<CriticalSectionRawMutex, Cell<bool>>, N>,
}

impl<N: ArrayLength> Default for HookListState<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<N: ArrayLength> HookListState<N> {
    pub(crate) fn new() -> Self {
        Self {
            signals: GenericArray::generate(|_| Signal::new()),
            state: GenericArray::generate(|_| Mutex::new(Cell::new(false))),
        }
    }

    pub(crate) fn set(&self, index: usize, enable: bool) -> Result<(), HookIndexOutOfRange> {
        if index >= N::USIZE {
            return Err(HookIndexOutOfRange);
        }
        self.state[index].lock(|state| {
            if state.get() != enable {
                state.set(enable);
                self.signals[index].signal(());
            }
        });
        Ok(())
    }

    pub(crate) fn get(&self, index: usize) -> Result<bool, HookIndexOutOfRange> {
        if index >= N::USIZE {
            return Err(HookIndexOutOfRange);
        }
        Ok(self.state[index].lock(|state| state.get()))
    }

    pub(crate) async fn wait(&self, index: usize) -> Result<bool, HookIndexOutOfRange> {
        if index >= N::USIZE {
            return Err(HookIndexOutOfRange);
        }
        self.signals[index].wait().await;
        Ok(self.state[index].lock(|state| state.get()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typenum::{U0, U2};

    #[test]
    fn new_starts_disabled() {
        let state = HookListState::<U2>::new();
        assert_eq!(state.get(0), Ok(false));
        assert_eq!(state.get(1), Ok(false));
    }

    #[test]
    fn default_starts_disabled() {
        let state = HookListState::<U2>::default();
        assert_eq!(state.get(0), Ok(false));
        assert_eq!(state.get(1), Ok(false));
    }

    #[test]
    fn set_updates_state() {
        let state = HookListState::<U2>::new();

        state.set(0, true).unwrap();
        assert_eq!(state.get(0), Ok(true));
        assert_eq!(state.get(1), Ok(false));

        state.set(0, false).unwrap();
        assert_eq!(state.get(0), Ok(false));
    }

    #[test]
    fn out_of_range_returns_error() {
        let state = HookListState::<U2>::new();
        assert_eq!(state.get(2), Err(HookIndexOutOfRange));
        assert_eq!(state.set(2, true), Err(HookIndexOutOfRange));
    }

    #[test]
    fn empty_state_has_no_indices() {
        let state = HookListState::<U0>::new();
        assert_eq!(state.get(0), Err(HookIndexOutOfRange));
        assert_eq!(state.set(0, true), Err(HookIndexOutOfRange));
    }

    #[test]
    fn wait_returns_pending_state() {
        let state = HookListState::<U2>::new();
        state.set(1, true).unwrap();
        assert_eq!(embassy_futures::block_on(state.wait(1)), Ok(true));
    }

    #[test]
    fn wait_resolves_after_set() {
        let state = HookListState::<U2>::new();
        let result = embassy_futures::block_on(async {
            embassy_futures::join::join(
                async {
                    state.set(0, true).unwrap();
                },
                state.wait(0),
            )
            .await
            .1
        });
        assert_eq!(result, Ok(true));
    }

    #[test]
    fn wait_out_of_range_returns_error() {
        let state = HookListState::<U2>::new();
        assert_eq!(
            embassy_futures::block_on(state.wait(2)),
            Err(HookIndexOutOfRange)
        );
    }
}
