//! Clap, the Facile suite's video editor whose project is one text file: the API.
#![deny(unused, dead_code)]

pub mod apiref;
pub mod auth;
pub mod env;
pub mod extract;
pub mod limits;
pub mod mcp;
pub mod modules;
pub mod origin;
pub mod router;
pub mod security;
pub mod server;
pub mod state;
pub mod version;

/// The app's migrations, embedded at build time.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

#[cfg(test)]
mod tests {
    #[test]
    fn the_first_migration_is_the_porte_schema_verbatim() {
        assert_eq!(include_str!("../migrations/0001_porte.sql"), porte::pg::SCHEMA);
    }
}
