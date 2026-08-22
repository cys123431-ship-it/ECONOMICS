use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct FuturesStats {
    pub regular_open_interest: f64,
    pub front_month_basis: Option<f64>,
    pub front_contract: Option<String>,
    pub regular_contracts: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OptionsStats {
    pub put_call_ratio: Option<f64>,
    pub active_implied_volatility: Option<f64>,
    pub regular_open_interest: f64,
    pub expiry: Option<String>,
    pub active_contracts: usize,
    pub put_volume: f64,
    pub call_volume: f64,
}

fn number(value: Option<&Value>) -> Option<f64> {
    match value? {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.replace(',', "").trim().parse().ok(),
        _ => None,
    }
}

fn text<'a>(row: &'a Value, field: &str) -> &'a str {
    row.get(field).and_then(Value::as_str).unwrap_or("").trim()
}

fn contract_month(name: &str) -> Option<String> {
    name.split_whitespace()
        .find(|token| {
            token.len() == 6 && token.starts_with("20") && token.chars().all(|c| c.is_ascii_digit())
        })
        .map(str::to_string)
}

fn is_regular_futures_contract(row: &Value) -> bool {
    text(row, "PROD_NM") == "코스피200 선물"
        && text(row, "MKT_NM") == "정규"
        && !text(row, "ISU_NM").contains(" SP ")
        && contract_month(text(row, "ISU_NM")).is_some()
}

pub fn futures_stats(rows: &[Value]) -> FuturesStats {
    let regular = rows
        .iter()
        .filter(|row| is_regular_futures_contract(row))
        .collect::<Vec<_>>();
    let regular_open_interest = regular
        .iter()
        .filter_map(|row| {
            number(row.get("ACC_OPNINT_QTY"))
                .or_else(|| number(row.get("OPNINT_QTY")))
                .or_else(|| number(row.get("OPEN_INT")))
        })
        .sum();
    let front_month = regular
        .iter()
        .filter_map(|row| contract_month(text(row, "ISU_NM")))
        .min();
    let front = front_month.as_ref().and_then(|month| {
        regular
            .iter()
            .filter(|row| contract_month(text(row, "ISU_NM")).as_ref() == Some(month))
            .max_by(|left, right| {
                number(left.get("ACC_TRDVOL"))
                    .unwrap_or(0.0)
                    .total_cmp(&number(right.get("ACC_TRDVOL")).unwrap_or(0.0))
            })
    });
    let front_month_basis = front.and_then(|row| {
        let derivative = number(row.get("SETL_PRC")).or_else(|| number(row.get("TDD_CLSPRC")))?;
        let spot = number(row.get("SPOT_PRC"))?;
        Some(derivative - spot)
    });
    FuturesStats {
        regular_open_interest,
        front_month_basis,
        front_contract: front.map(|row| text(row, "ISU_NM").to_string()),
        regular_contracts: regular.len(),
    }
}

fn is_regular_option(row: &Value) -> bool {
    text(row, "PROD_NM") == "코스피200 옵션"
        && text(row, "ISU_NM").contains("(정규)")
        && contract_month(text(row, "ISU_NM")).is_some()
}

