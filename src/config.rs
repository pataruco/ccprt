#[derive(Clone, Debug)]
pub struct Config {
    pub app_id: u64,
    pub private_key: String,
    pub webhook_secret: String,
    pub port: u16,
    pub github_api_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Self::from_lookup(|key| std::env::var(key).ok())
    }

    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Self, String> {
        let required =
            |name: &str| get(name).ok_or_else(|| format!("missing required env var {name}"));
        Ok(Self {
            app_id: required("APP_ID")?
                .parse()
                .map_err(|_| "APP_ID must be a number".to_string())?,
            private_key: required("PRIVATE_KEY")?,
            webhook_secret: required("WEBHOOK_SECRET")?,
            port: get("PORT")
                .unwrap_or_else(|| "8080".to_string())
                .parse()
                .map_err(|_| "PORT must be a number".to_string())?,
            github_api_url: get("GITHUB_API_URL")
                .unwrap_or_else(|| "https://api.github.com".to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn vars(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn lookup(map: &HashMap<String, String>) -> impl Fn(&str) -> Option<String> + '_ {
        move |key| map.get(key).cloned()
    }

    #[test]
    fn loads_complete_config() {
        let map = vars(&[
            ("APP_ID", "12345"),
            ("PRIVATE_KEY", "-----BEGIN RSA PRIVATE KEY-----"),
            ("WEBHOOK_SECRET", "s3cret"),
            ("PORT", "9999"),
        ]);
        let config = Config::from_lookup(lookup(&map)).unwrap();
        assert_eq!(config.app_id, 12345);
        assert_eq!(config.port, 9999);
        assert_eq!(config.github_api_url, "https://api.github.com");
    }

    #[test]
    fn port_defaults_to_8080() {
        let map = vars(&[
            ("APP_ID", "1"),
            ("PRIVATE_KEY", "k"),
            ("WEBHOOK_SECRET", "s"),
        ]);
        assert_eq!(Config::from_lookup(lookup(&map)).unwrap().port, 8080);
    }

    #[test]
    fn missing_required_var_names_the_var() {
        let map = vars(&[("APP_ID", "1"), ("PRIVATE_KEY", "k")]);
        let err = Config::from_lookup(lookup(&map)).unwrap_err();
        assert!(err.contains("WEBHOOK_SECRET"), "error was: {err}");
    }

    #[test]
    fn non_numeric_app_id_is_an_error() {
        let map = vars(&[
            ("APP_ID", "abc"),
            ("PRIVATE_KEY", "k"),
            ("WEBHOOK_SECRET", "s"),
        ]);
        assert!(Config::from_lookup(lookup(&map)).is_err());
    }
}
