use crate::db::{Db, Point};
use serde::Serialize;

#[derive(Clone, Copy)]
struct SeriesRef {
    source: &'static str,
    series: &'static str,
}

#[derive(Clone, Copy)]
struct IndicatorDefinition {
    key: &'static str,
    label: &'static str,
    symbol: &'static str,
    market: &'static str,
    asset_class: &'static str,
    candidates: &'static [SeriesRef],
    unit: &'static str,
    decimals: u8,
    comparison_points: usize,
    change_period: &'static str,
}

#[derive(Debug, Serialize)]
pub struct DashboardIndicator {
    pub key: &'static str,
    pub label: &'static str,
    pub symbol: &'static str,
    pub market: &'static str,
    pub asset_class: &'static str,
    pub value: Option<f64>,
    pub change: Option<f64>,
    pub change_pct: Option<f64>,
    pub history: Vec<f64>,
    pub observed_at: Option<String>,
    pub source: Option<String>,
    pub unit: &'static str,
    pub decimals: u8,
    pub change_period: &'static str,
}

#[derive(Debug, Serialize)]
pub struct DashboardData {
    pub indicators: Vec<DashboardIndicator>,
}

const fn series(source: &'static str, name: &'static str) -> SeriesRef {
    SeriesRef {
        source,
        series: name,
    }
}

const FRED_SP500: &[SeriesRef] = &[series("fred", "SP500")];
const FRED_NASDAQ: &[SeriesRef] = &[series("fred", "NASDAQCOM")];
const FRED_DOW: &[SeriesRef] = &[series("fred", "DJIA")];
const FRED_VIX: &[SeriesRef] = &[series("fred", "VIXCLS")];
const FRED_DGS10: &[SeriesRef] = &[series("fred", "DGS10")];
const FRED_DGS2: &[SeriesRef] = &[series("fred", "DGS2")];
const FRED_CURVE: &[SeriesRef] = &[series("fred", "T10Y2Y")];
const FRED_HY: &[SeriesRef] = &[series("fred", "BAMLH0A0HYM2")];
const FRED_USD: &[SeriesRef] = &[series("fred", "DTWEXBGS")];
const TREASURY_BTC: &[SeriesRef] = &[series("treasury", "AUCTION_BTC")];
const USD_KRW: &[SeriesRef] = &[series("ecos", "KR_USD_KRW"), series("fred", "DEXKOUS")];
const KR_BASE_RATE: &[SeriesRef] = &[series("ecos", "KR_BASE_RATE")];
const KOSPI: &[SeriesRef] = &[series("krx", "KRX_KOSPI_CLOSE")];
const KOSDAQ: &[SeriesRef] = &[series("krx", "KRX_KOSDAQ_CLOSE")];
const KOSPI_BREADTH: &[SeriesRef] = &[series("krx", "KRX_KOSPI_BREADTH")];
const KOSDAQ_BREADTH: &[SeriesRef] = &[series("krx", "KRX_KOSDAQ_BREADTH")];
const KRX_BREADTH: &[SeriesRef] = &[series("krx", "KRX_BREADTH")];
const KRX_BASIS: &[SeriesRef] = &[series("krx", "KRX_BASIS")];
const KRX_FUTURES_OI: &[SeriesRef] = &[series("krx", "KRX_FUTURES_OI")];
const KRX_PUT_CALL: &[SeriesRef] = &[series("krx", "KRX_PUT_CALL")];
const KRX_OPTION_IV: &[SeriesRef] = &[series("krx", "KRX_OPTIONS_AVG_IMPLIED_VOL")];
const KRX_BOND_YIELD: &[SeriesRef] = &[series("krx", "KRX_BOND_INDEX_AVG_YIELD")];
const KRX_KTS_YIELD: &[SeriesRef] = &[series("krx", "KRX_KTS_BOND_AVG_YIELD")];
const BTC_PRICE: &[SeriesRef] = &[series("binance", "BTC_PRICE_USD")];
const BTC_FUNDING: &[SeriesRef] = &[
    series("binance", "BTC_FUNDING_RATE"),
    series("binance", "BTC_FUNDING_ABS"),
];
const BTC_OI: &[SeriesRef] = &[series("binance", "BTC_OI")];
const BTC_GLOBAL_LS: &[SeriesRef] = &[series("binance", "BTC_GLOBAL_LONG_SHORT")];
const BTC_TOP_POSITION: &[SeriesRef] = &[series("binance", "BTC_TOP_POSITION_RATIO")];
const BTC_TOP_ACCOUNT: &[SeriesRef] = &[series("binance", "BTC_TOP_ACCOUNT_RATIO")];
const BTC_TAKER: &[SeriesRef] = &[series("binance", "BTC_TAKER_RATIO")];
const BTC_BASIS: &[SeriesRef] = &[
    series("binance", "BTC_BASIS_RATE"),
    series("binance", "BTC_BASIS_ABS"),
];

