use crate::db::{Db, Point};
use chrono::{DateTime, NaiveDate, Utc};
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

#[derive(Clone, Debug, Serialize)]
pub struct DashboardHistoryPoint {
    pub observed_at: String,
    pub value: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct DashboardIndicator {
    pub key: &'static str,
    pub label: &'static str,
    pub symbol: &'static str,
    pub market: &'static str,
    pub asset_class: &'static str,
    pub value: Option<f64>,
    pub raw_value: Option<f64>,
    pub previous_value: Option<f64>,
    pub change: Option<f64>,
    pub change_pct: Option<f64>,
    pub history: Vec<f64>,
    pub history_points: Vec<DashboardHistoryPoint>,
    pub history_low: Option<f64>,
    pub history_high: Option<f64>,
    pub history_average: Option<f64>,
    pub range_position: Option<f64>,
    pub observations: usize,
    pub observed_at: Option<String>,
    pub ingested_at: Option<String>,
    pub source: Option<String>,
    pub series: Option<String>,
    pub source_series: Option<String>,
    pub freshness: String,
    pub cadence: &'static str,
    pub unit: &'static str,
    pub decimals: u8,
    pub change_period: &'static str,
}

#[derive(Clone, Debug, Serialize)]
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
const BTC_SPOT_PRICE: &[SeriesRef] = &[series("binance", "BTC_SPOT_PRICE_USD")];
const BTC_PERP_PRICE: &[SeriesRef] = &[series("binance", "BTC_PRICE_USD")];

const FRED_IG: &[SeriesRef] = &[series("fred", "BAMLC0A0CM")];
const FRED_CURVE_3M: &[SeriesRef] = &[series("fred", "T10Y3M")];
const OFR_FSI: &[SeriesRef] = &[series("ofr_fsi", "OFR_FSI")];
const FRED_STLFSI: &[SeriesRef] = &[series("fred", "STLFSI4")];
const FRED_NFCI: &[SeriesRef] = &[series("fred", "NFCI")];
const FRED_ANFCI: &[SeriesRef] = &[series("fred", "ANFCI")];
const FRED_NFCI_LEVERAGE: &[SeriesRef] = &[series("fred", "NFCILEVERAGE")];
const FRED_WEI: &[SeriesRef] = &[series("fred", "WEI")];
const FRED_CFNAI: &[SeriesRef] = &[series("fred", "CFNAI")];
const FRED_SAHM: &[SeriesRef] = &[series("fred", "SAHMREALTIME")];
const FRED_ICSA: &[SeriesRef] = &[series("fred", "ICSA")];
const FRED_CCSA: &[SeriesRef] = &[series("fred", "CCSA")];
const FRED_MORTGAGE: &[SeriesRef] = &[series("fred", "MORTGAGE30US")];
const FRED_CARD_DELINQUENCY: &[SeriesRef] = &[series("fred", "DRCCLACBS")];
const FRED_LOAN_DELINQUENCY: &[SeriesRef] = &[series("fred", "DRCLACBS")];
const FRED_BUSINESS_LOANS: &[SeriesRef] = &[series("fred", "BUSLOANS")];
const FRED_TOTAL_LOANS: &[SeriesRef] = &[series("fred", "TOTLL")];
const FRED_FED_ASSETS: &[SeriesRef] = &[series("fred", "WALCL")];
const FRED_RRP: &[SeriesRef] = &[series("fred", "RRPONTSYD")];
const FRED_TOTAL_RESERVES: &[SeriesRef] = &[series("fred", "TOTRESNS")];
const FRED_RESERVE_BALANCES: &[SeriesRef] = &[series("fred", "WRESBAL")];
const FRED_BANK_CAPITAL: &[SeriesRef] = &[series("fred", "EQTA")];
const NYFED_DEALER_FAILS: &[SeriesRef] = &[series("nyfed", "DEALER_FAILS")];
const SCOOS_MARGIN: &[SeriesRef] = &[series("scoos", "MARGIN_TIGHTENING")];
const TREASURY_DEALER: &[SeriesRef] = &[series("treasury", "AUCTION_DEALER_SHARE")];
const TREASURY_DIRECT: &[SeriesRef] = &[series("treasury", "AUCTION_DIRECT_SHARE")];
const TREASURY_INDIRECT: &[SeriesRef] = &[series("treasury", "AUCTION_INDIRECT_SHARE")];
const BIS_DOLLAR_CREDIT: &[SeriesRef] = &[series("bis", "GLOBAL_DOLLAR_CREDIT")];
const FRED_KR_CLI: &[SeriesRef] = &[series("fred", "KORLOLITOAASTSAM")];
const FRED_CN_CLI: &[SeriesRef] = &[series("fred", "CHNLOLITOAASTSAM")];

const KOSPI_RETURN: &[SeriesRef] = &[series("krx", "KRX_KOSPI_RETURN")];
const KOSDAQ_RETURN: &[SeriesRef] = &[series("krx", "KRX_KOSDAQ_RETURN")];
const KOSPI_VALUE: &[SeriesRef] = &[series("krx", "KRX_KOSPI_TOTAL_VALUE")];
const KOSDAQ_VALUE: &[SeriesRef] = &[series("krx", "KRX_KOSDAQ_TOTAL_VALUE")];
const KOSPI_VOLUME: &[SeriesRef] = &[series("krx", "KRX_KOSPI_TOTAL_VOLUME")];
const KOSDAQ_VOLUME: &[SeriesRef] = &[series("krx", "KRX_KOSDAQ_TOTAL_VOLUME")];
const KOSPI_CAP: &[SeriesRef] = &[series("krx", "KRX_KOSPI_MARKET_CAP")];
const KOSDAQ_CAP: &[SeriesRef] = &[series("krx", "KRX_KOSDAQ_MARKET_CAP")];
const KOSPI_ISSUES: &[SeriesRef] = &[series("krx", "KRX_KOSPI_LISTED_ISSUE_COUNT")];
const KOSDAQ_ISSUES: &[SeriesRef] = &[series("krx", "KRX_KOSDAQ_LISTED_ISSUE_COUNT")];
const KRX_FUTURES_VOLUME: &[SeriesRef] = &[series("krx", "KRX_FUTURES_TOTAL_VOLUME")];
const KRX_FUTURES_VALUE: &[SeriesRef] = &[series("krx", "KRX_FUTURES_TOTAL_VALUE")];
const KRX_OPTIONS_OI: &[SeriesRef] = &[series("krx", "KRX_OPTIONS_OI")];
const KRX_OPTIONS_VOLUME: &[SeriesRef] = &[series("krx", "KRX_OPTIONS_TOTAL_VOLUME")];
const KRX_OPTIONS_VALUE: &[SeriesRef] = &[series("krx", "KRX_OPTIONS_TOTAL_VALUE")];
const KRX_BOND_DURATION: &[SeriesRef] = &[series("krx", "KRX_BOND_INDEX_AVG_DURATION")];
const KRX_BOND_CONVEXITY: &[SeriesRef] = &[series("krx", "KRX_BOND_INDEX_AVG_CONVEXITY")];
const KRX_BOND_BASKET_YIELD: &[SeriesRef] = &[series("krx", "KRX_BOND_AVG_YIELD")];
const KRX_SMALL_BOND_YIELD: &[SeriesRef] = &[series("krx", "KRX_SMALL_BOND_AVG_YIELD")];
const KRX_BOND_VALUE: &[SeriesRef] = &[series("krx", "KRX_BOND_TOTAL_VALUE")];
const KRX_KTS_VALUE: &[SeriesRef] = &[series("krx", "KRX_KTS_BOND_TOTAL_VALUE")];
const KRX_ETF_BREADTH: &[SeriesRef] = &[series("krx", "KRX_ETF_BREADTH")];
const KRX_ETF_VALUE: &[SeriesRef] = &[series("krx", "KRX_ETF_TOTAL_VALUE")];
const KRX_ETF_CAP: &[SeriesRef] = &[series("krx", "KRX_ETF_MARKET_CAP")];
const KRX_ETN_BREADTH: &[SeriesRef] = &[series("krx", "KRX_ETN_BREADTH")];
const KRX_ETN_VALUE: &[SeriesRef] = &[series("krx", "KRX_ETN_TOTAL_VALUE")];
const KRX_ETN_CAP: &[SeriesRef] = &[series("krx", "KRX_ETN_MARKET_CAP")];
const KRX_ELW_BREADTH: &[SeriesRef] = &[series("krx", "KRX_ELW_BREADTH")];
const KRX_ELW_VALUE: &[SeriesRef] = &[series("krx", "KRX_ELW_TOTAL_VALUE")];
const KRX_GOLD_VALUE: &[SeriesRef] = &[series("krx", "KRX_GOLD_TOTAL_VALUE")];
const KRX_GOLD_VOLUME: &[SeriesRef] = &[series("krx", "KRX_GOLD_TOTAL_VOLUME")];
const KRX_OIL_PRICE: &[SeriesRef] = &[series("krx", "KRX_OIL_AVG_PRICE")];
const KRX_OIL_VALUE: &[SeriesRef] = &[series("krx", "KRX_OIL_TOTAL_VALUE")];
const KRX_EMISSIONS_BREADTH: &[SeriesRef] = &[series("krx", "KRX_EMISSIONS_BREADTH")];
const KRX_EMISSIONS_VALUE: &[SeriesRef] = &[series("krx", "KRX_EMISSIONS_TOTAL_VALUE")];
const KRX_ESG_BREADTH: &[SeriesRef] = &[series("krx", "KRX_ESG_PRODUCTS_BREADTH")];
const KRX_ESG_INDEX_RETURN: &[SeriesRef] = &[series("krx", "KRX_ESG_INDEX_AVG_RETURN")];
const KRX_SRI_ISSUES: &[SeriesRef] = &[series("krx", "KRX_SRI_BONDS_ISSUE_COUNT")];
const KRX_SRI_AMOUNT: &[SeriesRef] = &[series("krx", "KRX_SRI_BONDS_TOTAL_LISTED_AMOUNT")];
const KRX_KONEX_BREADTH: &[SeriesRef] = &[series("krx", "KRX_KONEX_BREADTH")];
const KRX_KONEX_CAP: &[SeriesRef] = &[series("krx", "KRX_KONEX_MARKET_CAP")];

const BTC_FUNDING_ABS: &[SeriesRef] = &[series("binance", "BTC_FUNDING_ABS")];
const BTC_BASIS_ABS: &[SeriesRef] = &[series("binance", "BTC_BASIS_ABS")];
const BTC_SPOT_HIGH: &[SeriesRef] = &[series("binance", "BTC_SPOT_HIGH_24H")];
const BTC_SPOT_LOW: &[SeriesRef] = &[series("binance", "BTC_SPOT_LOW_24H")];
const BTC_SPOT_VOLUME: &[SeriesRef] = &[series("binance", "BTC_SPOT_VOLUME_24H")];
const BTC_SPOT_QUOTE_VOLUME: &[SeriesRef] = &[series("binance", "BTC_SPOT_QUOTE_VOLUME_24H")];
const BTC_SPOT_CHANGE: &[SeriesRef] = &[series("binance", "BTC_SPOT_CHANGE_24H")];
const BTC_MARK_PRICE: &[SeriesRef] = &[series("binance", "BTC_MARK_PRICE_USD")];
const BTC_INDEX_PRICE: &[SeriesRef] = &[series("binance", "BTC_INDEX_PRICE_USD")];
const BTC_CURRENT_FUNDING: &[SeriesRef] = &[series("binance", "BTC_CURRENT_FUNDING_RATE")];

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
        "코스피200 정규장 최근월 선물 베이시스",
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
        "코스피200 정규장 단순선물 전체 월물 미결제약정 합계",
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
        "코스피200 정규장 최근월 옵션 거래량 풋/콜",
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
        "코스피200 정규장 최근월 거래량가중 IV",
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
        "비트코인 현물(Binance BTC/USDT)",
        "BTC SPOT",
        "crypto",
        "spot",
        BTC_SPOT_PRICE,
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
        "24H",
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
    indicator(
        "ig_spread",
        "미국 투자등급 회사채 OAS",
        "IG OAS",
        "us",
        "credit",
        FRED_IG,
        "percent",
        2,
        2,
        "1D",
    ),
    indicator(
        "curve_10y3m",
        "미국 10년-3개월 금리차",
        "10Y3M",
        "us",
        "bonds",
        FRED_CURVE_3M,
        "percent",
        2,
        2,
        "1D",
    ),
    indicator(
        "ofr_fsi",
        "OFR 미국 금융스트레스지수",
        "OFR FSI",
        "us",
        "conditions",
        OFR_FSI,
        "index",
        3,
        2,
        "1D",
    ),
    indicator(
        "stlfsi",
        "세인트루이스 연은 금융스트레스지수",
        "STLFSI",
        "us",
        "conditions",
        FRED_STLFSI,
        "index",
        3,
        2,
        "WoW",
    ),
    indicator(
        "nfci",
        "시카고 연은 금융여건지수",
        "NFCI",
        "us",
        "conditions",
        FRED_NFCI,
        "index",
        3,
        2,
        "WoW",
    ),
    indicator(
        "anfci",
        "조정 시카고 연은 금융여건지수",
        "ANFCI",
        "us",
        "conditions",
        FRED_ANFCI,
        "index",
        3,
        2,
        "WoW",
    ),
    indicator(
        "nfci_leverage",
        "미국 금융여건 레버리지 하위지수",
        "NFCI LEV",
        "us",
        "leverage",
        FRED_NFCI_LEVERAGE,
        "index",
        3,
        2,
        "WoW",
    ),
    indicator(
        "wei",
        "미국 주간경제지수(연율 성장률)",
        "WEI",
        "us",
        "macro",
        FRED_WEI,
        "percent",
        2,
        2,
        "WoW",
    ),
    indicator(
        "cfnai",
        "시카고 연은 국가활동지수",
        "CFNAI",
        "us",
        "macro",
        FRED_CFNAI,
        "index",
        2,
        2,
        "MoM",
    ),
    indicator(
        "sahm",
        "실시간 Sahm 경기침체 지표",
        "SAHM",
        "us",
        "labor",
        FRED_SAHM,
        "percent",
        2,
        2,
        "MoM",
    ),
    indicator(
        "initial_claims",
        "미국 신규 실업수당 청구",
        "ICSA",
        "us",
        "labor",
        FRED_ICSA,
        "count",
        0,
        2,
        "WoW",
    ),
    indicator(
        "continued_claims",
        "미국 계속 실업수당 청구",
        "CCSA",
        "us",
        "labor",
        FRED_CCSA,
        "count",
        0,
        2,
        "WoW",
    ),
    indicator(
        "mortgage30",
        "미국 30년 고정 모기지 금리",
        "MORTGAGE",
        "us",
        "housing",
        FRED_MORTGAGE,
        "percent",
        2,
        2,
        "WoW",
    ),
    indicator(
        "card_delinquency",
        "미국 신용카드 연체율",
        "CARD DELQ",
        "us",
        "banking",
        FRED_CARD_DELINQUENCY,
        "percent",
        2,
        2,
        "QoQ",
    ),
    indicator(
        "loan_delinquency",
        "미국 은행 전체 대출·리스 연체율",
        "LOAN DELQ",
        "us",
        "banking",
        FRED_LOAN_DELINQUENCY,
        "percent",
        2,
        2,
        "QoQ",
    ),
    indicator(
        "business_loans",
        "미국 상업·산업 대출 잔액",
        "C&I LOANS",
        "us",
        "credit",
        FRED_BUSINESS_LOANS,
        "usd_billion",
        1,
        2,
        "MoM",
    ),
    indicator(
        "total_loans",
        "미국 상업은행 총 대출·리스",
        "TOTAL LOANS",
        "us",
        "credit",
        FRED_TOTAL_LOANS,
        "usd_billion",
        1,
        2,
        "WoW",
    ),
    indicator(
        "fed_assets",
        "연준 총자산",
        "FED ASSETS",
        "us",
        "liquidity",
        FRED_FED_ASSETS,
        "usd_million",
        0,
        2,
        "WoW",
    ),
    indicator(
        "rrp",
        "연준 익일 역레포 잔액",
        "RRP",
        "us",
        "liquidity",
        FRED_RRP,
        "usd_billion",
        1,
        2,
        "1D",
    ),
    indicator(
        "total_reserves",
        "미국 예금기관 총 준비금",
        "RESERVES",
        "us",
        "liquidity",
        FRED_TOTAL_RESERVES,
        "usd_billion",
        1,
        2,
        "MoM",
    ),
    indicator(
        "reserve_balances",
        "연준 지급준비금 잔액",
        "RES BAL",
        "us",
        "liquidity",
        FRED_RESERVE_BALANCES,
        "usd_million",
        0,
        2,
        "WoW",
    ),
    indicator(
        "bank_capital",
        "미국 은행 자기자본/자산(중단 참고계열)",
        "BANK CAP",
        "us",
        "banking",
        FRED_BANK_CAPITAL,
        "percent",
        2,
        2,
        "QoQ",
    ),
    indicator(
        "dealer_fails",
        "뉴욕 연은 국채·기관채 딜러 결제실패",
        "DLR FAILS",
        "us",
        "market_plumbing",
        NYFED_DEALER_FAILS,
        "usd_million",
        0,
        2,
        "WoW",
    ),
    indicator(
        "margin_tightening",
        "미국 헤지펀드 가격조건 순긴축 응답",
        "SCOOS",
        "us",
        "funding",
        SCOOS_MARGIN,
        "percent",
        1,
        2,
        "QoQ",
    ),
    indicator(
        "auction_dealer",
        "미 국채 입찰 딜러 배정 비중(혼합 만기 보조)",
        "DEALER",
        "us",
        "treasury",
        TREASURY_DEALER,
        "fraction_percent",
        2,
        2,
        "AUCTION",
    ),
    indicator(
        "auction_direct",
        "미 국채 입찰 직접응찰자 배정 비중(혼합 만기)",
        "DIRECT",
        "us",
        "treasury",
        TREASURY_DIRECT,
        "fraction_percent",
        2,
        2,
        "AUCTION",
    ),
    indicator(
        "auction_indirect",
        "미 국채 입찰 간접응찰자 배정 비중(혼합 만기)",
        "INDIRECT",
        "us",
        "treasury",
        TREASURY_INDIRECT,
        "fraction_percent",
        2,
        2,
        "AUCTION",
    ),
    indicator(
        "global_dollar_credit",
        "미국 외 비은행권 달러신용",
        "USD CREDIT",
        "us",
        "global",
        BIS_DOLLAR_CREDIT,
        "usd_million",
        0,
        2,
        "QoQ",
    ),
    indicator(
        "kr_cli",
        "한국 OECD 경기선행지수",
        "KR CLI",
        "korea",
        "macro",
        FRED_KR_CLI,
        "index",
        2,
        2,
        "MoM",
    ),
    indicator(
        "cn_cli",
        "중국 OECD 경기선행지수(한국 외부수요)",
        "CN CLI",
        "korea",
        "macro",
        FRED_CN_CLI,
        "index",
        2,
        2,
        "MoM",
    ),
    indicator(
        "kospi_return",
        "KRX 공식 코스피 일간 등락률",
        "KOSPI RET",
        "korea",
        "stocks",
        KOSPI_RETURN,
        "percent",
        2,
        2,
        "1D",
    ),
    indicator(
        "kosdaq_return",
        "KRX 공식 코스닥 일간 등락률",
        "KOSDAQ RET",
        "korea",
        "stocks",
        KOSDAQ_RETURN,
        "percent",
        2,
        2,
        "1D",
    ),
    indicator(
        "kospi_value",
        "코스피 일간 거래대금",
        "KOSPI VALUE",
        "korea",
        "stocks",
        KOSPI_VALUE,
        "krw_amount",
        0,
        2,
        "1D",
    ),
    indicator(
        "kosdaq_value",
        "코스닥 일간 거래대금",
        "KOSDAQ VALUE",
        "korea",
        "stocks",
        KOSDAQ_VALUE,
        "krw_amount",
        0,
        2,
        "1D",
    ),
    indicator(
        "kospi_volume",
        "코스피 일간 거래량",
        "KOSPI VOL",
        "korea",
        "stocks",
        KOSPI_VOLUME,
        "count",
        0,
        2,
        "1D",
    ),
    indicator(
        "kosdaq_volume",
        "코스닥 일간 거래량",
        "KOSDAQ VOL",
        "korea",
        "stocks",
        KOSDAQ_VOLUME,
        "count",
        0,
        2,
        "1D",
    ),
    indicator(
        "kospi_cap",
        "코스피 전체 시가총액",
        "KOSPI CAP",
        "korea",
        "stocks",
        KOSPI_CAP,
        "krw_amount",
        0,
        2,
        "1D",
    ),
    indicator(
        "kosdaq_cap",
        "코스닥 전체 시가총액",
        "KOSDAQ CAP",
        "korea",
        "stocks",
        KOSDAQ_CAP,
        "krw_amount",
        0,
        2,
        "1D",
    ),
    indicator(
        "kospi_issues",
        "코스피 상장 종목 수",
        "KOSPI ISSUES",
        "korea",
        "stocks",
        KOSPI_ISSUES,
        "count",
        0,
        2,
        "DAILY",
    ),
    indicator(
        "kosdaq_issues",
        "코스닥 상장 종목 수",
        "KOSDAQ ISSUES",
        "korea",
        "stocks",
        KOSDAQ_ISSUES,
        "count",
        0,
        2,
        "DAILY",
    ),
    indicator(
        "krx_futures_volume",
        "코스피200 정규장 선물 전체 월물 거래량",
        "K200 FUT VOL",
        "korea",
        "futures",
        KRX_FUTURES_VOLUME,
        "contracts",
        0,
        2,
        "1D",
    ),
    indicator(
        "krx_futures_value",
        "코스피200 정규장 선물 전체 월물 거래대금",
        "K200 FUT VAL",
        "korea",
        "futures",
        KRX_FUTURES_VALUE,
        "krw_amount",
        0,
        2,
        "1D",
    ),
    indicator(
        "krx_options_oi",
        "코스피200 정규장 최근월 옵션 미결제약정",
        "K200 OPT OI",
        "korea",
        "options",
        KRX_OPTIONS_OI,
        "contracts",
        0,
        2,
        "1D",
    ),
    indicator(
        "krx_options_volume",
        "코스피200 정규장 최근월 옵션 거래량",
        "K200 OPT VOL",
        "korea",
        "options",
        KRX_OPTIONS_VOLUME,
        "contracts",
        0,
        2,
        "1D",
    ),
    indicator(
        "krx_options_value",
        "코스피200 정규장 최근월 옵션 거래대금",
        "K200 OPT VAL",
        "korea",
        "options",
        KRX_OPTIONS_VALUE,
        "krw_amount",
        0,
        2,
        "1D",
    ),
    indicator(
        "krx_bond_duration",
        "KRX 채권지수 바스켓 평균 듀레이션",
        "BOND DUR",
        "korea",
        "bonds",
        KRX_BOND_DURATION,
        "years",
        2,
        2,
        "1D",
    ),
    indicator(
        "krx_bond_convexity",
        "KRX 채권지수 바스켓 평균 컨벡시티",
        "BOND CONV",
        "korea",
        "bonds",
        KRX_BOND_CONVEXITY,
        "index",
        2,
        2,
        "1D",
    ),
    indicator(
        "krx_bond_basket_yield",
        "KRX 거래채권 바스켓 평균수익률",
        "BOND YLD",
        "korea",
        "bonds",
        KRX_BOND_BASKET_YIELD,
        "percent",
        2,
        2,
        "1D",
    ),
    indicator(
        "krx_small_bond_yield",
        "KRX 소액채권 바스켓 평균수익률",
        "SMB YLD",
        "korea",
        "bonds",
        KRX_SMALL_BOND_YIELD,
        "percent",
        2,
        2,
        "1D",
    ),
    indicator(
        "krx_bond_value",
        "KRX 일반채권 거래대금",
        "BOND VALUE",
        "korea",
        "bonds",
        KRX_BOND_VALUE,
        "krw_amount",
        0,
        2,
        "1D",
    ),
    indicator(
        "krx_kts_value",
        "국채전문시장 거래대금",
        "KTS VALUE",
        "korea",
        "bonds",
        KRX_KTS_VALUE,
        "krw_amount",
        0,
        2,
        "1D",
    ),
    indicator(
        "etf_breadth",
        "KRX ETF 상승상품 비율",
        "ETF BR",
        "korea",
        "etp",
        KRX_ETF_BREADTH,
        "percent",
        1,
        2,
        "1D",
    ),
    indicator(
        "etf_value",
        "KRX ETF 일간 거래대금",
        "ETF VALUE",
        "korea",
        "etp",
        KRX_ETF_VALUE,
        "krw_amount",
        0,
        2,
        "1D",
    ),
    indicator(
        "etf_cap",
        "KRX ETF 전체 시가총액",
        "ETF CAP",
        "korea",
        "etp",
        KRX_ETF_CAP,
        "krw_amount",
        0,
        2,
        "1D",
    ),
    indicator(
        "etn_breadth",
        "KRX ETN 상승상품 비율",
        "ETN BR",
        "korea",
        "etp",
        KRX_ETN_BREADTH,
        "percent",
        1,
        2,
        "1D",
    ),
    indicator(
        "etn_value",
        "KRX ETN 일간 거래대금",
        "ETN VALUE",
        "korea",
        "etp",
        KRX_ETN_VALUE,
        "krw_amount",
        0,
        2,
        "1D",
    ),
    indicator(
        "etn_cap",
        "KRX ETN 전체 시가총액",
        "ETN CAP",
        "korea",
        "etp",
        KRX_ETN_CAP,
        "krw_amount",
        0,
        2,
        "1D",
    ),
    indicator(
        "elw_breadth",
        "KRX ELW 상승상품 비율",
        "ELW BR",
        "korea",
        "etp",
        KRX_ELW_BREADTH,
        "percent",
        1,
        2,
        "1D",
    ),
    indicator(
        "elw_value",
        "KRX ELW 일간 거래대금",
        "ELW VALUE",
        "korea",
        "etp",
        KRX_ELW_VALUE,
        "krw_amount",
        0,
        2,
        "1D",
    ),
    indicator(
        "gold_value",
        "KRX 금시장 일간 거래대금(2개 상품)",
        "GOLD VALUE",
        "korea",
        "commodities",
        KRX_GOLD_VALUE,
        "krw_amount",
        0,
        2,
        "1D",
    ),
    indicator(
        "gold_volume",
        "KRX 금시장 일간 거래량",
        "GOLD VOL",
        "korea",
        "commodities",
        KRX_GOLD_VOLUME,
        "count",
        0,
        2,
        "1D",
    ),
    indicator(
        "oil_price",
        "KRX 석유시장 상품 바스켓 평균가격",
        "OIL AVG",
        "korea",
        "commodities",
        KRX_OIL_PRICE,
        "krw",
        2,
        2,
        "1D",
    ),
    indicator(
        "oil_value",
        "KRX 석유시장 일간 거래대금",
        "OIL VALUE",
        "korea",
        "commodities",
        KRX_OIL_VALUE,
        "krw_amount",
        0,
        2,
        "1D",
    ),
    indicator(
        "emissions_breadth",
        "KRX 배출권 상승상품 비율",
        "ETS BR",
        "korea",
        "secondary",
        KRX_EMISSIONS_BREADTH,
        "percent",
        1,
        2,
        "1D",
    ),
    indicator(
        "emissions_value",
        "KRX 배출권 일간 거래대금",
        "ETS VALUE",
        "korea",
        "secondary",
        KRX_EMISSIONS_VALUE,
        "krw_amount",
        0,
        2,
        "1D",
    ),
    indicator(
        "esg_breadth",
        "KRX ESG 증권상품 상승 비율",
        "ESG BR",
        "korea",
        "secondary",
        KRX_ESG_BREADTH,
        "percent",
        1,
        2,
        "1D",
    ),
    indicator(
        "esg_index_return",
        "KRX ESG 지수군 평균 등락률",
        "ESG IDX",
        "korea",
        "secondary",
        KRX_ESG_INDEX_RETURN,
        "percent",
        2,
        2,
        "1D",
    ),
    indicator(
        "sri_issues",
        "KRX 사회책임투자채권 상장 종목 수",
        "SRI ISSUES",
        "korea",
        "secondary",
        KRX_SRI_ISSUES,
        "count",
        0,
        2,
        "DAILY",
    ),
    indicator(
        "sri_amount",
        "KRX 사회책임투자채권 상장 잔액",
        "SRI AMOUNT",
        "korea",
        "secondary",
        KRX_SRI_AMOUNT,
        "krw_amount",
        0,
        2,
        "DAILY",
    ),
    indicator(
        "konex_breadth",
        "KRX 코넥스 상승종목 비율",
        "KONEX BR",
        "korea",
        "secondary",
        KRX_KONEX_BREADTH,
        "percent",
        1,
        2,
        "1D",
    ),
    indicator(
        "konex_cap",
        "KRX 코넥스 전체 시가총액",
        "KONEX CAP",
        "korea",
        "secondary",
        KRX_KONEX_CAP,
        "krw_amount",
        0,
        2,
        "1D",
    ),
    indicator(
        "btc_funding_abs",
        "BTC 펀딩비 절댓값(위험계산용)",
        "FUND |ABS|",
        "crypto",
        "derived",
        BTC_FUNDING_ABS,
        "rate",
        4,
        4,
        "24H",
    ),
    indicator(
        "btc_basis_abs",
        "BTC 무기한선물 베이시스 절댓값(위험계산용)",
        "BASIS |ABS|",
        "crypto",
        "derived",
        BTC_BASIS_ABS,
        "rate",
        4,
        25,
        "24H",
    ),
    indicator(
        "btc_spot_high",
        "BTC 현물 24시간 고가",
        "SPOT HIGH",
        "crypto",
        "spot",
        BTC_SPOT_HIGH,
        "usd",
        0,
        2,
        "LIVE 24H",
    ),
    indicator(
        "btc_spot_low",
        "BTC 현물 24시간 저가",
        "SPOT LOW",
        "crypto",
        "spot",
        BTC_SPOT_LOW,
        "usd",
        0,
        2,
        "LIVE 24H",
    ),
    indicator(
        "btc_spot_volume",
        "BTC 현물 24시간 거래량",
        "SPOT VOL",
        "crypto",
        "spot",
        BTC_SPOT_VOLUME,
        "btc",
        2,
        2,
        "LIVE 24H",
    ),
    indicator(
        "btc_spot_quote_volume",
        "BTC 현물 24시간 거래대금",
        "SPOT VALUE",
        "crypto",
        "spot",
        BTC_SPOT_QUOTE_VOLUME,
        "usd",
        0,
        2,
        "LIVE 24H",
    ),
    indicator(
        "btc_spot_change",
        "BTC 현물 공식 24시간 등락률",
        "SPOT 24H",
        "crypto",
        "spot",
        BTC_SPOT_CHANGE,
        "percent",
        2,
        2,
        "LIVE 24H",
    ),
    indicator(
        "btc_perp_price",
        "BTC 무기한선물 최근가격",
        "PERP LAST",
        "crypto",
        "futures",
        BTC_PERP_PRICE,
        "usd",
        1,
        25,
        "24H",
    ),
    indicator(
        "btc_mark_price",
        "BTC 무기한선물 마크가격",
        "MARK",
        "crypto",
        "futures",
        BTC_MARK_PRICE,
        "usd",
        1,
        2,
        "LIVE",
    ),
    indicator(
        "btc_index_price",
        "BTC 무기한선물 기초지수 가격",
        "INDEX",
        "crypto",
        "futures",
        BTC_INDEX_PRICE,
        "usd",
        1,
        2,
        "LIVE",
    ),
    indicator(
        "btc_current_funding",
        "BTC 무기한선물 현재 펀딩비",
        "CURRENT FUND",
        "crypto",
        "futures",
        BTC_CURRENT_FUNDING,
        "rate",
        4,
        2,
        "LIVE",
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

pub fn build_at(db: &Db, as_of: Option<&str>) -> rusqlite::Result<DashboardData> {
    let mut indicators = Vec::with_capacity(INDICATORS.len());
    for definition in INDICATORS {
        indicators.push(read_indicator(db, definition, as_of)?);
    }
    Ok(DashboardData { indicators })
}

fn point_is_newer(candidate: &Point, selected: &Point) -> bool {
    (
        candidate.observed_at.as_str(),
        candidate.ingested_at.as_str(),
    ) > (selected.observed_at.as_str(), selected.ingested_at.as_str())
}

fn cadence(selected: Option<SeriesRef>) -> &'static str {
    let Some(selected) = selected else {
        return "UNKNOWN";
    };
    if selected.source == "binance" {
        return "INTRADAY";
    }
    if selected.source == "krx" || selected.source == "ecos" && selected.series == "KR_USD_KRW" {
        return "DAILY";
    }
    match selected.series {
        "DRCCLACBS" | "DRCLACBS" | "EQTA" | "GLOBAL_DOLLAR_CREDIT" | "MARGIN_TIGHTENING" => {
            "QUARTERLY"
        }
        "CFNAI" | "SAHMREALTIME" | "BUSLOANS" | "TOTRESNS" | "KORLOLITOAASTSAM"
        | "CHNLOLITOAASTSAM" | "KR_BASE_RATE" => "MONTHLY",
        "WEI" | "ICSA" | "CCSA" | "STLFSI4" | "NFCI" | "ANFCI" | "NFCILEVERAGE"
        | "MORTGAGE30US" | "WALCL" | "WRESBAL" | "TOTLL" | "DEALER_FAILS" => "WEEKLY",
        _ => "DAILY",
    }
}

fn max_age_days(selected: Option<SeriesRef>) -> i64 {
    match cadence(selected) {
        "INTRADAY" => 2,
        "DAILY" => 10,
        "WEEKLY" => 18,
        "MONTHLY" => 75,
        "QUARTERLY" => 150,
        _ => 10,
    }
}

fn freshness(
    db: &Db,
    selected: Option<SeriesRef>,
    point: Option<&Point>,
) -> rusqlite::Result<String> {
    let Some(point) = point else {
        return Ok("NO DATA".into());
    };
    if selected.is_some_and(|value| value.source.eq_ignore_ascii_case("binance")) {
        if let Ok(ingested) = DateTime::parse_from_rfc3339(&point.ingested_at) {
            let age = Utc::now()
                .signed_duration_since(ingested.with_timezone(&Utc))
                .num_seconds();
            return Ok(if age <= 120 {
                "LIVE".into()
            } else if age <= 600 {
                "DELAYED".into()
            } else {
                format!("STALE {}m", age.max(0) / 60)
            });
        }
    }
    let date = point
        .observed_at
        .get(..10)
        .and_then(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok());
    let Some(date) = date else {
        return Ok("UNKNOWN".into());
    };
    let age = (Utc::now().date_naive() - date).num_days();
    if let Some(selected) = selected.filter(|value| value.source.eq_ignore_ascii_case("krx")) {
        if let Some(status) = db.series_status(selected.source, selected.series)? {
            let checked_recently = DateTime::parse_from_rfc3339(&status.checked_at)
                .ok()
                .is_some_and(|checked| {
                    Utc::now()
                        .signed_duration_since(checked.with_timezone(&Utc))
                        .num_hours()
                        <= 24
                });
            if checked_recently && status.latest_observed_at == point.observed_at {
                return Ok("LATEST VERIFIED".into());
            }
        }
    }
    if age <= 0 {
        Ok("TODAY".into())
    } else if age <= max_age_days(selected) {
        Ok("PUBLISHED EOD".into())
    } else {
        Ok(format!("STALE {age}d"))
    }
}

fn read_indicator(
    db: &Db,
    definition: &IndicatorDefinition,
    as_of: Option<&str>,
) -> rusqlite::Result<DashboardIndicator> {
    let mut selected: Option<(SeriesRef, Vec<Point>)> = None;
    for candidate in definition.candidates {
        let points = db.recent(
            candidate.source,
            candidate.series,
            definition.comparison_points.max(400),
            as_of,
        )?;
        let Some(latest) = points.last() else {
            continue;
        };
        let replace = selected
            .as_ref()
            .and_then(|(_, selected_points)| selected_points.last())
            .is_none_or(|selected_latest| point_is_newer(latest, selected_latest));
        if replace {
            selected = Some((*candidate, points));
        }
    }
    let (mut selected_ref, points) = selected
        .map(|(candidate, points)| (Some(candidate), points))
        .unwrap_or_default();
    let mut points = points;
    if let Some(binance_ref) = definition
        .candidates
        .iter()
        .find(|candidate| candidate.source == "binance")
        .copied()
    {
        if let Some(live) = db.latest_live_quote("binance", binance_ref.series, "BTCUSDT")? {
            selected_ref = Some(binance_ref);
            if points
                .last()
                .is_none_or(|point| point.observed_at < live.observed_at)
            {
                points.push(live);
            } else if let Some(last) = points.last_mut() {
                *last = live;
            }
        }
    }
    let source = selected_ref.map(|candidate| candidate.source.to_uppercase());
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
    let values = points.iter().map(|point| point.value).collect::<Vec<_>>();
    let history_low = values.iter().copied().reduce(f64::min);
    let history_high = values.iter().copied().reduce(f64::max);
    let history_average =
        (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64);
    let range_position =
        latest
            .zip(history_low.zip(history_high))
            .and_then(|(latest, (low, high))| {
                ((high - low).abs() > f64::EPSILON)
                    .then(|| 100.0 * (latest.value - low) / (high - low))
            });
    let series_name = selected_ref.map(|candidate| candidate.series.to_string());
    let source_series =
        selected_ref.map(|candidate| format!("{}:{}", candidate.source, candidate.series));
    Ok(DashboardIndicator {
        key: definition.key,
        label: definition.label,
        symbol: definition.symbol,
        market: definition.market,
        asset_class: definition.asset_class,
        value: latest.map(|point| point.value),
        raw_value: latest.map(|point| point.value),
        previous_value: comparison.map(|point| point.value),
        change,
        change_pct,
        history: values,
        history_points: points
            .iter()
            .map(|point| DashboardHistoryPoint {
                observed_at: point.observed_at.clone(),
                value: point.value,
            })
            .collect(),
        history_low,
        history_high,
        history_average,
        range_position,
        observations: points.len(),
        observed_at: latest.map(|point| point.observed_at.clone()),
        ingested_at: latest.map(|point| point.ingested_at.clone()),
        source,
        series: series_name,
        source_series,
        freshness: freshness(db, selected_ref, latest)?,
        cadence: cadence(selected_ref),
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
        let dashboard = build_at(&db, None).unwrap();
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

    #[test]
    fn dashboard_uses_the_freshest_fallback_source() {
        let temporary = tempfile::tempdir().unwrap();
        let db = Db::open(&temporary.path().join("dashboard-fresh.db")).unwrap();
        db.put(&NewObservation::simple(
            "ecos",
            "KR_USD_KRW",
            "2026-08-18",
            1_390.0,
        ))
        .unwrap();
        db.put(&NewObservation::simple(
            "fred",
            "DEXKOUS",
            "2026-08-20",
            1_395.0,
        ))
        .unwrap();
        let dashboard = build_at(&db, None).unwrap();
        let usdkrw = dashboard
            .indicators
            .iter()
            .find(|indicator| indicator.key == "usdkrw")
            .unwrap();
        assert_eq!(usdkrw.value, Some(1_395.0));
        assert_eq!(usdkrw.source.as_deref(), Some("FRED"));
    }
}
