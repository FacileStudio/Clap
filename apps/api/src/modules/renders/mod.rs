//! Renders: the API's half of the file protocol it shares with the render worker. The API writes
//! a job directory and reads the worker's status file; it never launches Chrome.
//!
//! ```text
//! DATA_DIR/renders/<id>/bundle/index.html   the compiled scene, written by the API
//! DATA_DIR/renders/<id>/bundle/media/...    the original media the scene names
//! DATA_DIR/renders/<id>/job.json            written last, by rename: the job is ready
//! DATA_DIR/renders/<id>/status.json         written by the worker, by rename
//! DATA_DIR/renders/<id>/out.mp4             written by the worker
//! DATA_DIR/renders/<id>/cancel              written by the API to ask the worker to stop
//! DATA_DIR/outputs/<id>.mp4                 where the API keeps a finished render
//! ```

pub mod bundle;
mod events;
mod handlers;
pub mod jobfs;
pub mod mirror;
pub mod output;
pub mod posted;
mod router;
mod serve;
pub mod service;
pub mod spec;
pub mod stage;
pub mod status;
pub mod sweep;
pub mod types;

pub use handlers::posted as posted_route;
pub use router::routes;
