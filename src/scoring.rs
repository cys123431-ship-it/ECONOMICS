pub fn clamp(value: f64) -> f64 {
    value.clamp(0.0, 100.0)
}

pub fn percentile_rank(history: &[f64], value: f64) -> Option<f64> {
    let finite = history
        .iter()
        .copied()
        .filter(|sample| sample.is_finite())
        .collect::<Vec<_>>();
    if finite.is_empty() || !value.is_finite() {
        return None;
    }
    let less = finite.iter().filter(|sample| **sample < value).count() as f64;
    let equal = finite
        .iter()
        .filter(|sample| (**sample - value).abs() <= f64::EPSILON)
        .count() as f64;
    Some(100.0 * (less + 0.5 * equal) / finite.len() as f64)
}

pub fn risk_from_history(
    history: &[f64],
    current: f64,
    invert: bool,
    min_samples: usize,
) -> Option<f64> {
    if history.iter().filter(|value| value.is_finite()).count() < min_samples {
        return None;
    }
    let mut percentile = percentile_rank(history, current)?;
    if invert {
        percentile = 100.0 - percentile;
    }
    Some(clamp(percentile))
}

pub fn mean(values: &[f64]) -> Option<f64> {
    let finite = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .collect::<Vec<_>>();
    (!finite.is_empty()).then(|| finite.iter().sum::<f64>() / finite.len() as f64)
}

pub fn weighted_mean(values: &[(f64, f64)]) -> Option<f64> {
    let (weighted_sum, weight_sum) = values
        .iter()
        .filter(|(value, weight)| value.is_finite() && weight.is_finite() && *weight > 0.0)
        .fold((0.0, 0.0), |(sum, weights), (value, weight)| {
            (sum + value * weight, weights + weight)
        });
    (weight_sum > 0.0).then(|| weighted_sum / weight_sum)
}

pub fn correlation(left: &[f64], right: &[f64]) -> Option<f64> {
    let len = left.len().min(right.len());
    if len < 5 {
        return None;
    }
    let left = &left[left.len() - len..];
    let right = &right[right.len() - len..];
    let left_mean = left.iter().sum::<f64>() / len as f64;
    let right_mean = right.iter().sum::<f64>() / len as f64;
    let mut covariance = 0.0;
    let mut left_variance = 0.0;
    let mut right_variance = 0.0;
    for (left, right) in left.iter().zip(right) {
        let left_delta = left - left_mean;
        let right_delta = right - right_mean;
        covariance += left_delta * right_delta;
        left_variance += left_delta * left_delta;
        right_variance += right_delta * right_delta;
    }
    let denominator = (left_variance * right_variance).sqrt();
    (denominator > f64::EPSILON).then(|| covariance / denominator)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ties_use_midrank() {
        assert_eq!(percentile_rank(&[5.0; 20], 5.0), Some(50.0));
    }

    #[test]
    fn immature_history_is_unknown() {
        assert_eq!(risk_from_history(&[1.0, 2.0], 3.0, false, 20), None);
    }

    #[test]
    fn current_is_not_implicitly_added_to_history() {
        assert_eq!(percentile_rank(&[1.0, 2.0, 3.0, 4.0], 10.0), Some(100.0));
    }
}
