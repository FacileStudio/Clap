use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use porte::http_client::{HttpClient, HttpRequest, HttpResponse, Transport, TransportError};
use rsa::PrivateKey;
use rsa::pkcs1v15::SigningKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::signature::{SignatureEncoding, Signer};
use rsa::traits::PublicKeyParts;
use serde_json::{Value, json};
use sha2::Sha256;

use super::ISSUER;

pub const CLIENT_ID: &str = "clap";
const KEY_ID: &str = "test-key";

pub const REGISTRE: [(&str, &str); 5] = [
    ("OIDC_ISSUER", ISSUER),
    ("OIDC_CLIENT_ID", CLIENT_ID),
    ("OIDC_CLIENT_SECRET", "test-secret"),
    ("OIDC_REDIRECT_URL", "https://clap.test/api/auth/oidc/callback"),
    ("OIDC_SUCCESS_URL", "https://clap.test/"),
];

fn key() -> PrivateKey {
    PrivateKey::from_pkcs8_pem(include_str!("../testdata/idp_test_key.pem")).unwrap()
}

fn answer(body: Value) -> HttpResponse {
    http::Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(body.to_string().into())
        .unwrap()
}

struct Idp;

#[async_trait::async_trait]
impl Transport for Idp {
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse, TransportError> {
        let url = request.uri().to_string();
        let encode = |value: &rsa::BoxedUint| URL_SAFE_NO_PAD.encode(value.to_be_bytes_trimmed_vartime());
        match url.strip_prefix(ISSUER) {
            Some("/.well-known/openid-configuration") => Ok(answer(json!({
                "issuer": ISSUER, "authorization_endpoint": format!("{ISSUER}/authorize"),
                "token_endpoint": format!("{ISSUER}/token"), "userinfo_endpoint": format!("{ISSUER}/userinfo"),
                "jwks_uri": format!("{ISSUER}/keys"), "response_types_supported": ["code"],
                "subject_types_supported": ["public"], "id_token_signing_alg_values_supported": ["RS256"],
                "scopes_supported": ["openid", "email", "profile", "offline_access"],
            }))),
            Some("/keys") => {
                let (n, e) = (encode(key().n().as_ref()), encode(key().e()));
                Ok(answer(
                    json!({"keys": [{"kty": "RSA", "alg": "RS256", "use": "sig", "kid": KEY_ID, "n": n, "e": e}]}),
                ))
            }
            _ => Err(TransportError(format!("no route to {url}"))),
        }
    }
}

pub fn client() -> HttpClient {
    HttpClient::from_transport(Arc::new(Idp))
}

pub fn token(subject: &str, extra: Value) -> String {
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let mut claims = json!({
        "iss": ISSUER, "sub": subject, "aud": CLIENT_ID, "iat": now, "exp": now + 600,
        "email": format!("{subject}@example.test"), "email_verified": true, "name": subject,
        "scope": "clap.write", "client_id": "diapo", "act": {"sub": "diapo"},
    });
    for (name, value) in extra.as_object().cloned().unwrap_or_default() {
        claims[name] = value;
    }
    let segment = |value: &Value| URL_SAFE_NO_PAD.encode(serde_json::to_vec(value).unwrap());
    let input = format!("{}.{}", segment(&json!({"alg": "RS256", "kid": KEY_ID, "typ": "JWT"})), segment(&claims));
    let signature = SigningKey::<Sha256>::new(key()).sign(input.as_bytes());
    format!("{input}.{}", URL_SAFE_NO_PAD.encode(signature.to_bytes()))
}