const INDICATORS: &[IndicatorDefinition] = &[
    indicator(
        "sp500", "S&P 500", "SPX", "us", "stocks", FRED_SP500, "index", 2, 2, "1D",
    ),
    indicator(
        "nasdaq",
        "나스닥 종합",
        "NASDAQ",
        "us",
        "stocks",
        FRED_NASDAQ,
        "index",
        2,
        2,
        "1D",
    ),
    indicator(
        "dow",
        "다우존스",
        "DJIA",
        "us",
        "stocks",
        FRED_DOW,
        "index",
        2,
        2,
        "1D",
    ),
    indicator(
        "vix",
        "공포지수 VIX",
        "VIX",
        "us",
        "volatility",
        FRED_VIX,
        "index",
        2,
        2,
        "1D",
    ),
    indicator(
        "us10y",
        "미국 10년물",
        "US10Y",
        "us",
        "bonds",
        FRED_DGS10,
        "percent",
        2,
        2,
        "1D",
    ),
    indicator(
        "us2y",
        "미국 2년물",
        "US2Y",
        "us",
        "bonds",
        FRED_DGS2,
        "percent",
        2,
        2,
        "1D",
    ),
    indicator(
        "curve_10y2y",
        "10년-2년 금리차",
        "10Y2Y",
        "us",
        "bonds",
        FRED_CURVE,
        "percent",
        2,
        2,
        "1D",
    ),
    indicator(
        "hy_spread",
        "하이일드 스프레드",
        "HY OAS",
        "us",
        "credit",
        FRED_HY,
        "percent",
        2,
        2,
        "1D",
    ),
    indicator(
        "usd_index",
        "달러 지수",
        "USD",
        "us",
        "fx",
        FRED_USD,
        "index",
        2,
        2,
        "1D",
    ),
    indicator(
        "treasury_bid_cover",
        "미 국채 입찰 응찰률",
        "BTCVR",
        "us",
        "bonds",
        TREASURY_BTC,
        "ratio",
        2,
        2,
        "1D",
    ),
    indicator(
        "usdkrw",
        "원·달러 환율",
        "USD/KRW",
        "korea",
        "fx",
        USD_KRW,
        "krw",
        2,
        2,
        "1D",
    ),
    indicator(
        "kr_base_rate",
        "한국 기준금리",
        "BOK RATE",
        "korea",
        "rates",
        KR_BASE_RATE,
        "percent",
        2,
        2,
        "1D",
    ),
    indicator(
        "kospi",
        "코스피",
        "KOSPI",
        "korea",
        "stocks",
        KOSPI,
        "index",
        2,
        2,
        "1D",
    ),
    indicator(
        "kosdaq",
        "코스닥",
        "KOSDAQ",
        "korea",
        "stocks",
        KOSDAQ,
        "index",
        2,
        2,
        "1D",
    ),
    indicator(
        "kospi_breadth",
        "코스피 상승종목 비율",
        "KOSPI BR",
        "korea",
        "stocks",
        KOSPI_BREADTH,
        "percent",
        1,
        2,
        "1D",
    ),
    indicator(
        "kosdaq_breadth",
        "코스닥 상승종목 비율",
        "KOSDAQ BR",
        "korea",
        "stocks",
        KOSDAQ_BREADTH,
        "percent",
        1,
        2,
        "1D",
    ),
    indicator(
        "krx_breadth",
        "한국시장 종합 시장폭",
        "KRX BR",
        "korea",
        "stocks",
        KRX_BREADTH,
        "percent",
        1,
        2,
        "1D",
    ),
    indicator(
        "krx_basis",
        "코스피200 선물 베이시스",
        "K200 BASIS",
        "korea",
        "futures",
        KRX_BASIS,
        "points",
        2,
        2,
        "1D",
    ),
    indicator(
        "krx_futures_oi",
        "코스피200 선물 미결제약정",
        "K200 OI",
        "korea",
        "futures",
        KRX_FUTURES_OI,
        "contracts",
        0,
        2,
        "1D",
    ),
    indicator(
        "krx_put_call",
        "코스피200 옵션 풋/콜",
        "PUT/CALL",
        "korea",
        "options",
        KRX_PUT_CALL,
        "ratio",
        2,
        2,
        "1D",
    ),
    indicator(
        "krx_option_iv",
        "코스피200 옵션 내재변동성",
        "K200 IV",
        "korea",
        "options",
        KRX_OPTION_IV,
        "percent",
        2,
        2,
        "1D",
    ),
    indicator(
        "krx_bond_yield",
        "한국 채권지수 평균수익률",
        "KR BOND",
        "korea",
        "bonds",
        KRX_BOND_YIELD,
        "percent",
        2,
        2,
        "1D",
    ),
    indicator(
        "krx_kts_yield",
        "국채전문시장 평균수익률",
        "KTS YLD",
        "korea",
        "bonds",
        KRX_KTS_YIELD,
        "percent",
        2,
        2,
        "1D",
    ),
    indicator(
        "btc",
        "비트코인",
        "BTC/USD",
        "crypto",
        "spot",
        BTC_PRICE,
        "usd",
        0,
        25,
        "24H",
    ),
    indicator(
        "btc_funding",
        "비트코인 펀딩비",
        "FUNDING",
        "crypto",
        "futures",
        BTC_FUNDING,
        "rate",
        4,
        4,
        "3H",
    ),
    indicator(
        "btc_oi",
        "비트코인 미결제약정",
        "OPEN INT",
        "crypto",
        "futures",
        BTC_OI,
        "usd",
        0,
        25,
        "24H",
    ),
    indicator(
        "btc_global_ls",
        "전체 롱/숏 비율",
        "GLOBAL L/S",
        "crypto",
        "positioning",
        BTC_GLOBAL_LS,
        "ratio",
        2,
        25,
        "24H",
    ),
    indicator(
        "btc_top_position",
        "상위 포지션 롱/숏",
        "TOP POS",
        "crypto",
        "positioning",
        BTC_TOP_POSITION,
        "ratio",
        2,
        25,
        "24H",
    ),
    indicator(
        "btc_top_account",
        "상위 계정 롱/숏",
        "TOP ACCT",
        "crypto",
        "positioning",
        BTC_TOP_ACCOUNT,
        "ratio",
        2,
        25,
        "24H",
    ),
    indicator(
        "btc_taker",
        "테이커 매수/매도",
        "TAKER",
        "crypto",
        "flow",
        BTC_TAKER,
        "ratio",
        2,
        25,
        "24H",
    ),
    indicator(
        "btc_basis",
        "무기한 선물 베이시스",
        "BASIS",
        "crypto",
        "futures",
        BTC_BASIS,
        "rate",
        4,
        25,
        "24H",
    ),
];

