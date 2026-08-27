use crate::db::{Db, Point};
use chrono::{Duration, NaiveDate, Utc};
use serde::Serialize;
use std::collections::BTreeMap;

const HISTORY_LIMIT: usize = 3_000;
const HISTORY_START_DAYS: i64 = 365 * 7;

#[derive(Clone, Debug, Serialize)]
pub struct CentralBankHistoryPoint {
    pub observed_at: String,
    pub fed_yoy: Option<f64>,
    pub ecb_yoy: Option<f64>,
    pub boj_yoy: Option<f64>,
    pub g3_yoy: Option<f64>,
    pub g3_usd_yoy: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CentralBankComponent {
    pub key: &'static str,
    pub name: &'static str,
    pub series: &'static str,
    pub source_series: String,
    pub native_unit: &'static str,
    pub native_value: Option<f64>,
    pub usd_trillion: Option<f64>,
    pub yoy: Option<f64>,
    pub change_1m: Option<f64>,
    pub annualized_3m: Option<f64>,
    pub yoy_acceleration_3m: Option<f64>,
    pub observed_at: Option<String>,
    pub freshness: String,
    pub increase_meaning: &'static str,
    pub decrease_meaning: &'static str,
    pub interpretation: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct LiquidityFactor {
    pub key: &'static str,
    pub label: &'static str,
    pub value: Option<f64>,
    pub unit: &'static str,
    pub score: Option<f64>,
    pub weight: f64,
    pub contribution: Option<f64>,
    pub source_series: &'static str,
    pub observed_at: Option<String>,
    pub increase_meaning: &'static str,
    pub decrease_meaning: &'static str,
    pub interpretation: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct FedPlumbing {
    pub net_liquidity_usd_trillion: Option<f64>,
    pub net_liquidity_yoy: Option<f64>,
    pub net_liquidity_3m_annualized: Option<f64>,
    pub reserve_balances_usd_trillion: Option<f64>,
    pub reserve_balances_3m_annualized: Option<f64>,
    pub reverse_repo_usd_trillion: Option<f64>,
    pub treasury_general_account_usd_trillion: Option<f64>,
    pub observed_at: Option<String>,
    pub formula: &'static str,
    pub explanation: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct CentralBankLiquidity {
    pub score: Option<f64>,
    pub long_bias: Option<f64>,
    pub short_bias: Option<f64>,
    pub confidence: f64,
    pub signal: String,
    pub regime: String,
    pub summary: String,
    pub as_of: Option<String>,
    pub g3_total_usd_trillion: Option<f64>,
    pub g3_yoy: Option<f64>,
    pub g3_usd_yoy: Option<f64>,
    pub g3_annualized_3m: Option<f64>,
    pub g3_yoy_acceleration_3m: Option<f64>,
    pub components: Vec<CentralBankComponent>,
    pub factors: Vec<LiquidityFactor>,
    pub fed_plumbing: FedPlumbing,
    pub history: Vec<CentralBankHistoryPoint>,
    pub methodology: Vec<&'static str>,
    pub caveats: Vec<&'static str>,
}

#[derive(Clone, Copy)]
enum Conversion<'a> {
    UsdMillions,
    EurMillions(&'a [Point]),
    JpyHundredMillion(&'a [Point]),
}

#[derive(Clone, Copy)]
struct BankData<'a> {
    key: &'static str,
    name: &'static str,
    series: &'static str,
    native_unit: &'static str,
    points: &'a [Point],
    conversion: Conversion<'a>,
    max_lag_days: i64,
    increase_meaning: &'static str,
    decrease_meaning: &'static str,
}

#[derive(Clone, Debug)]
struct BankMetrics {
    native_value: f64,
    usd_trillion: f64,
    yoy: Option<f64>,
    change_1m: Option<f64>,
    annualized_3m: Option<f64>,
    yoy_acceleration_3m: Option<f64>,
    observed_at: String,
}

fn date(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value.get(..10).unwrap_or(value), "%Y-%m-%d").ok()
}

fn requested_date(as_of: Option<&str>) -> NaiveDate {
    as_of
        .and_then(date)
        .unwrap_or_else(|| Utc::now().date_naive())
}

fn point_at_or_before(points: &[Point], target: NaiveDate) -> Option<&Point> {
    points
        .iter()
        .rev()
        .find(|point| date(&point.observed_at).is_some_and(|value| value <= target))
}

fn valid_latest<'a>(bank: &BankData<'a>, target: NaiveDate) -> Option<&'a Point> {
    let point = point_at_or_before(bank.points, target)?;
    let observed = date(&point.observed_at)?;
    ((target - observed).num_days() <= bank.max_lag_days).then_some(point)
}

fn pct_change(points: &[Point], target: NaiveDate, days: i64) -> Option<f64> {
    let latest = point_at_or_before(points, target)?;
    let latest_date = date(&latest.observed_at)?;
    let prior = point_at_or_before(points, latest_date - Duration::days(days))?;
    let prior_date = date(&prior.observed_at)?;
    let span = (latest_date - prior_date).num_days();
    if span < days * 2 / 3 || prior.value.abs() <= f64::EPSILON {
        return None;
    }
    Some(100.0 * (latest.value / prior.value - 1.0))
}

fn annualized_change(points: &[Point], target: NaiveDate, days: i64) -> Option<f64> {
    let latest = point_at_or_before(points, target)?;
    let latest_date = date(&latest.observed_at)?;
    let prior = point_at_or_before(points, latest_date - Duration::days(days))?;
    let prior_date = date(&prior.observed_at)?;
    let span = (latest_date - prior_date).num_days();
    if span < days * 2 / 3 || prior.value <= 0.0 || latest.value <= 0.0 {
        return None;
    }
    Some(100.0 * ((latest.value / prior.value).powf(365.0 / span as f64) - 1.0))
}

fn fx_value(points: &[Point], target: NaiveDate) -> Option<f64> {
    let point = point_at_or_before(points, target)?;
    let observed = date(&point.observed_at)?;
    ((target - observed).num_days() <= 10 && point.value > 0.0).then_some(point.value)
}

fn usd_trillion(value: f64, target: NaiveDate, conversion: Conversion<'_>) -> Option<f64> {
    match conversion {
        Conversion::UsdMillions => Some(value / 1_000_000.0),
        Conversion::EurMillions(fx) => Some(value * fx_value(fx, target)? / 1_000_000.0),
        Conversion::JpyHundredMillion(fx) => Some(value / (10_000.0 * fx_value(fx, target)?)),
    }
}

fn bank_metrics(bank: &BankData<'_>, target: NaiveDate) -> Option<BankMetrics> {
    let latest = valid_latest(bank, target)?;
    let latest_date = date(&latest.observed_at)?;
    let yoy = pct_change(bank.points, latest_date, 365);
    let prior_yoy = pct_change(bank.points, latest_date - Duration::days(91), 365);
    Some(BankMetrics {
        native_value: latest.value,
        usd_trillion: usd_trillion(latest.value, latest_date, bank.conversion)?,
        yoy,
        change_1m: pct_change(bank.points, latest_date, 30),
        annualized_3m: annualized_change(bank.points, latest_date, 91),
        yoy_acceleration_3m: yoy.zip(prior_yoy).map(|(now, prior)| now - prior),
        observed_at: latest.observed_at.clone(),
    })
}

fn weighted_metric<F>(metrics: &[BankMetrics], select: F) -> Option<f64>
where
    F: Fn(&BankMetrics) -> Option<f64>,
{
    if metrics.len() != 3 {
        return None;
    }
    let mut numerator = 0.0;
    let mut denominator = 0.0;
    for metric in metrics {
        numerator += metric.usd_trillion * select(metric)?;
        denominator += metric.usd_trillion;
    }
    (denominator > 0.0).then_some(numerator / denominator)
}

fn g3_usd_level(banks: &[BankData<'_>], target: NaiveDate) -> Option<f64> {
    let levels = banks
        .iter()
        .map(|bank| bank_metrics(bank, target).map(|metric| metric.usd_trillion))
        .collect::<Option<Vec<_>>>()?;
    (levels.len() == 3).then(|| levels.iter().sum())
}

fn g3_usd_yoy(banks: &[BankData<'_>], target: NaiveDate) -> Option<f64> {
    let current = g3_usd_level(banks, target)?;
    let prior = g3_usd_level(banks, target - Duration::days(365))?;
    (prior > 0.0).then_some(100.0 * (current / prior - 1.0))
}

fn component(
    bank: &BankData<'_>,
    metric: Option<BankMetrics>,
    target: NaiveDate,
) -> CentralBankComponent {
    let interpretation = match metric.as_ref().and_then(|value| value.yoy) {
        Some(value) if value > 0.0 => format!(
            "전년보다 자산이 {value:.2}% 증가해 중앙은행 유동성 충격에 플러스입니다. 단, 증가율이 둔화 중이면 롱 강도는 약해집니다."
        ),
        Some(value) if value < 0.0 => format!(
            "전년보다 자산이 {:.2}% 감소해 유동성 충격은 마이너스입니다. 증가율이 위로 올라가면 축소 속도 완화로 해석합니다.",
            value.abs()
        ),
        Some(_) => "전년 수준과 비슷해 자산 증가율만으로는 방향성이 중립입니다.".into(),
        None => "전년 비교 데이터가 부족해 방향을 추정하지 않습니다.".into(),
    };
    let observed_at = metric.as_ref().map(|value| value.observed_at.clone());
    let freshness = observed_at
        .as_deref()
        .and_then(date)
        .map(|observed| {
            let age = (target - observed).num_days().max(0);
            if age <= bank.max_lag_days {
                format!("공식 최신 · {age}일 시차")
            } else {
                format!("STALE {age}d")
            }
        })
        .unwrap_or_else(|| "NO DATA".into());
    CentralBankComponent {
        key: bank.key,
        name: bank.name,
        series: bank.series,
        source_series: format!("fred:{}", bank.series),
        native_unit: bank.native_unit,
        native_value: metric.as_ref().map(|value| value.native_value),
        usd_trillion: metric.as_ref().map(|value| value.usd_trillion),
        yoy: metric.as_ref().and_then(|value| value.yoy),
        change_1m: metric.as_ref().and_then(|value| value.change_1m),
        annualized_3m: metric.as_ref().and_then(|value| value.annualized_3m),
        yoy_acceleration_3m: metric.as_ref().and_then(|value| value.yoy_acceleration_3m),
        observed_at,
        freshness,
        increase_meaning: bank.increase_meaning,
        decrease_meaning: bank.decrease_meaning,
        interpretation,
    }
}

fn normalized_score(value: Option<f64>, full_scale: f64) -> Option<f64> {
    value.map(|value| (100.0 * value / full_scale).clamp(-100.0, 100.0))
}

#[allow(clippy::too_many_arguments)]
fn factor(
    key: &'static str,
    label: &'static str,
    value: Option<f64>,
    unit: &'static str,
    weight: f64,
    full_scale: f64,
    source_series: &'static str,
    observed_at: Option<String>,
    increase_meaning: &'static str,
    decrease_meaning: &'static str,
) -> LiquidityFactor {
    let score = normalized_score(value, full_scale);
    let interpretation = match value {
        Some(value) if value > 0.0 => {
            format!("현재 {value:+.2}{unit}로 상승 방향이며 롱 성향을 강화합니다.")
        }
        Some(value) if value < 0.0 => {
            format!("현재 {value:+.2}{unit}로 하락 방향이며 숏 성향을 강화합니다.")
        }
        Some(_) => "현재 변화가 0에 가까워 방향성 기여가 중립입니다.".into(),
        None => "필요한 공식 관측치가 부족해 이 요인은 점수에서 제외됩니다.".into(),
    };
    LiquidityFactor {
        key,
        label,
        value,
        unit,
        score,
        weight,
        contribution: score.map(|score| score * weight / 100.0),
        source_series,
        observed_at,
        increase_meaning,
        decrease_meaning,
        interpretation,
    }
}

fn derived_fed_net(
    target: NaiveDate,
    fed: &[Point],
    tga: &[Point],
    rrp: &[Point],
) -> Option<(NaiveDate, f64)> {
    let effective = [
        point_at_or_before(fed, target)?,
        point_at_or_before(tga, target)?,
        point_at_or_before(rrp, target)?,
    ]
    .iter()
    .filter_map(|point| date(&point.observed_at))
    .min()?;
    if (target - effective).num_days() > 21 {
        return None;
    }
    // Align all three legs to the slowest common observation date. Using a newer
    // daily RRP value with an older weekly Fed/TGA value would introduce look-ahead.
    let fed_point = point_at_or_before(fed, effective)?;
    let tga_point = point_at_or_before(tga, effective)?;
    let rrp_point = point_at_or_before(rrp, effective)?;
    Some((
        effective,
        (fed_point.value - tga_point.value - rrp_point.value * 1_000.0) / 1_000_000.0,
    ))
}

fn derived_change<F>(target: NaiveDate, days: i64, annualized: bool, value_at: F) -> Option<f64>
where
    F: Fn(NaiveDate) -> Option<(NaiveDate, f64)>,
{
    let (effective, current) = value_at(target)?;
    let (prior_date, prior) = value_at(effective - Duration::days(days))?;
    let span = (effective - prior_date).num_days();
    if span < days * 2 / 3 || current <= 0.0 || prior <= 0.0 {
        return None;
    }
    if annualized {
        Some(100.0 * ((current / prior).powf(365.0 / span as f64) - 1.0))
    } else {
        Some(100.0 * (current / prior - 1.0))
    }
}

fn fed_plumbing(
    target: NaiveDate,
    fed: &[Point],
    tga: &[Point],
    rrp: &[Point],
    reserves: &[Point],
) -> FedPlumbing {
    let value_at = |date| derived_fed_net(date, fed, tga, rrp);
    let net = value_at(target);
    let latest_reserves = point_at_or_before(reserves, target);
    let latest_rrp = point_at_or_before(rrp, target);
    let latest_tga = point_at_or_before(tga, target);
    FedPlumbing {
        net_liquidity_usd_trillion: net.map(|(_, value)| value),
        net_liquidity_yoy: derived_change(target, 365, false, value_at),
        net_liquidity_3m_annualized: derived_change(target, 91, true, value_at),
        reserve_balances_usd_trillion: latest_reserves.map(|point| point.value / 1_000_000.0),
        reserve_balances_3m_annualized: annualized_change(reserves, target, 91),
        reverse_repo_usd_trillion: latest_rrp.map(|point| point.value / 1_000.0),
        treasury_general_account_usd_trillion: latest_tga.map(|point| point.value / 1_000_000.0),
        observed_at: net.map(|(date, _)| date.to_string()),
        formula: "Fed 총자산 − 미 재무부 TGA − ON RRP",
        explanation: "총자산만 볼 때 생기는 착시를 줄이기 위한 보조치입니다. TGA 증가는 은행 준비금을 흡수하고, ON RRP 감소는 초기 QT 충격을 흡수할 수 있습니다. 광의통화나 투자 가능한 현금을 정확히 측정하는 공식 통계는 아닙니다.",
    }
}

fn monthly_history(banks: &[BankData<'_>], target: NaiveDate) -> Vec<CentralBankHistoryPoint> {
    let start = target - Duration::days(HISTORY_START_DAYS);
    let mut months = BTreeMap::<String, NaiveDate>::new();
    for point in banks[0].points {
        let Some(observed) = date(&point.observed_at) else {
            continue;
        };
        if observed < start || observed > target {
            continue;
        }
        months.insert(observed.format("%Y-%m").to_string(), observed);
    }
    months
        .into_values()
        .filter_map(|observed| {
            let metrics = banks
                .iter()
                .filter_map(|bank| bank_metrics(bank, observed))
                .collect::<Vec<_>>();
            if metrics.len() != 3 {
                return None;
            }
            Some(CentralBankHistoryPoint {
                observed_at: observed.to_string(),
                fed_yoy: metrics[0].yoy,
                ecb_yoy: metrics[1].yoy,
                boj_yoy: metrics[2].yoy,
                g3_yoy: weighted_metric(&metrics, |metric| metric.yoy),
                g3_usd_yoy: g3_usd_yoy(banks, observed),
            })
        })
        .collect()
}

fn regime(yoy: Option<f64>, acceleration: Option<f64>) -> String {
    match yoy.zip(acceleration) {
        Some((yoy, acceleration)) if yoy >= 0.0 && acceleration >= 0.0 => {
            "확장 가속 · 강한 롱 환경".into()
        }
        Some((yoy, _)) if yoy >= 0.0 => "확장 중이나 둔화 · 롱 강도 약화".into(),
        Some((_, acceleration)) if acceleration >= 0.0 => "축소 중이나 완화 · 롱 방향 개선".into(),
        Some(_) => "축소 가속 · 숏 환경 강화".into(),
        None => "데이터 부족".into(),
    }
}

fn signal(score: Option<f64>, confidence: f64) -> String {
    if confidence < 60.0 || score.is_none() {
        return "DATA WAIT".into();
    }
    match score.unwrap_or_default() {
        value if value >= 40.0 => "강한 롱 우세".into(),
        value if value >= 15.0 => "롱 우세".into(),
        value if value <= -40.0 => "강한 숏 우세".into(),
        value if value <= -15.0 => "숏 우세".into(),
        _ => "중립·혼조".into(),
    }
}

pub fn build(db: &Db, as_of: Option<&str>) -> rusqlite::Result<CentralBankLiquidity> {
    let target = requested_date(as_of);
    let fed = db.recent("fred", "WALCL", HISTORY_LIMIT, as_of)?;
    let ecb = db.recent("fred", "ECBASSETSW", HISTORY_LIMIT, as_of)?;
    let boj = db.recent("fred", "JPNASSETS", HISTORY_LIMIT, as_of)?;
    let eurusd = db.recent("fred", "DEXUSEU", HISTORY_LIMIT, as_of)?;
    let usdjpy = db.recent("fred", "DEXJPUS", HISTORY_LIMIT, as_of)?;
    let tga = db.recent("fred", "WTREGEN", HISTORY_LIMIT, as_of)?;
    let rrp = db.recent("fred", "RRPONTSYD", HISTORY_LIMIT, as_of)?;
    let reserves = db.recent("fred", "WRESBAL", HISTORY_LIMIT, as_of)?;

    let banks = [
        BankData {
            key: "fed",
            name: "미국 연방준비제도 총자산",
            series: "WALCL",
            native_unit: "백만 달러",
            points: &fed,
            conversion: Conversion::UsdMillions,
            max_lag_days: 21,
            increase_meaning: "미국 달러 준비금과 중앙은행 자산 완충력이 늘어나는 방향이며 다른 조건이 같으면 위험자산에 우호적입니다.",
            decrease_meaning: "Fed 보유자산이 민간으로 이전되고 준비금 완충력이 줄어 장기금리·레포시장에 긴축 압력을 줄 수 있습니다.",
        },
        BankData {
            key: "ecb",
            name: "유로시스템 중앙은행 총자산",
            series: "ECBASSETSW",
            native_unit: "백만 유로",
            points: &ecb,
            conversion: Conversion::EurMillions(&eurusd),
            max_lag_days: 21,
            increase_meaning: "유로지역 중앙은행 유동성 공급이 확대되는 방향이며 글로벌 위험자산과 유로 자금조달에 우호적일 수 있습니다.",
            decrease_meaning: "APP·PEPP 만기상환과 대출 회수 등의 정상화가 민간 채권흡수 부담과 기간 프리미엄을 높일 수 있습니다.",
        },
        BankData {
            key: "boj",
            name: "일본은행 총자산",
            series: "JPNASSETS",
            native_unit: "1억 엔",
            points: &boj,
            conversion: Conversion::JpyHundredMillion(&usdjpy),
            max_lag_days: 70,
            increase_meaning: "일본은행의 엔화 본원유동성과 국채시장 지원이 확대되는 방향으로 글로벌 캐리 유동성에 우호적일 수 있습니다.",
            decrease_meaning: "JGB 매입축소·만기상환으로 일본은행의 시장 완충 역할과 엔화 본원유동성이 줄어드는 방향입니다.",
        },
    ];

    let metrics = banks
        .iter()
        .filter_map(|bank| bank_metrics(bank, target))
        .collect::<Vec<_>>();
    let components = banks
        .iter()
        .map(|bank| component(bank, bank_metrics(bank, target), target))
        .collect::<Vec<_>>();
    let g3_total =
        (metrics.len() == 3).then(|| metrics.iter().map(|value| value.usd_trillion).sum());
    let g3_yoy = weighted_metric(&metrics, |metric| metric.yoy);
    let g3_annualized_3m = weighted_metric(&metrics, |metric| metric.annualized_3m);
    let g3_acceleration = weighted_metric(&metrics, |metric| metric.yoy_acceleration_3m);
    let usd_yoy = g3_usd_yoy(&banks, target);
    let plumbing = fed_plumbing(target, &fed, &tga, &rrp, &reserves);
    let observed_at = components
        .iter()
        .filter_map(|component| component.observed_at.as_deref())
        .min()
        .map(str::to_string);

    let factors = vec![
        factor(
            "g3_yoy",
            "G3 자산증가율 YoY · 검은선 수준",
            g3_yoy,
            "%",
            30.0,
            10.0,
            "fred:WALCL+ECBASSETSW+JPNASSETS",
            observed_at.clone(),
            "0% 위로 높아질수록 순확대와 롱 성향이 강해집니다.",
            "0% 아래로 낮아질수록 순축소와 숏 성향이 강해집니다.",
        ),
        factor(
            "g3_yoy_acceleration",
            "G3 YoY 3개월 가속도 · 검은선 기울기",
            g3_acceleration,
            "%p",
            25.0,
            8.0,
            "derived:G3_YOY_ACCELERATION",
            observed_at.clone(),
            "마이너스 구간에서도 상승하면 유동성 회수 속도가 느려져 롱 방향으로 개선됩니다.",
            "하락하면 자산 증가 둔화 또는 축소 가속이므로 숏 방향으로 악화됩니다.",
        ),
        factor(
            "g3_3m_annualized",
            "G3 최근 3개월 연율 변화",
            g3_annualized_3m,
            "%",
            20.0,
            15.0,
            "derived:G3_3M_ANNUALIZED",
            observed_at.clone(),
            "최근 실제 자산 흐름이 확대되고 있어 오래된 YoY 기저효과보다 빠른 롱 확인 신호입니다.",
            "최근 자산 흐름이 축소되고 있어 YoY가 양수여도 향후 숏 전환을 경고할 수 있습니다.",
        ),
        factor(
            "fed_net_3m_annualized",
            "Fed 순유동성 3개월 연율",
            plumbing.net_liquidity_3m_annualized,
            "%",
            15.0,
            20.0,
            "derived:WALCL-WTREGEN-RRPONTSYD",
            plumbing.observed_at.clone(),
            "TGA와 ON RRP를 조정한 Fed 측 시장 유동성이 늘어 롱 성향을 확인합니다.",
            "Fed 측 순유동성이 줄어 미국 달러 자금시장과 위험자산에 숏 압력을 줍니다.",
        ),
        factor(
            "reserve_balances_3m_annualized",
            "미국 은행 지급준비금 3개월 연율",
            plumbing.reserve_balances_3m_annualized,
            "%",
            10.0,
            20.0,
            "fred:WRESBAL",
            plumbing.observed_at.clone(),
            "결제·레포시장 완충재가 증가해 유동성 충격 흡수력이 개선됩니다.",
            "준비금 완충재가 줄어 레포금리와 단기자금 변동성이 커질 위험이 높아집니다.",
        ),
    ];
    let available_weight = factors
        .iter()
        .filter(|factor| factor.score.is_some())
        .map(|factor| factor.weight)
        .sum::<f64>();
    let total_score = (available_weight >= 60.0).then(|| {
        factors
            .iter()
            .filter_map(|factor| factor.contribution)
            .sum::<f64>()
            * 100.0
            / available_weight
    });
    let confidence = available_weight.clamp(0.0, 100.0);
    let signal = signal(total_score, confidence);
    let regime = regime(g3_yoy, g3_acceleration);
    let summary = match (g3_yoy, g3_acceleration, total_score) {
        (Some(yoy), Some(acceleration), Some(score)) => format!(
            "G3 총자산 YoY는 {yoy:+.2}%이고 3개월 가속도는 {acceleration:+.2}%p입니다. 검은선의 수준과 기울기, 최근 3개월 흐름, Fed 순유동성·준비금을 결합한 점수는 {score:+.1}입니다. 이 값은 수익 확률이 아니라 중앙은행 유동성 방향성입니다."
        ),
        _ => "Fed·ECB·BOJ와 환율 또는 자금시장 데이터가 부족해 종합 방향을 계산하지 않습니다.".into(),
    };

    Ok(CentralBankLiquidity {
        score: total_score,
        long_bias: total_score.map(|score| 50.0 + score / 2.0),
        short_bias: total_score.map(|score| 50.0 - score / 2.0),
        confidence,
        signal,
        regime,
        summary,
        as_of: observed_at,
        g3_total_usd_trillion: g3_total,
        g3_yoy,
        g3_usd_yoy: usd_yoy,
        g3_annualized_3m,
        g3_yoy_acceleration_3m: g3_acceleration,
        components,
        factors,
        fed_plumbing: plumbing,
        history: monthly_history(&banks, target),
        methodology: vec![
            "핵심 검은선은 Fed·ECB·BOJ의 현지통화 YoY를 현재 달러환산 자산규모로 가중한 FX 중립 G3 증가율입니다.",
            "점수는 G3 YoY 30%, YoY 3개월 가속도 25%, G3 3개월 연율 20%, Fed 순유동성 3개월 연율 15%, 미국 지급준비금 3개월 연율 10%로 구성합니다.",
            "각 요인은 사전에 정한 완전점수 구간으로 -100~+100에 제한하며, 결측 요인은 제외하되 가용 가중치가 60% 미만이면 DATA WAIT로 처리합니다.",
            "롱·숏 성향은 점수를 보기 쉽게 양분한 방향성 지수이며, 백테스트로 보정된 수익 확률이나 매매 권고가 아닙니다.",
        ],
        caveats: vec![
            "중앙은행 총자산 감소는 본원통화 측 유동성 축소와 연결되지만 광의통화·민간신용의 동일한 감소를 뜻하지 않습니다.",
            "USD 환산 G3 YoY는 환율 변동이 포함되므로 정책 충격 판단에는 FX 중립 G3 YoY를 우선합니다.",
            "자산 증가율은 유동성의 속도, 자산 절대규모와 지급준비금은 남아 있는 완충재이므로 둘을 함께 봅니다.",
            "Fed·ECB·BOJ의 발표주기가 달라 종합 기준일은 세 입력 중 가장 느린 공식 관측일을 사용합니다.",
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::NewObservation;

    fn put(db: &Db, series: &str, date: &str, value: f64) {
        db.put(&NewObservation::simple("fred", series, date, value))
            .unwrap();
    }

    fn point(date: &str, value: f64) -> Point {
        Point {
            observed_at: date.into(),
            value,
            released_at: None,
            source_asof: None,
            ingested_at: "2026-08-27T00:00:00Z".into(),
        }
    }

    #[test]
    fn expanding_and_accelerating_g3_produces_long_bias() {
        let temporary = tempfile::tempdir().unwrap();
        let db = Db::open(&temporary.path().join("central-bank.db")).unwrap();
        for (date, multiplier) in [
            ("2024-05-20", 0.80),
            ("2024-08-20", 0.82),
            ("2025-05-20", 0.95),
            ("2025-08-20", 1.00),
            ("2026-05-20", 1.02),
            ("2026-08-20", 1.10),
        ] {
            put(&db, "WALCL", date, 7_000_000.0 * multiplier);
            put(&db, "ECBASSETSW", date, 6_000_000.0 * multiplier);
            put(&db, "JPNASSETS", date, 6_000_000.0 * multiplier);
            put(&db, "DEXUSEU", date, 1.1);
            put(&db, "DEXJPUS", date, 150.0);
            put(&db, "WTREGEN", date, 800_000.0);
            put(&db, "RRPONTSYD", date, 100.0);
            put(&db, "WRESBAL", date, 3_000_000.0 * multiplier);
        }
        let result = build(&db, None).unwrap();
        assert!(result.g3_yoy.unwrap() > 0.0);
        assert!(result.g3_yoy_acceleration_3m.unwrap() > 0.0);
        assert!(result.score.unwrap() > 0.0);
        assert!(result.long_bias.unwrap() > result.short_bias.unwrap());
        assert!(result.regime.contains("확장 가속"));
    }

    #[test]
    fn missing_g3_inputs_are_not_fabricated() {
        let temporary = tempfile::tempdir().unwrap();
        let db = Db::open(&temporary.path().join("central-bank-missing.db")).unwrap();
        put(&db, "WALCL", "2026-08-01", 7_000_000.0);
        let result = build(&db, Some("2026-08-01T23:59:59Z")).unwrap();
        assert_eq!(result.score, None);
        assert_eq!(result.g3_yoy, None);
        assert_eq!(result.signal, "DATA WAIT");
    }

    #[test]
    fn fed_net_liquidity_aligns_daily_and_weekly_inputs() {
        let fed = vec![point("2026-08-20", 7_000_000.0)];
        let tga = vec![point("2026-08-20", 1_000_000.0)];
        let rrp = vec![point("2026-08-20", 100.0), point("2026-08-21", 900.0)];
        let (observed, value) = derived_fed_net(
            NaiveDate::from_ymd_opt(2026, 8, 21).unwrap(),
            &fed,
            &tga,
            &rrp,
        )
        .unwrap();
        assert_eq!(observed, NaiveDate::from_ymd_opt(2026, 8, 20).unwrap());
        assert!((value - 5.9).abs() < 1e-9);
    }
}
