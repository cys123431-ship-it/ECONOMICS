use std::{collections::HashMap, env, fs, path::PathBuf};

pub const CANONICAL_RULEBOOK_NAME: &str = "Market_Economy_Radar_Rulebook_v4_ULTRA.txt";

#[derive(Clone, Debug)]
pub struct Config {
    pub fred_api_key: Option<String>,
    pub ecos_api_key: Option<String>,
    pub krx_api_key: Option<String>,
    pub official_adapters_file: Option<PathBuf>,
    pub db_path: PathBuf,
    pub rulebook_path: PathBuf,
    pub host: String,
    pub min_samples: usize,
    pub http_timeout_secs: u64,
    pub krx_lookback_days: usize,
    pub crypto_refresh_seconds: u64,
    pub refresh_minutes: u64,
    pub macro_refresh_minutes: u64,
    pub full_refresh_hours: u64,
}

fn strip_inline_comment(value: &str) -> &str {
    let mut quote = None;
    for (idx, ch) in value.char_indices() {
        match ch {
            '\'' | '"' if quote == Some(ch) => quote = None,
            '\'' | '"' if quote.is_none() => quote = Some(ch),
            '#' if quote.is_none()
                && idx > 0
                && value[..idx]
                    .chars()
                    .next_back()
                    .is_some_and(char::is_whitespace) =>
            {
                return value[..idx].trim_end();
            }
            _ => {}
        }
    }
    value
}

fn parse_dotenv(text: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for raw in text.lines() {
        let mut line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("export ") {
            line = rest.trim_start();
        }
        if let Some((key, raw_value)) = line.split_once('=') {
            let value = strip_inline_comment(raw_value.trim())
                .trim_matches(['"', '\''])
                .to_string();
            map.insert(key.trim().to_string(), value);
        }
    }
    map
}

fn load_dotenv() -> HashMap<String, String> {
    fs::read_to_string(".env")
        .map(|text| parse_dotenv(&text))
        .unwrap_or_default()
}

fn value(name: &str, file: &HashMap<String, String>) -> Option<String> {
    env::var(name)
        .ok()
        .or_else(|| file.get(name).cloned())
        .filter(|value| !value.trim().is_empty())
}

fn default_rulebook_path() -> PathBuf {
    let local = PathBuf::from("rulebook").join(CANONICAL_RULEBOOK_NAME);
    if local.is_file() {
        return local;
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            let packaged = parent.join("rulebook").join(CANONICAL_RULEBOOK_NAME);
            if packaged.is_file() {
                return packaged;
            }
        }
    }
    local
}

impl Config {
    pub fn load() -> Self {
        let file = load_dotenv();
        Self {
            fred_api_key: value("FRED_API_KEY", &file),
            ecos_api_key: value("ECOS_API_KEY", &file),
            krx_api_key: value("KRX_API_KEY", &file),
            official_adapters_file: value("OFFICIAL_ADAPTERS_FILE", &file).map(PathBuf::from),
            db_path: PathBuf::from(
                value("ECONOMICS_DB", &file).unwrap_or_else(|| "runtime/economics.db".into()),
            ),
            rulebook_path: value("ECONOMICS_RULEBOOK", &file)
                .map(PathBuf::from)
                .unwrap_or_else(default_rulebook_path),
            host: value("ECONOMICS_HOST", &file).unwrap_or_else(|| "127.0.0.1:8765".into()),
            min_samples: value("ECONOMICS_MIN_SAMPLES", &file)
                .and_then(|value| value.parse().ok())
                .unwrap_or(20),
            http_timeout_secs: value("ECONOMICS_HTTP_TIMEOUT_SECS", &file)
                .and_then(|value| value.parse().ok())
                .unwrap_or(30),
            krx_lookback_days: value("ECONOMICS_KRX_LOOKBACK_DAYS", &file)
                .and_then(|value| value.parse().ok())
                .unwrap_or(60)
                .clamp(20, 365),
            crypto_refresh_seconds: value("ECONOMICS_CRYPTO_REFRESH_SECONDS", &file)
                .and_then(|value| value.parse().ok())
                .unwrap_or(30)
                .clamp(15, 300),
            refresh_minutes: value("ECONOMICS_REFRESH_MINUTES", &file)
                .and_then(|value| value.parse().ok())
                .unwrap_or(5)
                .clamp(1, 60),
            macro_refresh_minutes: value("ECONOMICS_MACRO_REFRESH_MINUTES", &file)
                .and_then(|value| value.parse().ok())
                .unwrap_or(30)
                .clamp(5, 360),
            full_refresh_hours: value("ECONOMICS_FULL_REFRESH_HOURS", &file)
                .and_then(|value| value.parse().ok())
                .unwrap_or(6)
                .clamp(1, 168),
        }
    }

    pub fn print_key_status(&self) {
        for (name, present) in [
            ("FRED_API_KEY", self.fred_api_key.is_some()),
            ("ECOS_API_KEY", self.ecos_api_key.is_some()),
            ("KRX_API_KEY", self.krx_api_key.is_some()),
        ] {
            println!("{name}: {}", if present { "configured" } else { "missing" });
        }
        println!("rulebook: {}", self.rulebook_path.display());
        println!(
            "refresh: crypto={}s market={}m macro={}m full={}h",
            self.crypto_refresh_seconds,
            self.refresh_minutes,
            self.macro_refresh_minutes,
            self.full_refresh_hours
        );
        println!(
            "official adapters: {}",
            self.official_adapters_file
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "not configured".into())
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dotenv_supports_quotes_export_and_comments() {
        let values = parse_dotenv(
            "# comment\nexport A=one\nB=\"two words\" # note\nC='hash#inside'\nEMPTY=\n",
        );
        assert_eq!(values.get("A").map(String::as_str), Some("one"));
        assert_eq!(values.get("B").map(String::as_str), Some("two words"));
        assert_eq!(values.get("C").map(String::as_str), Some("hash#inside"));
    }
}
