pub mod channel;
pub mod pubsub;

#[allow(unused)]
#[collapse_debuginfo(yes)]
macro_rules! unwrap {
    ($($x:tt)*) => {{
        #[cfg(feature = "defmt")]
        { defmt::unwrap!($($x)*) }
        #[cfg(not(feature = "defmt"))]
        { $($x)*.unwrap() }
    }}
}

#[allow(unused)]
#[collapse_debuginfo(yes)]
macro_rules! panic {
    ($($x:tt)*) => {
        {
            #[cfg(feature = "defmt")]
            defmt::panic!($($x)*);
            #[cfg(not(feature = "defmt"))]
            core::panic!($($x)*);
        }
    };
}

#[allow(unused)]
#[collapse_debuginfo(yes)]
macro_rules! error {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(feature = "defmt")]
            defmt::error!($s $(, $x)*);
            #[cfg(not(feature = "defmt"))]
            let _ = ($(&$x),*);
        }
    };
}

#[allow(unused)]
#[collapse_debuginfo(yes)]
macro_rules! warn_ {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(feature = "defmt")]
            defmt::warn!($s $(, $x)*);
            #[cfg(not(feature = "defmt"))]
            let _ = ($(&$x),*);
        }
    };
}

#[allow(unused)]
#[collapse_debuginfo(yes)]
macro_rules! info {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(feature = "defmt")]
            defmt::info!($s $(, $x)*);
            #[cfg(not(feature = "defmt"))]
            let _ = ($(&$x),*);
        }
    };
}

#[allow(unused)]
#[collapse_debuginfo(yes)]
macro_rules! debug {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(feature = "defmt")]
            defmt::debug!($s $(, $x)*);
            #[cfg(not(feature = "defmt"))]
            let _ = ($(&$x),*);
        }
    };
}

#[allow(unused)]
pub(crate) use {debug, error, info, panic, unwrap, warn_ as warn};
