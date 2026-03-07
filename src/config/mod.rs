pub mod auth;

use url::Url;

const DEFAULT_BACKEND_URL: &str = "https://platform-api.aixplain.com";
const DEFAULT_MODELS_RUN_URL: &str = "https://models.aixplain.com/api/v2/execute";

/// Fully resolved configuration for a session, combining .env, env vars, config file, and CLI flags.
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub api_key: String,
    pub key_type: KeyType,
    pub backend_url: Url,
    pub models_run_url: Url,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyType {
    Team,
    Individual,
}

impl ResolvedConfig {
    pub fn auth_header_name(&self) -> &str {
        match self.key_type {
            KeyType::Team => "x-api-key",
            KeyType::Individual => "x-aixplain-key",
        }
    }

    pub fn upload_auth_value(&self) -> String {
        format!("token {}", self.api_key)
    }
}

/// Build a ResolvedConfig from the environment (.env files + env vars + optional CLI override).
pub fn resolve(api_key_override: Option<&str>) -> anyhow::Result<ResolvedConfig> {
    // Load .env from CWD — override existing env vars so .env always wins
    let _ = dotenvy::dotenv_override();

    let api_key = api_key_override
        .map(String::from)
        .or_else(|| std::env::var("AIXPLAIN_API_KEY").ok())
        .or_else(|| std::env::var("TEAM_API_KEY").ok())
        .ok_or_else(|| {
            anyhow::anyhow!("No API key found. Set AIXPLAIN_API_KEY or run `aix auth login`.")
        })?;

    let key_type = std::env::var("AIXPLAIN_KEY_TYPE")
        .ok()
        .and_then(|v| {
            if v.eq_ignore_ascii_case("individual") {
                Some(KeyType::Individual)
            } else {
                None
            }
        })
        .unwrap_or(KeyType::Team);

    let backend_url =
        std::env::var("BACKEND_URL").unwrap_or_else(|_| DEFAULT_BACKEND_URL.to_string());
    let backend_url = Url::parse(&backend_url)?;

    let models_run_url =
        std::env::var("MODELS_RUN_URL").unwrap_or_else(|_| DEFAULT_MODELS_RUN_URL.to_string());
    let models_run_url = Url::parse(&models_run_url)?;

    Ok(ResolvedConfig {
        api_key,
        key_type,
        backend_url,
        models_run_url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_header_team() {
        let cfg = ResolvedConfig {
            api_key: "test".into(),
            key_type: KeyType::Team,
            backend_url: Url::parse(DEFAULT_BACKEND_URL).unwrap(),
            models_run_url: Url::parse(DEFAULT_MODELS_RUN_URL).unwrap(),
        };
        assert_eq!(cfg.auth_header_name(), "x-api-key");
    }

    #[test]
    fn auth_header_individual() {
        let cfg = ResolvedConfig {
            api_key: "test".into(),
            key_type: KeyType::Individual,
            backend_url: Url::parse(DEFAULT_BACKEND_URL).unwrap(),
            models_run_url: Url::parse(DEFAULT_MODELS_RUN_URL).unwrap(),
        };
        assert_eq!(cfg.auth_header_name(), "x-aixplain-key");
    }

    #[test]
    fn upload_auth_format() {
        let cfg = ResolvedConfig {
            api_key: "abc123".into(),
            key_type: KeyType::Team,
            backend_url: Url::parse(DEFAULT_BACKEND_URL).unwrap(),
            models_run_url: Url::parse(DEFAULT_MODELS_RUN_URL).unwrap(),
        };
        assert_eq!(cfg.upload_auth_value(), "token abc123");
    }
}