pub fn options_stats(rows: &[Value]) -> OptionsStats {
    let regular = rows
        .iter()
        .filter(|row| is_regular_option(row))
        .collect::<Vec<_>>();
    let expiry = regular
        .iter()
        .filter(|row| number(row.get("ACC_TRDVOL")).unwrap_or(0.0) > 0.0)
        .filter_map(|row| contract_month(text(row, "ISU_NM")))
        .min()
        .or_else(|| {
            regular
                .iter()
                .filter_map(|row| contract_month(text(row, "ISU_NM")))
                .min()
        });
    let selected = regular
        .iter()
        .filter(|row| contract_month(text(row, "ISU_NM")) == expiry)
        .collect::<Vec<_>>();
    let mut put_volume = 0.0;
    let mut call_volume = 0.0;
    let mut iv_weighted = 0.0;
    let mut iv_weight = 0.0;
    let mut active_contracts = 0usize;
    for row in &selected {
        let volume = number(row.get("ACC_TRDVOL"))
            .or_else(|| number(row.get("TRD_VOL")))
            .unwrap_or(0.0);
        let side = text(row, "RGHT_TP_NM").to_ascii_uppercase();
        if side == "PUT" || side == "풋" {
            put_volume += volume;
        } else if side == "CALL" || side == "콜" {
            call_volume += volume;
        }
        if volume > 0.0 {
            if let Some(iv) = number(row.get("IMP_VOLT")).filter(|iv| *iv > 0.0) {
                iv_weighted += iv * volume;
                iv_weight += volume;
                active_contracts += 1;
            }
        }
    }
    let regular_open_interest = selected
        .iter()
        .filter_map(|row| number(row.get("ACC_OPNINT_QTY")))
        .sum();
    OptionsStats {
        put_call_ratio: (call_volume > f64::EPSILON).then_some(put_volume / call_volume),
        active_implied_volatility: (iv_weight > f64::EPSILON).then_some(iv_weighted / iv_weight),
        regular_open_interest,
        expiry,
        active_contracts,
        put_volume,
        call_volume,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn futures_exclude_night_and_calendar_spreads_and_use_front_regular_contract() {
        let rows = vec![
            json!({"PROD_NM":"코스피200 선물","MKT_NM":"야간","ISU_NM":"코스피200 F 202609 (야간)","SETL_PRC":"1080","SPOT_PRC":"1096.25","ACC_OPNINT_QTY":"150000","ACC_TRDVOL":"30000"}),
            json!({"PROD_NM":"코스피200 선물","MKT_NM":"정규","ISU_NM":"코스피200 F 202609 (주간)","SETL_PRC":"1099.70","SPOT_PRC":"1096.25","ACC_OPNINT_QTY":"152658","ACC_TRDVOL":"117863"}),
            json!({"PROD_NM":"코스피200 선물","MKT_NM":"정규","ISU_NM":"코스피200 F 202612 (주간)","SETL_PRC":"1102.80","SPOT_PRC":"1096.25","ACC_OPNINT_QTY":"8797","ACC_TRDVOL":"4758"}),
            json!({"PROD_NM":"코스피200 선물","MKT_NM":"정규","ISU_NM":"코스피200 SP 202609-202612","SETL_PRC":"3.1","SPOT_PRC":"1096.25","ACC_OPNINT_QTY":"999","ACC_TRDVOL":"99"}),
        ];
        let stats = futures_stats(&rows);
        assert_eq!(stats.regular_open_interest, 161455.0);
        assert!((stats.front_month_basis.unwrap() - 3.45).abs() < 1e-9);
        assert_eq!(
            stats.front_contract.as_deref(),
            Some("코스피200 F 202609 (주간)")
        );
        assert_eq!(stats.regular_contracts, 2);
    }

    #[test]
    fn options_use_only_regular_front_month_and_positive_volume_for_iv() {
        let rows = vec![
            json!({"PROD_NM":"코스피200 옵션","ISU_NM":"코스피200 P 202609 1000.0 (정규)","RGHT_TP_NM":"PUT","ACC_TRDVOL":"300","IMP_VOLT":"40","ACC_OPNINT_QTY":"10"}),
            json!({"PROD_NM":"코스피200 옵션","ISU_NM":"코스피200 C 202609 1100.0 (정규)","RGHT_TP_NM":"CALL","ACC_TRDVOL":"200","IMP_VOLT":"20","ACC_OPNINT_QTY":"20"}),
            json!({"PROD_NM":"코스피200 옵션","ISU_NM":"코스피200 C 202609 1200.0 (정규)","RGHT_TP_NM":"CALL","ACC_TRDVOL":"0","IMP_VOLT":"99","ACC_OPNINT_QTY":"30"}),
            json!({"PROD_NM":"코스피200 옵션","ISU_NM":"코스피200 P 202609 1000.0 (야간)","RGHT_TP_NM":"PUT","ACC_TRDVOL":"900","IMP_VOLT":"80","ACC_OPNINT_QTY":"100"}),
            json!({"PROD_NM":"코스피200 옵션","ISU_NM":"코스피200 P 202610 1000.0 (정규)","RGHT_TP_NM":"PUT","ACC_TRDVOL":"1000","IMP_VOLT":"70","ACC_OPNINT_QTY":"200"}),
        ];
        let stats = options_stats(&rows);
        assert_eq!(stats.expiry.as_deref(), Some("202609"));
        assert_eq!(stats.put_call_ratio, Some(1.5));
        assert_eq!(stats.regular_open_interest, 60.0);
        assert!((stats.active_implied_volatility.unwrap() - 32.0).abs() < 1e-9);
        assert_eq!(stats.active_contracts, 2);
    }
}
