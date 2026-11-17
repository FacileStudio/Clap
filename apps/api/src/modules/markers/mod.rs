//! Ruler markers: named times on a project's timeline, kept by the API beside the source, since
//! the `clap.md` format has no place for them.

mod router;
pub mod service;

pub use router::routes;