#[allow(clippy::too_many_arguments)]
const fn indicator(
    key: &'static str,
    label: &'static str,
    symbol: &'static str,
    market: &'static str,
    asset_class: &'static str,
    candidates: &'static [SeriesRef],
    unit: &'static str,
    decimals: u8,
    comparison_points: usize,
    change_period: &'static str,
) -> IndicatorDefinition {
    IndicatorDefinition {
        key,
        label,
        symbol,
        market,
        asset_class,
        candidates,
        unit,
        decimals,
        comparison_points,
        change_period,
    }
}

pub fn build(db: &Db) -> rusqlite::Result<DashboardData> {
    let mut indicators = Vec::with_capacity(INDICATORS.len());
    for definition in INDICATORS {
        indicators.push(read_indicator(db, definition)?);
    }
    Ok(DashboardData { indicators })
}

fn read_indicator(
    db: &Db,
    definition: &IndicatorDefinition,
) -> rusqlite::Result<DashboardIndicator> {
    let mut selected: Option<(SeriesRef, Vec<Point>)> = None;
    for candidate in definition.candidates {
        let points = db.recent(
            candidate.source,
            candidate.series,
            definition.comparison_points.max(30),
            None,
        )?;
        if !points.is_empty() {
            selected = Some((*candidate, points));
            break;
        }
    }
    let (source, points) = selected
        .map(|(candidate, points)| (Some(candidate.source.to_uppercase()), points))
        .unwrap_or_default();
    let latest = points.last();
    let comparison = (points.len() >= definition.comparison_points)
        .then(|| &points[points.len() - definition.comparison_points]);
    let change = latest
        .zip(comparison)
        .map(|(latest, prior)| latest.value - prior.value);
    let change_pct = latest.zip(comparison).and_then(|(latest, prior)| {
        (prior.value.abs() > f64::EPSILON)
            .then(|| 100.0 * (latest.value - prior.value) / prior.value)
    });
    Ok(DashboardIndicator {
        key: definition.key,
        label: definition.label,
        symbol: definition.symbol,
        market: definition.market,
        asset_class: definition.asset_class,
        value: latest.map(|point| point.value),
        change,
        change_pct,
        history: points.iter().map(|point| point.value).collect(),
        observed_at: latest.map(|point| point.observed_at.clone()),
        source,
        unit: definition.unit,
        decimals: definition.decimals,
        change_period: definition.change_period,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::NewObservation;

    #[test]
    fn dashboard_preserves_missing_values_and_calculates_changes() {
        let temporary = tempfile::tempdir().unwrap();
        let db = Db::open(&temporary.path().join("dashboard.db")).unwrap();
        db.put(&NewObservation::simple(
            "fred",
            "SP500",
            "2026-08-19",
            6_400.0,
        ))
        .unwrap();
        db.put(&NewObservation::simple(
            "fred",
            "SP500",
            "2026-08-20",
            6_464.0,
        ))
        .unwrap();
        let dashboard = build(&db).unwrap();
        let sp500 = dashboard
            .indicators
            .iter()
            .find(|indicator| indicator.key == "sp500")
            .unwrap();
        assert_eq!(sp500.value, Some(6_464.0));
        assert_eq!(sp500.change, Some(64.0));
        assert_eq!(sp500.change_pct, Some(1.0));
        let bitcoin = dashboard
            .indicators
            .iter()
            .find(|indicator| indicator.key == "btc")
            .unwrap();
        assert_eq!(bitcoin.value, None);
        assert_eq!(bitcoin.change, None);
    }
}
