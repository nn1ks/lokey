pub mod channel;
pub mod pubsub;

macro_rules! unwrap {
    ($($x:tt)*) => {{
        #[cfg(feature = "defmt")]
        { defmt::unwrap!($($x)*) }
        #[cfg(not(feature = "defmt"))]
        { $($x)*.unwrap() }
    }}
}

pub(crate) use unwrap;
