use std::{
    env,
    sync::Arc,
    time::{Duration, Instant},
};

use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
#[cfg(test)]
use serde::Serialize;
use tokio::sync::RwLock;

const DEFAULT_TENANT_ID: &str = "35c6fe40-0ec0-46b6-98c6-213ad4de6650";
const DEFAULT_SUBDOMAIN: &str = "sociobotcustomers";
const DEFAULT_CLIENT_ID: &str = "25c704f4-465a-47af-80ab-2c489466b697";

#[cfg(test)]
const TEST_CIAM_ISSUER: &str =
    "https://sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650/v2.0";
#[cfg(test)]
const TEST_CIAM_KID: &str = "stock-promise-test-key";
#[cfg(test)]
const TEST_CIAM_JWK_N: &str = "nuwh7mS3Rn3BLvOqJwf3jWD-0f5OYIy6JMPsfCIRYi-LXiQz7NYouUcS8-Re731fqh2oox_n4Oizoor0AyZIeQ_ejR_KRbCSCtNS3sfuqvi-wneUD_dDdLjWMFwMjzqHoc-PIdjIcfVrOQj5v0_c9p4lAKmHxik2jg38yXyIOAAWP8oJCpFU0uAdlHh7tvTm6KDXz2oLBgCvNeoB9ywE9FB5hAPJXjzeE6jlpIRRNNGw9HBNBaFhEBGKsIQB5Pw5YBUYEAYwy2wJpiStj4tSjDikxbik9crYSX-iPbgFwOiSvpCL6ulG-LO3YenHX5ElQRI6kUlR4KaSsk0r5NXDjw";
#[cfg(test)]
const TEST_CIAM_PRIVATE_KEY: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvwIBADANBgkqhkiG9w0BAQEFAASCBKkwggSlAgEAAoIBAQCe7CHuZLdGfcEu
86onB/eNYP7R/k5gjLokw+x8IhFiL4teJDPs1ii5RxLz5F7vfV+qHaijH+fg6LOi
ivQDJkh5D96NH8pFsJIK01Lex+6q+L7Cd5QP90N0uNYwXAyPOoehz48h2Mhx9Ws5
CPm/T9z2niUAqYfGKTaODfzJfIg4ABY/ygkKkVTS4B2UeHu29ObooNfPagsGAK81
6gH3LAT0UHmEA8lePN4TqOWkhFE00bD0cE0FoWEQEYqwhAHk/DlgFRgQBjDLbAmm
JK2Pi1KMOKTFuKT1ythJf6I9uAXA6JK+kIvq6Ub4s7dh6cdfkSVBEjqRSVHgppKy
TSvk1cOPAgMBAAECggEAKOzsTAT5wZhu4FV1cvm1QV5CIPfZXTnAUel2g/0N9vgB
buvpx0rbT+NCuTYNUKU7toZOwDteLmgeonQgJJN+RFbW3AbcwxeWdF05UQ+xoavP
DCtZIMdvQqa62ddKkLPk68GUvYWjEN2/j/+EuUSqxYOySbpesaQ5XVmyyHxeY/UR
4bOlF9snJVItoUeO0Z6wxdAHo3P5LdZc7M32SZrRnnbKLs02tIem8dbBFoyYTDlL
cWk6R8IFF+D0qlVv2DPfoKbD0rZJznTneInZw70HkXkkk9qblMr+pXa2CdSmckWz
CUo00IDFXKtNNvis82gB44Hk+/IciU9hJgEAAOKO2QKBgQDLl8jxrfxfiQmSwcHz
R3110mRAUWKxrucnyIJVWiF8fWRn4UY04U6QqjAZ5obHg3TpCMqUZ6APFm/q6hNq
gR1kRFzDb+JSizdud+Sk6n++Bu1B2hlhT4qNjXtB+14UGLIj+BY0cxl691y/Ee0g
Ao4Io1EdgRJOOwFv3qhpf81ULQKBgQDH1K7R7XAg+125r+IUlx/pwqktPon4nPU7
9g9z1DFFLCZpnp+0lq6C53OQVmDwmWMYm0oMB4dCzt7SMOvwgs2t635CETCWNDYn
iz5n+GJuRejOGgE7QU+Le2yEDH4SQWm6Z+2ydBoIsCHd1tJw2KL7646nyEi5TKrg
wwQ+0v8gKwKBgQChe90oxQXeiQWM4B6EZnn+0EgwM92CbeZvSb0HEhxpb9QKCUb0
fpkgab0Jbi4NZhl/FTgJikAMd5GB1PRdj9fOReMGKWJscnwfJP24ST/VbA0YJHPp
GXweVtAjP8wZSJVTrl1+cKUPQdDQxNk+gIhEFbYrHst0PZ0gLI2MUJB+aQKBgQC0
NnMecRIhPG/vCNZLmWq3Zs0pN3A4HFzxIVwYKnHwnvtZXytKMmXZOiA3OB8efEYp
J8qkhJmQP34lcucktOIGig0ISfZWT2nTSbkTDKbAKh3k2QDpTmINOVbI03dSwVk3
OYjc2eWsliNEq/qGuGhr5qh1WaN0MNcd8eG/QovAlwKBgQCEjPxAil0E//212Vv6
PIAV9qfb7zRTWhimdEawLsyYIkWfUF1QacJOVpbs9N4RPnCMi3Xru+fZvUfB45p0
OmNW6N48TPf8kDlbr/wHF0R9LRA/fJ4+d5JLAyNl0X9aN+AxZ/96OPfs8t0Ccl2y
6vuG/kj/nZu/XGpZRRZPMb787w==
-----END PRIVATE KEY-----"#;

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

    #[cfg(test)]
    pub fn ciam_for_tests() -> Self {
        Self {
            mode: AuthMode::Ciam,
            tenant_id: DEFAULT_TENANT_ID.into(),
            client_id: DEFAULT_CLIENT_ID.into(),
            discovery_url: "test://stock-promise-ciam".into(),
            client: reqwest::Client::new(),
            cached: Arc::new(RwLock::new(Some(CachedKeys {
                issuer: TEST_CIAM_ISSUER.into(),
                keys: vec![Jwk {
                    kid: TEST_CIAM_KID.into(),
                    kty: "RSA".into(),
                    n: TEST_CIAM_JWK_N.into(),
                    e: "AQAB".into(),
                }],
                fetched_at: Instant::now(),
            }))),
        }
    }

    #[cfg(test)]
    pub fn test_bearer(tenant_id: &str, roles: &[&str]) -> String {
        #[derive(Serialize)]
        struct TestClaims<'a> {
            aud: &'a str,
            exp: usize,
            iat: usize,
            iss: &'a str,
            oid: &'a str,
            roles: Vec<&'a str>,
            tid: &'a str,
        }

        let mut header = jsonwebtoken::Header::new(Algorithm::RS256);
        header.kid = Some(TEST_CIAM_KID.into());
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_secs() as usize;
        jsonwebtoken::encode(
            &header,
            &TestClaims {
                aud: DEFAULT_CLIENT_ID,
                exp: now + 3600,
                iat: now,
                iss: TEST_CIAM_ISSUER,
                oid: "stock-promise-test-user",
                roles: roles.to_vec(),
                tid: tenant_id,
            },
            &jsonwebtoken::EncodingKey::from_rsa_pem(TEST_CIAM_PRIVATE_KEY.as_bytes())
                .expect("test signing key is valid"),
        )
        .expect("test token is signed")
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

    // @claim:hosted-access
    #[test]
    fn claim_hosted_access_uses_sociobot_ciam_by_default() {
        let service = AuthService::from_env();
        assert!(service
            .discovery_url
            .contains("sociobotcustomers.ciamlogin.com"));
        assert_eq!(service.tenant_id, DEFAULT_TENANT_ID);
        assert_eq!(service.client_id, DEFAULT_CLIENT_ID);
    }
}
