use crate::odyn::hook_list_state::HookListState;
use crate::odyn::{HookBitmask, OdynHook, OdynHookBundle};
use embassy_futures::select::select;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex as AsyncMutex;
use generic_array::ArrayLength;
use lokey::DynContext;
use seq_macro::seq;

pub trait TupleSize {
    type Size: ArrayLength;
}

pub struct TupleHookBundle<Hooks: TupleSize> {
    hooks: Hooks,
    state: HookListState<Hooks::Size>,
}

/// Maps a tuple arity to the corresponding [`typenum`] length type.
macro_rules! tuple_size {
    (0) => {
        typenum::U0
    };
    (1) => {
        typenum::U1
    };
    (2) => {
        typenum::U2
    };
    (3) => {
        typenum::U3
    };
    (4) => {
        typenum::U4
    };
    (5) => {
        typenum::U5
    };
    (6) => {
        typenum::U6
    };
    (7) => {
        typenum::U7
    };
    (8) => {
        typenum::U8
    };
    (9) => {
        typenum::U9
    };
    (10) => {
        typenum::U10
    };
    (11) => {
        typenum::U11
    };
    (12) => {
        typenum::U12
    };
    (13) => {
        typenum::U13
    };
    (14) => {
        typenum::U14
    };
    (15) => {
        typenum::U15
    };
    (16) => {
        typenum::U16
    };
}

macro_rules! impl_tuple_hook_bundle {
    ($num:literal) => {
        seq!(N in 0..=$num {
            #(impl_tuple_hook_bundle!(@ N);)*
        });
    };
    (@ $num:literal) => {
        seq!(N in 0..$num {
            impl<#(T~N,)*> TupleSize for (#(T~N,)*) {
                type Size = tuple_size!($num);
            }

            impl<#(T~N,)*> From<(#(T~N,)*)>
                for TupleHookBundle<(#(AsyncMutex<CriticalSectionRawMutex, T~N>,)*)>
            {
                #[allow(unused)]
                fn from(value: (#(T~N,)*)) -> Self {
                    Self {
                        hooks: (#(AsyncMutex::new(value.N),)*),
                        state: HookListState::new(),
                    }
                }
            }

            impl<#(T~N,)*> OdynHookBundle
                for TupleHookBundle<(#(AsyncMutex<CriticalSectionRawMutex, T~N>,)*)>
            where
                #(T~N: OdynHook + 'static,)*
            {
                #[allow(unused_variables)]
                async fn run_all<const NUM_LEDS: usize>(&self, context: DynContext) {
                    #(
                        let fut~N = async {
                            loop {
                                let enabled = self.state.get(N).unwrap() || self.state.wait(N).await.unwrap();
                                if enabled {
                                    let mut hook = self.hooks.N.lock().await;
                                    let fut_run = hook.run::<NUM_LEDS>(context);
                                    let fut_wait_disabled = async { self.state.wait(N).await.unwrap() };
                                    select(fut_run, fut_wait_disabled).await;
                                }
                            }
                        };
                    )*
                    futures_util::join!(#(fut~N,)*);
                }

                #[allow(unused_variables)]
                fn set_enabled_hooks(&self, bitmask: HookBitmask) {
                    #(self.state.set(N, bitmask.value & T~N::ID > 0).unwrap();)*
                }

                #[allow(unused_mut)]
                fn enabled_hooks(&self) -> HookBitmask {
                    let mut bitmask = 0u64;
                    #(
                        if self.state.get(N).unwrap() {
                            bitmask |= T~N::ID;
                        }
                    )*
                    bitmask.into()
                }

                #[allow(unused_mut)]
                fn available_hooks(&self) -> HookBitmask {
                    let mut bitmask = 0u64;
                    #(bitmask |= T~N::ID;)*
                    bitmask.into()
                }
            }
        });
    };
}

impl_tuple_hook_bundle!(16);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Hook;
    use generic_array::typenum::Unsigned;

    macro_rules! define_test_hooks {
        ($($name:ident => $bit:expr),* $(,)?) => {
            $(
                struct $name;

                impl Hook for $name {
                    async fn run<const NUM_LEDS: usize>(&mut self, _context: DynContext) {}
                }

                impl OdynHook for $name {
                    const ID: u64 = 1 << $bit;
                }
            )*
        };
    }

    define_test_hooks!(
        Hook0 => 0,
        Hook1 => 1,
        Hook2 => 2,
        Hook3 => 3,
        Hook4 => 4,
        Hook5 => 5,
        Hook6 => 6,
        Hook7 => 7,
        Hook8 => 8,
        Hook9 => 9,
        Hook10 => 10,
        Hook11 => 11,
        Hook12 => 12,
        Hook13 => 13,
        Hook14 => 14,
        Hook15 => 15,
    );

    #[test]
    fn tuple_size_matches_arity() {
        assert_eq!(<(Hook0,) as TupleSize>::Size::USIZE, 1);
        assert_eq!(<(Hook0, Hook1) as TupleSize>::Size::USIZE, 2);
        assert_eq!(<(Hook0, Hook1, Hook2) as TupleSize>::Size::USIZE, 3);
        assert_eq!(
            <(
                Hook0,
                Hook1,
                Hook2,
                Hook3,
                Hook4,
                Hook5,
                Hook6,
                Hook7,
                Hook8,
                Hook9,
                Hook10,
                Hook11,
                Hook12,
                Hook13,
                Hook14,
                Hook15,
            ) as TupleSize>::Size::USIZE,
            16
        );
    }

    #[test]
    fn from_creates_disabled_bundle() {
        let bundle = TupleHookBundle::from((Hook0, Hook1, Hook2));
        assert_eq!(bundle.enabled_hooks().value, 0);
        assert_eq!(
            bundle.available_hooks().value,
            Hook0::ID | Hook1::ID | Hook2::ID
        );
    }

    #[test]
    fn set_enabled_hooks_updates_enabled_hooks() {
        let bundle = TupleHookBundle::from((Hook0, Hook1, Hook2));

        bundle.set_enabled_hooks(HookBitmask::from(Hook0::ID | Hook2::ID));
        assert_eq!(bundle.enabled_hooks().value, Hook0::ID | Hook2::ID);

        bundle.set_enabled_hooks(HookBitmask::from(Hook1::ID));
        assert_eq!(bundle.enabled_hooks().value, Hook1::ID);

        bundle.set_enabled_hooks(HookBitmask::from(0));
        assert_eq!(bundle.enabled_hooks().value, 0);
    }

    #[test]
    fn set_enabled_hooks_ignores_unknown_hooks() {
        let bundle = TupleHookBundle::from((Hook0, Hook1));
        bundle.set_enabled_hooks(HookBitmask::from(Hook2::ID));
        assert_eq!(bundle.enabled_hooks().value, 0);
    }

    #[test]
    fn empty_tuple_bundle_has_no_hooks() {
        let bundle = TupleHookBundle::from(());
        assert_eq!(bundle.enabled_hooks().value, 0);
        assert_eq!(bundle.available_hooks().value, 0);
    }

    #[test]
    fn sixteen_tuple_available_hooks() {
        let bundle = TupleHookBundle::from((
            Hook0, Hook1, Hook2, Hook3, Hook4, Hook5, Hook6, Hook7, Hook8, Hook9, Hook10, Hook11,
            Hook12, Hook13, Hook14, Hook15,
        ));
        let expected = (0..16).fold(0u64, |acc, bit| acc | (1 << bit));
        assert_eq!(bundle.available_hooks().value, expected);
    }
}
