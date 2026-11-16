//! Assets: the media of a project, named by the relative path its source writes, stored once by
//! content on the data volume, probed and given a poster, a filmstrip, a waveform and a 720p
//! proxy in the background.

pub mod derive;
pub mod files;
pub mod inflight;
pub mod intake;
pub mod leftovers;
pub mod name;
pub mod peaks;
mod previews;
pub mod probe;
pub mod quota;
mod router;
pub mod seal;
mod serve;
pub mod service;
pub mod sniff;
pub mod strip;
pub mod sweep;
pub mod temporary;
pub mod tools;
pub mod upload;
pub mod worker;

pub use router::routes;
