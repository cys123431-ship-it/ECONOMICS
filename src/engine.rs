use crate::{db::Db,dsl::{self,Context},rulebook,scoring};
use serde::{Deserialize,Serialize};
use std::collections::HashMap;

#[derive(Debug,Serialize,Deserialize)]
pub struct Snapshot{pub ts:String,pub global_risk:f64,pub stress:f64,pub vulnerability:f64,pub resilience:f64,pub confidence:f64,pub diffusion:usize,pub stage:u8,pub markets:HashMap<String,f64>,pub causes:Vec<String>,pub rules_evaluated:usize,pub rule_hits:usize}

const METRICS:&[(&str,&str,&str,bool,&str)]=&[
("fred","STLFSI4","FINCOND",false,"stress"),("fred","NFCI","FINCOND",false,"stress"),("fred","BAMLH0A0HYM2","CREDIT",false,"stress"),("fred","VIXCLS","VOLATILITY",false,"stress"),("fred","T10Y2Y","RATES",true,"vulnerability"),("fred","NFCILEVERAGE","LEVERAGE",false,"vulnerability"),("fred","ICSA","LABOR",false,"vulnerability"),("fred","WEI","GROWTH",true,"vulnerability"),("binance","BTC_FUNDING","CRYPTO_DERIVATIVES",false,"stress"),("binance","BTC_OI","CRYPTO_DERIVATIVES",false,"vulnerability"),("treasury","AUCTION_BTC","TREASURY_AUCTION",true,"stress"),("ecos","KR_USD_KRW","KOREA_FIN_STAB",false,"stress")];

pub fn run(db:&Db)->Result<Snapshot,Box<dyn std::error::Error>>{
    let mut ctx=Context::default(); let mut stress=Vec::new();let mut vuln=Vec::new();let mut confidence_parts=0usize;let mut diffusion=0usize;let mut causes=Vec::new();
    for (source,series,module,invert,bucket) in METRICS { if let Some(cur)=db.latest(source,series)? { let hist=db.recent(source,series,256)?; let score=scoring::risk_from_z_like(&hist,cur,*invert);ctx.values.insert((*series).into(),cur);ctx.scores.entry((*module).into()).and_modify(|v|*v=(*v+score)/2.0).or_insert(score);confidence_parts+=1;if score>=75.0{diffusion+=1;causes.push(format!("{module}:{score:.0}"));}if *bucket=="stress"{stress.push(score)}else{vuln.push(score)} }}
    let stress_s=scoring::mean(&stress);let vuln_s=scoring::mean(&vuln);let resilience=scoring::clamp(100.0-(0.35*stress_s+0.25*vuln_s));let global=scoring::clamp(0.55*stress_s+0.45*vuln_s+0.15*(100.0-resilience));
    ctx.scores.insert("STRESS_SCORE".into(),stress_s);ctx.scores.insert("VULNERABILITY_SCORE".into(),vuln_s);ctx.scores.insert("RESILIENCE_SCORE".into(),resilience);ctx.scores.insert("GLOBAL_RISK".into(),global);
    let (evaluated,hits)=rulebook::stream_evaluate(|cond|dsl::eval(cond,&ctx),64)?;for h in &hits{causes.push(format!("{}: {}",h.id,h.title));}
    let confidence=(confidence_parts as f64/METRICS.len() as f64*100.0).clamp(0.0,100.0);let stage=if global>=90.0{5}else if global>=75.0{4}else if global>=60.0{3}else if global>=45.0{2}else if global>=25.0{1}else{0};
    let mut markets=HashMap::new();markets.insert("US_EQUITY".into(),scoring::clamp(0.45*stress_s+0.55*vuln_s));markets.insert("CRYPTO".into(),ctx.scores.get("CRYPTO_DERIVATIVES").copied().unwrap_or(global));markets.insert("KOREA_EQUITY".into(),ctx.scores.get("KOREA_FIN_STAB").copied().unwrap_or(global));
    let s=Snapshot{ts:chrono::Utc::now().to_rfc3339(),global_risk:global,stress:stress_s,vulnerability:vuln_s,resilience,confidence,diffusion,stage,markets,causes,rules_evaluated:evaluated,rule_hits:hits.len()};db.save_snapshot(&s)?;Ok(s)
}
