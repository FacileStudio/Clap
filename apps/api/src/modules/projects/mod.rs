//! Projects: one source text per project, its revisions, trash and the checks every write shares.

pub mod access;
pub mod edit;
pub mod error;
mod handlers;
pub mod patch;
pub mod problem;
pub mod purge;
mod router;
pub mod service;
pub mod source;
pub mod timeline;
pub mod trash;
pub mod types;
pub mod write;

pub use router::routes;
