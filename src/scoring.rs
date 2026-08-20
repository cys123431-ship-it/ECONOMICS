pub fn percentile_rank(history:&[f64], x:f64) -> f64 {
    if history.is_empty() { return 50.0; }
    let n=history.iter().filter(|v| **v <= x).count();
    100.0 * n as f64 / history.len() as f64
}

pub fn clamp(v:f64)->f64 { v.clamp(0.0,100.0) }

pub fn risk_from_z_like(history:&[f64], current:f64, invert:bool)->f64 {
    let mut p=percentile_rank(history,current);
    if invert { p=100.0-p; }
    clamp(p)
}

pub fn mean(xs:&[f64])->f64 { if xs.is_empty(){0.0}else{xs.iter().sum::<f64>()/xs.len() as f64} }
