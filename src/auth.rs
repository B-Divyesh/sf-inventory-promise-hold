use std::{
    env,
    sync::Arc,
    time::{Duration, Instant},
};

use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use tokio::sync::RwLock;

const DEFAULT_TENANT_ID: &str = "35c6fe40-0ec0-46b6-98c6-213ad4de6650";
const DEFAULT_SUBDOMAIN: &str = "sociobotcustomers";
const DEFAULT_CLIENT_ID: &str = "25c704f4-465a-47af-80ab-2c489466b697";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    Staff,
    Supervisor,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Staff => "staff",
            Self::Supervisor => "supervisor",
        }
    }

    pub fn allows(self, required: Self) -> bool {
        self == Self::Supervisor || required == Self::Staff
    }
}

#[derive(Clone, Debug)]
pub struct Principal {
    pub oid: String,
    pub role: Role,
}

#[derive(Clone, Debug)]
pub enum AuthMode {
    Local,
    Ciam,
}

impl AuthMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Ciam => "ciam",
        }
    }
}

#[derive(Clone)]
pub struct AuthService {
    pub mode: AuthMode,
    tenant_id: String,
    client_id: String,
    discovery_url: String,
    client: reqwest::Client,
    cached: Arc<RwLock<Option<CachedKeys>>>,
}

#[derive(Clone)]
struct CachedKeys {
    issuer: String,
    keys: Vec<Jwk>,
    fetched_at: Instant,
}

#[derive(Deserialize)]
struct Discovery {
    issuer: String,
    jwks_uri: String,
}

#[derive(Clone, Deserialize)]
struct Jwk {
    kid: String,
    kty: String,
    n: String,
    e: String,
}

#[derive(Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Deserialize)]
struct Claims {
    oid: String,
    tid: String,
    #[serde(default)]
    roles: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Sign in is required to use the live promise desk.")]
    Missing,
    #[error("Your sign-in token is not valid. Sign in again.")]
    Invalid,
    #[error("Your account needs the Stock Promise staff or supervisor role.")]
    RoleMissing,
    #[error("Sign-in verification is temporarily unavailable. Try again.")]
    Unavailable,
}

impl AuthService {
    pub fn from_env() -> Self {
        let tenant_id = env::var("ENTRA_TENANT_ID").unwrap_or_else(|_| DEFAULT_TENANT_ID.into());
        let subdomain =
            env::var("ENTRA_TENANT_SUBDOMAIN").unwrap_or_else(|_| DEFAULT_SUBDOMAIN.into());
        let client_id = env::var("ENTRA_CLIENT_ID").unwrap_or_else(|_| DEFAULT_CLIENT_ID.into());
        let mode = if env::var("AUTH_MODE").ok().as_deref() == Some("local") {
            AuthMode::Local
        } else {
            AuthMode::Ciam
        };
        let discovery_url = format!(
            "https://{subdomain}.ciamlogin.com/{tenant_id}/v2.0/.well-known/openid-configuration"
        );
        Self {
            mode,
            tenant_id,
            client_id,
            discovery_url,
            client: reqwest::Client::new(),
            cached: Arc::new(RwLock::new(None)),
        }
    }

    pub fn local_for_tests() -> Self {
        let mut service = Self::from_env();
        service.mode = AuthMode::Local;
        service
    }

    pub async fn validate(&self, token: Option<&str>) -> Result<Principal, AuthError> {
        if matches!(self.mode, AuthMode::Local) {
            return Err(AuthError::Invalid);
        }
        let token = token.ok_or(AuthError::Missing)?;
        let header = decode_header(token).map_err(|_| AuthError::Invalid)?;
        let kid = header.kid.ok_or(AuthError::Invalid)?;
        let keys = self.keys().await?;
        let key = keys
            .keys
            .iter()
            .find(|key| key.kid == kid && key.kty == "RSA")
            .ok_or(AuthError::Invalid)?;
        let decoding_key =
            DecodingKey::from_rsa_components(&key.n, &key.e).map_err(|_| AuthError::Invalid)?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[self.client_id.as_str()]);
        validation.set_issuer(&[keys.issuer.as_str()]);
        let claims = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|_| AuthError::Invalid)?
            .claims;
        if claims.tid != self.tenant_id || claims.oid.trim().is_empty() {
            return Err(AuthError::Invalid);
        }
        let role = claims
            .roles
            .iter()
            .find_map(|role| match role.to_ascii_lowercase().as_str() {
                "stockpromise.supervisor" | "stock_promise_supervisor" | "supervisor" => {
                    Some(Role::Supervisor)
                }
                "stockpromise.staff" | "stock_promise_staff" | "staff" => Some(Role::Staff),
                _ => None,
            })
            .ok_or(AuthError::RoleMissing)?;
        Ok(Principal {
            oid: claims.oid,
            role,
        })
    }

    async fn keys(&self) -> Result<CachedKeys, AuthError> {
        if let Some(cached) = self.cached.read().await.clone() {
            if cached.fetched_at.elapsed() < Duration::from_secs(3600) {
                return Ok(cached);
            }
        }
        let discovery = self
            .client
            .get(&self.discovery_url)
            .send()
            .await
            .map_err(|_| AuthError::Unavailable)?
            .error_for_status()
            .map_err(|_| AuthError::Unavailable)?
            .json::<Discovery>()
            .await
            .map_err(|_| AuthError::Unavailable)?;
        let jwks = self
            .client
            .get(discovery.jwks_uri)
            .send()
            .await
            .map_err(|_| AuthError::Unavailable)?
            .error_for_status()
            .map_err(|_| AuthError::Unavailable)?
            .json::<Jwks>()
            .await
            .map_err(|_| AuthError::Unavailable)?;
        let cached = CachedKeys {
            issuer: discovery.issuer,
            keys: jwks.keys,
            fetched_at: Instant::now(),
        };
        *self.cached.write().await = Some(cached.clone());
        Ok(cached)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_defaults_use_the_shared_ciam_tenant() {
        let service = AuthService::from_env();
        assert!(service
            .discovery_url
            .contains("sociobotcustomers.ciamlogin.com"));
        assert_eq!(service.tenant_id, DEFAULT_TENANT_ID);
        assert_eq!(service.client_id, DEFAULT_CLIENT_ID);
    }
}
