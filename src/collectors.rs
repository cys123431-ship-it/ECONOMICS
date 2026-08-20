use crate::{config::Config, db::Db};
use reqwest::blocking::Client;
use serde_json::Value;
use std::{error::Error, time::Duration};

fn client() -> Result<Client, Box<dyn Error>> {
    Ok(Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("ECONOMICS-Radar/0.2")
        .build()?)
}

const FRED_SERIES: &[&str] = &[
    "WEI",
    "CFNAI",
    "SAHMREALTIME",
    "ICSA",
    "CCSA",
    "STLFSI4",
    "NFCI",
    "ANFCI",
    "NFCILEVERAGE",
    "BAMLH0A0HYM2",
    "T10Y2Y",
    "VIXCLS",
    "WALCL",
    "RRPONTSYD",
    "DEXKOUS",
    "KORLOLITOAASTSAM",
    "CHNLOLITOAASTSAM",
];

pub fn collect_fred(
    cfg: &Config,
    db: &Db,
    start: &str,
    initial_release: bool,
) -> Result<usize, Box<dyn Error>> {
    let key = cfg.fred_api_key.as_ref().ok_or("FRED_API_KEY missing")?;
    let http = client()?;
    let mut total = 0usize;

    for series in FRED_SERIES {
        let mut url = format!(
            "https://api.stlouisfed.org/fred/series/observations?series_id={}&api_key={}&file_type=json&observation_start={}",
            urlencoding::encode(series),
            urlencoding::encode(key),
            urlencoding::encode(start)
        );
        if initial_release {
            url.push_str("&output_type=4");
        }
        let value: Value = http.get(&url).send()?.error_for_status()?.json()?;
        if let Some(observations) = value.get("observations").and_then(Value::as_array) {
            for observation in observations {
                let date = observation
                    .get("date")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let parsed = observation
                    .get("value")
                    .and_then(Value::as_str)
                    .and_then(|s| s.parse::<f64>().ok());
                if let Some(number) = parsed {
                    db.put(
                        if initial_release { "alfred" } else { "fred" },
                        series,
                        date,
                        number,
                        if initial_release {
                            observation.get("realtime_start").and_then(Value::as_str)
                        } else {
                            None
                        },
                    )?;
                    total += 1;
                }
            }
        }
    }
    Ok(total)
}

pub fn collect_binance(cfg: &Config, db: &Db) -> Result<usize, Box<dyn Error>> {
    let http = client()?;
    let mut stored = 0usize;
    let today = chrono::Utc::now().date_naive().to_string();

    let oi: Value = http
        .get("https://fapi.binance.com/fapi/v1/openInterest?symbol=BTCUSDT")
        .send()?
        .error_for_status()?
        .json()?;
    if let Some(number) = oi
        .get("openInterest")
        .and_then(Value::as_str)
        .and_then(|s| s.parse::<f64>().ok())
    {
        db.put("binance", "BTC_OI", &today, number, None)?;
        stored += 1;
    }

    let funding: Value = http
        .get("https://fapi.binance.com/fapi/v1/premiumIndex?symbol=BTCUSDT")
        .send()?
        .error_for_status()?
        .json()?;
    if let Some(number) = funding
        .get("lastFundingRate")
        .and_then(Value::as_str)
        .and_then(|s| s.parse::<f64>().ok())
    {
        db.put("binance", "BTC_FUNDING", &today, number, None)?;
        stored += 1;
    }

    // Public market-data endpoints do not require a key. Keep the optional key
    // available for approved read-only endpoints without ever printing it.
    let _ = cfg.binance_api_key.as_ref();
    Ok(stored)
}

pub fn collect_treasury(db: &Db) -> Result<usize, Box<dyn Error>> {
    let http = client()?;
    let url = "https://api.fiscaldata.treasury.gov/services/api/fiscal_service/v1/accounting/od/auctions_query?sort=-auction_date&page%5Bsize%5D=20";
    let value: Value = http.get(url).send()?.error_for_status()?.json()?;
    let mut stored = 0usize;

    if let Some(rows) = value.get("data").and_then(Value::as_array) {
        for row in rows {
            let date = row
                .get("auction_date")
                .and_then(Value::as_str)
                .unwrap_or("");
            for (series, key) in [
                ("AUCTION_BTC", "bid_to_cover_ratio"),
                ("AUCTION_DEALER_ACCEPTED", "primary_dealer_accepted"),
            ] {
                if let Some(number) = row
                    .get(key)
                    .and_then(Value::as_str)
                    .and_then(|s| s.replace(',', "").parse::<f64>().ok())
                {
                    db.put("treasury", series, date, number, None)?;
                    stored += 1;
                }
            }
        }
    }
    Ok(stored)
}

pub fn collect_ecos(cfg: &Config, db: &Db) -> Result<usize, Box<dyn Error>> {
    let key = cfg.ecos_api_key.as_ref().ok_or("ECOS_API_KEY missing")?;
    let http = client()?;
    let mut stored = 0usize;

    for (series, stat, item) in [
        ("KR_BASE_RATE", "722Y001", "0101000"),
        ("KR_USD_KRW", "731Y001", "0000001"),
    ] {
        let end = chrono::Utc::now().format("%Y%m%d").to_string();
        let url = format!(
            "https://ecos.bok.or.kr/api/StatisticSearch/{}/json/kr/1/100/{}/D/20000101/{}/{}",
            urlencoding::encode(key),
            stat,
            end,
            item
        );
        let value: Value = http.get(&url).send()?.error_for_status()?.json()?;
        if let Some(rows) = value.pointer("/StatisticSearch/row").and_then(Value::as_array) {
            for row in rows {
                if let (Some(date), Some(number)) = (
                    row.get("TIME").and_then(Value::as_str),
                    row.get("DATA_VALUE")
                        .and_then(Value::as_str)
                        .and_then(|s| s.parse::<f64>().ok()),
                ) {
                    db.put("ecos", series, date, number, None)?;
                    stored += 1;
                }
            }
        }
    }
    Ok(stored)
}

pub fn collect_krx(cfg: &Config, db: &Db) -> Result<usize, Box<dyn Error>> {
    let Some(url) = cfg.krx_api_url.as_ref() else {
        return Ok(0);
    };
    let key = cfg.krx_api_key.as_ref().ok_or("KRX_API_KEY missing")?;
    let text = client()?
        .get(url)
        .header("AUTH_KEY", key)
        .send()?
        .error_for_status()?
        .text()?;
    db.put(
        "krx",
        "KRX_PAYLOAD_BYTES",
        &chrono::Utc::now().date_naive().to_string(),
        text.len() as f64,
        None,
    )?;
    Ok(1)
}
