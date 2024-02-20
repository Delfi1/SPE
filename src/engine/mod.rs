#![cfg_attr(
    not(debug_assertions),
    windows_subsystem = "windows"
)]

pub(crate) mod config;
pub(crate) mod context;
pub(crate) mod event;
pub(crate) mod graphics;
pub(crate) mod input;
pub(crate) mod time;
pub(crate) mod shaders;
