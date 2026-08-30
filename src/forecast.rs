use crate::db::Db;
use serde::Serialize;
use std::{cmp::Ordering, collections::BTreeMap};

const MIN_FEATURE_POINTS: usize = 64;
const MIN_NEIGHBORS: usize = 30;
const MAX_NEIGHBORS: usize = 80;

#[derive(Clone, Debug, Serialize)]
pub struct MarketForecast {
    pub market: &'static str,
    pub label: &'static str,
    pub source_series: String,
    pub as_of: Option<String>,
    pub status: &'static str,
    pub methodology: &'static str,
    pub current_regime: Option<CurrentRegime>,
    pub horizons: Vec<ForecastHorizon>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CurrentRegime {
    pub return_1w: f64,
    pub return_1m: f64,
    pub return_3m: f64,
    pub realized_volatility_20d: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ForecastHorizon {
    pub horizon_days: usize,
    pub flat_band_percent: f64,
    pub rise_probability: f64,
    pub sideways_probability: f64,
    pub fall_probability: f64,
    pub dominant_direction: &'static str,
    pub expected_return_percent: f64,
    pub analog_samples: usize,
    pub average_distance: f64,
    pub validation_samples: usize,
    pub validation_hit_rate: Option<f64>,
    pub validation_brier: Option<f64>,
    pub uniform_brier: f64,
    pub validation_state: &'static str,
    pub warning: &'static str,
}

#[derive(Clone, Copy)]
struct ForecastConfig {
    market: &'static str,
    label: &'static str,
    source: &'static str,
    series: &'static str,
    flat_daily: f64,
    distance_scales: [f64; 4],
}

#[derive(Clone, Copy)]
struct Features {
    return_1w: f64,
    return_1m: f64,
    return_3m: f64,
    volatility: f64,
}

struct Distribution {
    probabilities: [f64; 3],
    expected_return: f64,
    samples: usize,
    average_distance: f64,
}

pub fn build(db: &Db, as_of: Option<&str>) -> rusqlite::Result<Vec<MarketForecast>> {
    let configs = [
        ForecastConfig {
            market: "us",
            label: "미국 S&P 500",
            source: "fred",
            series: "SP500",
            flat_daily: 0.0025,
            distance_scales: [0.025, 0.06, 0.12, 0.12],
        },
        ForecastConfig {
            market: "korea",
            label: "한국 KOSPI",
            source: "krx",
            series: "KRX_KOSPI_CLOSE",
            flat_daily: 0.0035,
            distance_scales: [0.035, 0.08, 0.16, 0.16],
        },
        ForecastConfig {
            market: "crypto",
            label: "비트코인 USD 기준가격",
            source: "coinmetrics",
            series: "BTC_PRICE_USD",
            flat_daily: 0.0100,
            distance_scales: [0.10, 0.22, 0.45, 0.45],
        },
    ];
    configs
        .iter()
        .map(|config| build_market(db, *config, as_of))
        .collect()
}

fn build_market(
    db: &Db,
    config: ForecastConfig,
    as_of: Option<&str>,
) -> rusqlite::Result<MarketForecast> {
    let points = db.recent(config.source, config.series, 5_000, as_of)?;
    let mut daily = BTreeMap::new();
    for point in points {
        if let Some(date) = point.observed_at.get(..10) {
            if point.value.is_finite() && point.value > 0.0 {
                daily.insert(date.to_string(), point.value);
            }
        }
    }
    let dates = daily.keys().cloned().collect::<Vec<_>>();
    let prices = daily.values().copied().collect::<Vec<_>>();
    let source_series = format!("{}:{}", config.source, config.series);
    if prices.len() < MIN_FEATURE_POINTS + MIN_NEIGHBORS + 21 {
        return Ok(MarketForecast {
            market: config.market,
            label: config.label,
            source_series,
            as_of: dates.last().cloned(),
            status: "INSUFFICIENT HISTORY",
            methodology: "현재 1주·1개월·3개월 수익률과 20일 실현변동성이 비슷했던 과거 구간의 이후 수익률을 비교합니다.",
            current_regime: None,
            horizons: Vec::new(),
        });
    }
    let current_index = prices.len() - 1;
    let feature_cache = (0..prices.len())
        .map(|index| features(&prices, index))
        .collect::<Vec<_>>();
    let current = feature_cache[current_index].expect("history length checked");
    let current_regime = CurrentRegime {
        return_1w: current.return_1w * 100.0,
        return_1m: current.return_1m * 100.0,
        return_3m: current.return_3m * 100.0,
        realized_volatility_20d: current.volatility * 100.0,
    };
    let horizons = [1_usize, 5, 21]
        .into_iter()
        .filter_map(|horizon| {
            let flat_band = config.flat_daily * (horizon as f64).sqrt();
            let distribution = analog_distribution(
                &prices,
                &feature_cache,
                current_index,
                horizon,
                flat_band,
                current_index.saturating_sub(horizon),
                config.distance_scales,
            )?;
            let validation = validate(
                &prices,
                &feature_cache,
                horizon,
                flat_band,
                config.distance_scales,
            );
            let dominant = dominant_direction(distribution.probabilities);
            let validation_state = match validation {
                Some((_, brier, samples)) if samples >= 50 && brier < 2.0 / 3.0 => {
                    "VALIDATED / BEATS UNIFORM"
                }
                Some((_, _, samples)) if samples >= 50 => "VALIDATED / NO EDGE VS UNIFORM",
                _ => "VALIDATION INSUFFICIENT",
            };
            Some(ForecastHorizon {
                horizon_days: horizon,
                flat_band_percent: flat_band * 100.0,
                rise_probability: distribution.probabilities[0] * 100.0,
                sideways_probability: distribution.probabilities[1] * 100.0,
                fall_probability: distribution.probabilities[2] * 100.0,
                dominant_direction: dominant,
                expected_return_percent: distribution.expected_return * 100.0,
                analog_samples: distribution.samples,
                average_distance: distribution.average_distance,
                validation_samples: validation.map(|value| value.2).unwrap_or(0),
                validation_hit_rate: validation.map(|value| value.0 * 100.0),
                validation_brier: validation.map(|value| value.1),
                uniform_brier: 2.0 / 3.0,
                validation_state,
                warning: "유사국면의 조건부 빈도이며 보장된 수익확률이 아닙니다. 검증이 기준모형을 이기지 못하면 방향판단에 사용하지 마세요.",
            })
        })
        .collect::<Vec<_>>();
    Ok(MarketForecast {
        market: config.market,
        label: config.label,
        source_series,
        as_of: dates.last().cloned(),
        status: if horizons.is_empty() {
            "INSUFFICIENT ANALOGS"
        } else {
            "EXPERIMENTAL / WALK-FORWARD CHECKED"
        },
        methodology: "현재 1주·1개월·3개월 수익률과 20일 실현변동성이 비슷했던 과거 구간을 거리순으로 최대 80개 선택하고, 유사도 가중 이후 수익률을 상승·횡보·하락 빈도로 변환합니다. 최근 최대 120개 시점은 당시 이용 가능 데이터만으로 순차 검증합니다.",
        current_regime: Some(current_regime),
        horizons,
    })
}

fn features(prices: &[f64], index: usize) -> Option<Features> {
    if index < MIN_FEATURE_POINTS || prices.get(index).copied()? <= 0.0 {
        return None;
    }
    let price = prices[index];
    let return_1w = price / prices[index - 5] - 1.0;
    let return_1m = price / prices[index - 21] - 1.0;
    let return_3m = price / prices[index - 63] - 1.0;
    let returns = (index - 19..=index)
        .map(|position| (prices[position] / prices[position - 1]).ln())
        .collect::<Vec<_>>();
    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance = returns
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / (returns.len() - 1) as f64;
    Some(Features {
        return_1w,
        return_1m,
        return_3m,
        volatility: variance.sqrt() * 252.0_f64.sqrt(),
    })
}

fn feature_distance(left: Features, right: Features, scales: [f64; 4]) -> f64 {
    [
        (left.return_1w - right.return_1w) / scales[0],
        (left.return_1m - right.return_1m) / scales[1],
        (left.return_3m - right.return_3m) / scales[2],
        (left.volatility - right.volatility) / scales[3],
    ]
    .iter()
    .map(|value| value * value)
    .sum::<f64>()
    .sqrt()
}

fn outcome_index(value: f64, flat_band: f64) -> usize {
    if value > flat_band {
        0
    } else if value < -flat_band {
        2
    } else {
        1
    }
}

fn analog_distribution(
    prices: &[f64],
    feature_cache: &[Option<Features>],
    current_index: usize,
    horizon: usize,
    flat_band: f64,
    candidate_end: usize,
    scales: [f64; 4],
) -> Option<Distribution> {
    let current = feature_cache.get(current_index).copied().flatten()?;
    let mut candidates = (MIN_FEATURE_POINTS..=candidate_end)
        .filter_map(|index| {
            if index + horizon >= prices.len() || index >= current_index {
                return None;
            }
            let candidate = feature_cache.get(index).copied().flatten()?;
            let future_return = prices[index + horizon] / prices[index] - 1.0;
            Some((feature_distance(current, candidate, scales), future_return))
        })
        .collect::<Vec<_>>();
    if candidates.len() < MIN_NEIGHBORS {
        return None;
    }
    if candidates.len() > MAX_NEIGHBORS {
        candidates.select_nth_unstable_by(MAX_NEIGHBORS, |left, right| {
            left.0.partial_cmp(&right.0).unwrap_or(Ordering::Equal)
        });
        candidates.truncate(MAX_NEIGHBORS);
    }
    let selected = candidates.len().min(MAX_NEIGHBORS);
    let mut weighted = [0.0_f64; 3];
    let mut weight_sum = 0.0;
    let mut expected_return = 0.0;
    let mut distance_sum = 0.0;
    for (distance, future_return) in candidates.into_iter().take(selected) {
        let weight = (-distance.min(20.0)).exp().max(1e-9);
        let outcome = outcome_index(future_return, flat_band);
        weighted[outcome] += weight;
        weight_sum += weight;
        expected_return += weight * future_return;
        distance_sum += distance;
    }
    if weight_sum <= 0.0 {
        return None;
    }
    Some(Distribution {
        probabilities: weighted.map(|value| value / weight_sum),
        expected_return: expected_return / weight_sum,
        samples: selected,
        average_distance: distance_sum / selected as f64,
    })
}

fn validate(
    prices: &[f64],
    feature_cache: &[Option<Features>],
    horizon: usize,
    flat_band: f64,
    scales: [f64; 4],
) -> Option<(f64, f64, usize)> {
    if prices.len() < 180 + horizon {
        return None;
    }
    let last_origin = prices.len() - horizon - 1;
    let first_origin =
        (MIN_FEATURE_POINTS + MIN_NEIGHBORS + horizon).max(last_origin.saturating_sub(119));
    let mut correct = 0_usize;
    let mut brier = 0.0;
    let mut count = 0_usize;
    for origin in first_origin..=last_origin {
        let Some(distribution) = analog_distribution(
            prices,
            feature_cache,
            origin,
            horizon,
            flat_band,
            origin.saturating_sub(horizon),
            scales,
        ) else {
            continue;
        };
        let realized = prices[origin + horizon] / prices[origin] - 1.0;
        let outcome = outcome_index(realized, flat_band);
        let predicted = distribution
            .probabilities
            .iter()
            .enumerate()
            .max_by(|left, right| left.1.partial_cmp(right.1).unwrap_or(Ordering::Equal))
            .map(|value| value.0)
            .unwrap_or(1);
        correct += usize::from(predicted == outcome);
        brier += distribution
            .probabilities
            .iter()
            .enumerate()
            .map(|(index, probability)| {
                let actual = if index == outcome { 1.0 } else { 0.0 };
                (probability - actual).powi(2)
            })
            .sum::<f64>();
        count += 1;
    }
    (count > 0).then_some((correct as f64 / count as f64, brier / count as f64, count))
}

fn dominant_direction(probabilities: [f64; 3]) -> &'static str {
    let maximum = probabilities
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    if maximum < 0.40 {
        return "혼조";
    }
    match probabilities
        .iter()
        .position(|value| (*value - maximum).abs() < f64::EPSILON)
        .unwrap_or(1)
    {
        0 => "상승",
        2 => "하락",
        _ => "횡보",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcome_bands_are_explicit() {
        assert_eq!(outcome_index(0.02, 0.01), 0);
        assert_eq!(outcome_index(0.005, 0.01), 1);
        assert_eq!(outcome_index(-0.02, 0.01), 2);
    }

    #[test]
    fn forecast_probabilities_sum_to_one() {
        let prices = (0..500)
            .map(|index| 100.0 * (1.0_f64 + 0.001).powi(index))
            .collect::<Vec<_>>();
        let forecast = analog_distribution(
            &prices,
            &(0..prices.len())
                .map(|index| features(&prices, index))
                .collect::<Vec<_>>(),
            prices.len() - 1,
            5,
            0.005,
            prices.len() - 6,
            [0.03, 0.08, 0.16, 0.16],
        )
        .expect("synthetic trend should have analogs");
        assert!((forecast.probabilities.iter().sum::<f64>() - 1.0).abs() < 1e-9);
    }
}
