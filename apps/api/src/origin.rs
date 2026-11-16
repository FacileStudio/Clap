//! The origins the app is reached at and may be framed from.

use tronc::env::{Core, EnvError, Vars};

/// The origin Prisme frames embeds from, allowed when `EMBED_ANCESTORS` is unset.
pub const DEFAULT_ANCESTOR: &str = "https://prisme.facile.studio";

fn origin(key: &str, raw: &str) -> Result<String, EnvError> {
    let refused = || EnvError(format!("{key} {raw:?} must be an http(s) origin, without a path"));
    let url = url::Url::parse(raw.trim()).map_err(|_| refused())?;
    let bare = matches!(url.scheme(), "https" | "http") && url.host_str().is_some_and(|host| !host.is_empty());
    bare.then(|| url.origin().ascii_serialization()).ok_or_else(refused)
}

/// `PUBLIC_URL`, else the origin of `OIDC_SUCCESS_URL`, else `http://localhost:<PORT>`.
pub fn public_url(vars: &Vars, core: &Core, porte: &porte::Config) -> Result<String, EnvError> {
    let given = vars.string("PUBLIC_URL", "");
    match (given.trim().is_empty(), porte.success_url.trim().is_empty()) {
        (false, _) => origin("PUBLIC_URL", &given),
        (true, false) => origin("OIDC_SUCCESS_URL", &porte.success_url),
        (true, true) => Ok(format!("http://localhost:{}", core.port)),
    }
}

/// `EMBED_ANCESTORS` as origins, [`DEFAULT_ANCESTOR`] when unset.
pub fn ancestors(vars: &Vars) -> Result<Vec<String>, EnvError> {
    let raw = vars.string("EMBED_ANCESTORS", DEFAULT_ANCESTOR);
    raw.split([' ', ',']).filter(|part| !part.is_empty()).map(|part| origin("EMBED_ANCESTORS", part)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_public_url_and_the_ancestors_are_bare_origins() {
        let core = Core::from_vars(&Vars::from_pairs(vec![("DATABASE_URL", "postgres://x/y")]), true).unwrap();
        let porte = porte::Config { success_url: "https://clap.test/projects".to_owned(), ..porte::Config::default() };
        let none = Vars::from_pairs(Vec::<(&str, &str)>::new());
        assert_eq!(public_url(&none, &core, &porte).unwrap(), "https://clap.test");
        let named = Vars::from_pairs(vec![("PUBLIC_URL", "http://127.0.0.1:9/x")]);
        assert_eq!(public_url(&named, &core, &porte).unwrap(), "http://127.0.0.1:9");
        let bare = porte::Config::default();
        assert_eq!(public_url(&none, &core, &bare).unwrap(), format!("http://localhost:{}", core.port));
        assert_eq!(ancestors(&none).unwrap(), [DEFAULT_ANCESTOR]);
        let two = Vars::from_pairs(vec![("EMBED_ANCESTORS", "https://a.test, https://b.test/")]);
        assert_eq!(ancestors(&two).unwrap(), ["https://a.test", "https://b.test"]);
        assert!(ancestors(&Vars::from_pairs(vec![("EMBED_ANCESTORS", "*")])).is_err());
    }
}
