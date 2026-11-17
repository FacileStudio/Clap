//! Assets: the media of a project, named by the relative path its source writes, stored once by
//! content on the data volume, then probed and given a poster, a filmstrip, a waveform and a
//! 720p proxy by the render worker, in its sandbox: the API never decodes an upload.

pub mod derive;
mod facts;
pub mod files;
pub mod inflight;
pub mod intake;
pub mod leftovers;
pub mod media;
pub mod name;
mod previews;
pub mod quota;
pub mod rename;
mod router;
pub mod seal;
mod serve;
pub mod service;
pub mod sniff;
pub mod sweep;
pub mod temporary;
pub mod tools;
pub mod upload;
pub mod worker;

pub use router::routes;
