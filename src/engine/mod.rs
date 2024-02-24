#![cfg_attr(
    not(debug_assertions),
    windows_subsystem = "windows"
)]

pub mod config;
pub mod context;
pub mod event;
pub mod graphics;
pub mod input;
pub mod time;
pub mod updater;