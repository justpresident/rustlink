#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::panic,
    clippy::dbg_macro,
    clippy::missing_const_for_fn,
    clippy::needless_pass_by_value,
    clippy::redundant_pub_crate
)]
#![allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::multiple_crate_versions,
    clippy::missing_panics_doc,
    clippy::option_if_let_else
)]

pub mod app;
pub mod commands;
pub mod connection;
pub mod model;
pub mod player;
pub mod terminal;
pub mod tools;
pub mod ui;
pub mod world;
