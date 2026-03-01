macro_rules! __declare_const_for_feature_group {
    // Entry: list of (feature, value) tuples
    ($name:ident, [ $( ( $feat:expr, $val:expr ) ),+ $(,)? ]) => {
        // Default when none of the features are enabled
        #[cfg(not(any( $( feature = $feat ),* )))]
        #[cfg_attr(docsrs, doc(auto_cfg = false))]
        pub const $name: usize = 0;

        // Emit per-feature consts, each excluding all remaining features to avoid duplicate defs.
        $crate::util::declare_const_for_feature_group!(@emit $name, $( ( $feat, $val ) ),+ );
    };

    // Recursive emitter: when more than one pair remain, emit for the first while excluding the rest
    (@emit $name:ident, ( $feat:expr, $val:expr ), $( ( $rest_feat:expr, $rest_val:expr ) ),+ ) => {
        #[cfg(all(feature = $feat, not(any( $( feature = $rest_feat ),* ))))]
        #[cfg_attr(docsrs, doc(auto_cfg = false))]
        pub const $name: usize = $val;

        $crate::util::declare_const_for_feature_group!(@emit $name, $( ( $rest_feat, $rest_val ) ),+ );
    };

    // Base case: single remaining pair -> no exclusion necessary
    (@emit $name:ident, ( $feat:expr, $val:expr ) ) => {
        #[cfg(feature = $feat)]
        #[cfg_attr(docsrs, doc(auto_cfg = false))]
        pub const $name: usize = $val;
    };
}

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
pub(crate) use __declare_const_for_feature_group as declare_const_for_feature_group;
#[doc(inline)]
pub use {
    __debug as debug, __error as error, __info as info, __panic as panic, __unwrap as unwrap,
    __warn as warn,
};
