pub mod channel;
pub mod pubsub;

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
pub(crate) use debug;
#[allow(unused)]
pub(crate) use error;
#[allow(unused)]
pub(crate) use info;
pub(crate) use unwrap;
#[allow(unused)]
pub(crate) use warn_ as warn;
