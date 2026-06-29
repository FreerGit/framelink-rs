#![no_std]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![warn(
    clippy::unwrap_used,         // no silent panics
    clippy::expect_used,         // same
    clippy::panic,               // explicit about panics
    clippy::indexing_slicing,    // bounds-check discipline
    clippy::integer_division,    // lossy division awareness
    clippy::as_conversions,      // prefer From/Into
    clippy::todo,                // no leftover TODOs in CI
    clippy::unreachable,         // be explicit
)]

pub mod frame;

pub mod transport;

pub mod protocol;
