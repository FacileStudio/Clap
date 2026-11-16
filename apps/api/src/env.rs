use std::path::PathBuf;

use tronc::env::{Core, EnvError, Vars};

use crate::limits::Limits;
use crate::modules::assets::tools::Tools;
use crate::modules::assistant::Bulle;

/// Everything the binary reads from its environment.
#[derive(Debug, Clone)]
pub struct Config {
    /// The chassis variables: port, database, proxies, Journal.
    pub core: Core,
    /// Registre sign-in. `sso_only` is always true.
    pub porte: porte::Config,
    /// `PORTE_TOKEN_KEY`: seals the stored OIDC tokens, and the frame asset links derive their key
    /// from it.
    pub token_key: String,
    /// `DATA_DIR`: where uploads, their posters and proxies, render jobs and rendered files live.
    pub data_dir: PathBuf,
    /// `MCP_RESOURCE`: turns `/mcp` on.
    pub mcp_resource: String,
    /// `MCP_STATE_KEY`: the HMAC key of the confirmation flow.
    pub mcp_state_key: String,
    /// The byte and render caps.
    pub limits: Limits,
    /// `CLAP_FFMPEG` and `CLAP_FFPROBE`: the media binaries, looked up on `PATH` when unset.
    pub tools: Tools,
    /// `BULLE_URL` and `BULLE_CLIENT_ID`: where the person's assistant lives.
    pub bulle: Bulle,
}

impl Config {
    /// Reads the process environment.
    pub fn load() -> Result<Self, EnvError> {
        Self::from_vars(&Vars::process())
    }

    /// Reads `vars`, refusing a production boot without Registre.
    pub fn from_vars(vars: &Vars) -> Result<Self, EnvError> {
        let core = Core::from_vars(vars, true)?;
        let text = |key: &str| vars.string(key, "").trim().to_owned();
        let porte = porte::Config { app_name: "Clap".to_owned(), sso_only: true, ..porte::Config::from_vars(vars)? };
        let config = Config {
            porte,
            token_key: text("PORTE_TOKEN_KEY"),
            data_dir: PathBuf::from(vars.string("DATA_DIR", "/data")),
            mcp_resource: text("MCP_RESOURCE"),
            mcp_state_key: text("MCP_STATE_KEY"),
            limits: Limits::from_vars(vars)?,
            tools: Tools::from_vars(vars),
            bulle: Bulle::from_vars(vars)?,
            core,
        };
        config.check()?;
        Ok(config)
    }

    fn check(&self) -> Result<(), EnvError> {
        if self.core.is_production() && self.porte.issuer.is_empty() {
            return Err(EnvError("OIDC_ISSUER is required in production".to_owned()));
        }
        if self.core.is_production() && self.token_key.is_empty() {
            return Err(EnvError("PORTE_TOKEN_KEY is required in production".to_owned()));
        }
        if !self.mcp_resource.is_empty() && self.mcp_state_key.is_empty() {
            return Err(EnvError("MCP_STATE_KEY is required with MCP_RESOURCE".to_owned()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(extra: &[(&str, &str)]) -> Vars {
        let mut pairs = vec![("DATABASE_URL", "postgres://x/y")];
        pairs.extend_from_slice(extra);
        Vars::from_pairs(pairs)
    }

    #[test]
    fn production_needs_the_issuer() {
        let error = Config::from_vars(&vars(&[("APP_ENV", "production")])).unwrap_err();
        assert!(error.0.contains("OIDC_ISSUER"), "{error}");
    }

    #[test]
    fn mcp_needs_a_state_key() {
        let error = Config::from_vars(&vars(&[("MCP_RESOURCE", "https://clap.test/mcp")])).unwrap_err();
        assert!(error.0.contains("MCP_STATE_KEY"), "{error}");
    }

    #[test]
    fn defaults_are_sso_only_with_slash_data_the_suite_bulle_and_path_binaries() {
        let config = Config::from_vars(&vars(&[])).unwrap();
        assert!(config.porte.sso_only);
        assert_eq!(config.porte.app_name, "Clap");
        assert_eq!(config.data_dir, PathBuf::from("/data"));
        assert_eq!(config.bulle.url, "https://bulle.facile.studio");
        assert_eq!(config.tools.ffprobe, PathBuf::from("ffprobe"));
        let config = Config::from_vars(&vars(&[("CLAP_FFMPEG", "/opt/ffmpeg")])).unwrap();
        assert_eq!(config.tools.ffmpeg, PathBuf::from("/opt/ffmpeg"));
    }
}
