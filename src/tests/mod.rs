#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

macro_rules! access {
    ($value:ident$($rest:tt)*) => {
        access!(@impl ($value) | $($rest)*)
    };
    (@impl ($curr:expr) | .$prop:ident$($rest:tt)*) => {
        access!(@impl (($curr).obj().index(stringify!($prop))) | $($rest)*)
    };
    (@impl ($curr:expr) | [$idx:literal]$($rest:tt)*) => {
        access!(@impl (($curr).arr().index($idx)) | $($rest)*)
    };
    (@impl ($curr:expr) | $(($final_fn:ident))?) => {
        $curr$(.$final_fn())?
    };
}

mod challs;
mod defaults;
mod json;
mod req;
mod utils;
