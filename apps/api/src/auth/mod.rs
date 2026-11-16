//! porte wiring: sessions, Registre sign-in, directory sync, deletions and MCP auth.

mod deletions;
mod picture;
mod retire;
mod routes;
mod spaces;

use std::sync::Arc;
use std::time::Duration;

use porte::directory::{self, Deletions, Syncer};
use porte::exchange::{self, Exchanger};
use porte::http_client::HttpClient;
use porte::oidc::{self, Kit};
use porte::session::{self, Manager};
use porte::{mcpauth, pg};
use sqlx::PgPool;

pub use deletions::{AppData, DeletionState};
pub use picture::Pictures;
pub use routes::routes;
pub use spaces::SpacesApply;

use crate::env::Config;

/// Every auth kit the app builds once at boot.
#[derive(Clone)]
pub struct Kits {
    /// Sessions, cookies and the auth middleware.
    pub sessions: Manager,
    /// Registre sign-in routes.
    pub oidc: Kit,
    /// The directory loop, when Registre is configured.
    pub syncer: Option<Syncer>,
    /// The deletion loop and receiver, when Registre is configured.
    pub deletions: Option<Deletions>,
    /// The MCP token verifier, when `MCP_RESOURCE` is set.
    pub mcp: Option<Arc<mcpauth::Verifier>>,
    /// Token exchange to other suite apps, when Registre is configured.
    pub exchanger: Option<Exchanger>,
    /// Verifies the bearer of `POST /api/renders` and creates the account on a first call, when
    /// Registre is configured.
    pub machine: Option<Arc<mcpauth::Verifier>>,
}

type BootError = Box<dyn std::error::Error + Send + Sync>;

impl Kits {
    /// Builds the kits. A wrong issuer fails here, so the boot fails.
    pub async fn new(cfg: &Config, pool: &PgPool) -> Result<Kits, BootError> {
        Kits::over(cfg, pool, HttpClient::new()).await
    }

    /// [`Kits::new`] with the outbound HTTP given: a test answers for a fake issuer.
    pub async fn over(cfg: &Config, pool: &PgPool, http: HttpClient) -> Result<Kits, BootError> {
        let store = pg::Store::new(pool.clone()).with_token_sealer(pg::TokenSealer::parse(&cfg.token_key)?);
        let sessions = Manager::new(&cfg.porte, session::Deps { sessions: store.sessions(), now: None });
        let oidc = Kit::new(&cfg.porte, deps(&store, &sessions, &http)).await?;
        let mut kits =
            Kits { sessions, oidc, syncer: None, deletions: None, mcp: None, exchanger: None, machine: None };
        if cfg.porte.enabled() {
            kits.exchanger = Some(Exchanger::new(exchange::Config::from_porte(&cfg.porte, store.identities()))?);
            kits.directory(cfg, pool, store.clone());
            let delegated = delegated(cfg, &store, &kits, &http).await?;
            let resource = machine_resource(cfg);
            kits.machine = Some(Arc::new(verifier(cfg, &store, &delegated, (resource, http.clone())).await?));
        }
        if !cfg.mcp_resource.is_empty() {
            let resource = cfg.mcp_resource.clone();
            kits.mcp = Some(Arc::new(verifier(cfg, &store, &kits.oidc, (resource, http)).await?));
        }
        Ok(kits)
    }

    fn directory(&mut self, cfg: &Config, pool: &PgPool, store: pg::Store) {
        let client = directory::Client {
            issuer: cfg.porte.issuer.clone(),
            client_id: cfg.porte.client_id.clone(),
            client_secret: cfg.porte.client_secret.clone(),
            http: HttpClient::new(),
        };
        let apply = Arc::new(SpacesApply::new(pool.clone()));
        self.syncer = Some(Syncer::new(client.clone(), apply, Duration::ZERO));
        let data = Arc::new(AppData::new(pool.clone()));
        let state = Arc::new(DeletionState::new(pool.clone()));
        let deletions = Deletions::new(client, data, Some(Arc::new(store.clone())), Some(state), Duration::ZERO)
            .with_holds(Arc::new(store));
        self.oidc = self.oidc.clone().with_erased(Arc::new(deletions.clone()));
        self.deletions = Some(deletions);
    }
}

fn deps(store: &pg::Store, sessions: &Manager, http: &HttpClient) -> oidc::Deps {
    oidc::Deps {
        users: Pictures::wrap(store.users()),
        identities: store.identities(),
        codes: store.login_codes(),
        sessions: sessions.clone(),
        config_extra: None,
        http: Some(http.clone()),
        now: None,
    }
}

/// The kit `POST /api/renders` provisions accounts through. Registre mints a token exchanged for
/// a plain app audience with `email` only when the address is verified, and with no
/// `email_verified` claim (it adds the profile claims only for an MCP resource audience: see
/// `GetPrivateClaimsFromTokenExchangeRequest` in Registre's `modules/oidc/storage_exchange.go`).
/// So this kit trusts an email that comes without the claim; porte still refuses an explicit
/// `false`, and erased subjects as the sign-in kit does. Sign-in and `/mcp` keep the strict kit.
async fn delegated(cfg: &Config, store: &pg::Store, kits: &Kits, http: &HttpClient) -> Result<Kit, BootError> {
    let trusting = porte::Config { trust_email_without_verified_claim: true, ..cfg.porte.clone() };
    let kit = Kit::new(&trusting, deps(store, &kits.sessions, http)).await?;
    Ok(match kits.deletions.clone() {
        Some(deletions) => kit.with_erased(Arc::new(deletions)),
        None => kit,
    })
}

/// The resource a bearer of `POST /api/renders` may name besides the app's audience: the MCP
/// resource, or the URL it would have under the sign-in redirect's origin.
fn machine_resource(cfg: &Config) -> String {
    if !cfg.mcp_resource.is_empty() {
        return cfg.mcp_resource.clone();
    }
    let origin = url::Url::parse(&cfg.porte.redirect_url).map(|url| url.origin().ascii_serialization());
    format!("{}/mcp", origin.unwrap_or_else(|_| cfg.porte.issuer.clone()))
}

type Checked = (String, HttpClient);

async fn verifier(
    cfg: &Config,
    store: &pg::Store,
    oidc: &Kit,
    checked: Checked,
) -> Result<mcpauth::Verifier, BootError> {
    let audience = match cfg.porte.machine_audience.is_empty() {
        true => cfg.porte.client_id.clone(),
        false => cfg.porte.machine_audience.clone(),
    };
    let config = mcpauth::Config {
        issuer: cfg.porte.issuer.clone(),
        resource: checked.0,
        scope_prefix: crate::env::SCOPE_PREFIX.to_owned(),
        audience,
        identities: store.identities(),
        provision: true,
        accounts: Some(oidc.clone()),
        http: checked.1,
    };
    Ok(mcpauth::Verifier::new(config).await?)
}
