use std::{collections::HashMap, env, fs, path::PathBuf};

#[derive(Clone, Debug)]
pub struct Config {
    pub fred_api_key: Option<String>,
    pub ecos_api_key: Option<String>,
    pub krx_api_key: Option<String>,
    pub binance_api_key: Option<String>,
    pub krx_api_url: Option<String>,
    pub db_path: PathBuf,
    pub host: String,
}

fn load_dotenv() -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(text) = fs::read_to_string(".env") {
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                map.insert(k.trim().to_string(), v.trim().trim_matches('"').to_string());
            }
        }
    }
    map
}

fn value(name: &str, file: &HashMap<String, String>) -> Option<String> {
    env::var(name)
        .ok()
        .or_else(|| file.get(name).cloned())
        .filter(|s| !s.is_empty())
}

impl Config {
    pub fn load() -> Self {
        let file = load_dotenv();
        Self {
            fred_api_key: value("FRED_API_KEY", &file),
            ecos_api_key: value("ECOS_API_KEY", &file),
            krx_api_key: value("KRX_API_KEY", &file),
            binance_api_key: value("BINANCE_API_KEY", &file),
            krx_api_url: value("KRX_API_URL", &file),
            db_path: PathBuf::from(
                value("ECONOMICS_DB", &file).unwrap_or_else(|| "runtime/economics.db".into()),
            ),
            host: value("ECONOMICS_HOST", &file).unwrap_or_else(|| "127.0.0.1:8765".into()),
        }
    }

    pub fn print_key_status(&self) {
        for (name, present) in [
            ("FRED_API_KEY", self.fred_api_key.is_some()),
            ("ECOS_API_KEY", self.ecos_api_key.is_some()),
            ("KRX_API_KEY", self.krx_api_key.is_some()),
            ("BINANCE_API_KEY", self.binance_api_key.is_some()),
        ] {
            println!("{name}: {}", if present { "configured" } else { "missing" });
        }
        println!(
            "KRX_API_URL: {}",
            if self.krx_api_url.is_some() {
                "configured"
            } else {
                "missing"
            }
        );
    }
}
