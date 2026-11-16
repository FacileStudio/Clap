//! What a share link opens, with no session and no cookie: the latest finished render of the
//! shared project, its video and poster, the `/v/{token}` and `/embed/{token}` page shells and
//! oEmbed. Every route answers `404` for a token that is unknown, revoked, on a trashed project
//! or on a project with no finished render, and none answers the source, the project id or the
//! space.

mod guard;
pub mod info;
pub mod oembed;
pub mod pages;
pub mod poster;
mod router;
mod site;

pub use router::api_routes;
pub use site::root_routes;
