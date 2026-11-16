//! The `clap` command line: write a video as one text file, preview it with live reload, edit
//! its timeline, render it to an MP4, and sync it with a Clap server.
//!
//! Every HTTP call lives in [`api`]. The decisions of `pull` and `push` are the pure functions
//! of [`sync::plan`]. Rendering is `clap-render`, and timeline edits are `clap-timeline`.

pub mod api;
mod browser;
pub mod cli;
mod commands;
pub mod config;
pub mod diagnose;
pub mod failure;
pub mod login;
mod mime;
mod progress;
pub mod project;
mod run;
mod serve;
pub mod starters;
pub mod sync;
pub mod ui;
pub mod url;

pub use run::run;
