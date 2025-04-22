pub mod channel;
pub mod pubsub;

#[macro_export]
#[doc(hidden)]
#[collapse_debuginfo(yes)]
macro_rules! __unwrap {
    ($($x:tt)*) => {{
        #[cfg(feature = "defmt")]
        { defmt::unwrap!($($x)*) }
        #[cfg(not(feature = "defmt"))]
        { $($x)*.unwrap() }
    }}
}

#[macro_export]
#[doc(hidden)]
#[collapse_debuginfo(yes)]
macro_rules! __panic {
    ($($x:tt)*) => {
        {
            #[cfg(feature = "defmt")]
            defmt::panic!($($x)*);
            #[cfg(not(feature = "defmt"))]
            core::panic!($($x)*);
        }
    };
}

#[macro_export]
#[doc(hidden)]
#[collapse_debuginfo(yes)]
macro_rules! __error {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(feature = "defmt")]
            defmt::error!($s $(, $x)*);
            #[cfg(not(feature = "defmt"))]
            let _ = ($(&$x),*);
        }
    };
}

#[macro_export]
#[doc(hidden)]
#[collapse_debuginfo(yes)]
macro_rules! __warn {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(feature = "defmt")]
            defmt::warn!($s $(, $x)*);
            #[cfg(not(feature = "defmt"))]
            let _ = ($(&$x),*);
        }
    };
}

#[macro_export]
#[doc(hidden)]
#[collapse_debuginfo(yes)]
macro_rules! __info {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(feature = "defmt")]
            defmt::info!($s $(, $x)*);
            #[cfg(not(feature = "defmt"))]
            let _ = ($(&$x),*);
        }
    };
}

#[macro_export]
#[doc(hidden)]
#[collapse_debuginfo(yes)]
macro_rules! __debug {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(feature = "defmt")]
            defmt::debug!($s $(, $x)*);
            #[cfg(not(feature = "defmt"))]
            let _ = ($(&$x),*);
        }
    };
}

#[doc(inline)]
pub use {
    __debug as debug, __error as error, __info as info, __panic as panic, __unwrap as unwrap,
    __warn as warn,
};
