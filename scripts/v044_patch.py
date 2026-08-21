from pathlib import Path
import re


def replace_once(path: str, old: str, new: str):
    p = Path(path)
    text = p.read_text(encoding="utf-8")
    if old not in text:
        raise SystemExit(f"pattern not found in {path}: {old[:80]!r}")
    p.write_text(text.replace(old, new, 1), encoding="utf-8")


def regex_once(path: str, pattern: str, replacement: str):
    p = Path(path)
    text = p.read_text(encoding="utf-8")
    out, count = re.subn(pattern, replacement, text, count=1, flags=re.S)
    if count != 1:
        raise SystemExit(f"regex expected one match in {path}, got {count}")
    p.write_text(out, encoding="utf-8")


# Version.
replace_once('Cargo.toml', 'version = "0.4.3"', 'version = "0.4.4"')
lock = Path('Cargo.lock')
lock_text = lock.read_text(encoding='utf-8')
lock_text = re.sub(r'(name = "economics-radar"\nversion = ")[^"]+("\n)', r'\g<1>0.4.4\2', lock_text, count=1)
lock.write_text(lock_text, encoding='utf-8')

# Risk heatmap: make market ownership explicit and visually distinct.
node_meta = r'''const NODE_META = {
  VALUATION: { label: '주식 밸류에이션', market: 'us' },
  BUSINESS_DEBT: { label: '기업부채', market: 'us' },
  CREDIT: { label: '신용시장', market: 'us' },
  VOLATILITY: { label: '변동성', market: 'us' },
  FINCOND: { label: '금융여건', market: 'us' },
  LEVERAGE: { label: '레버리지', market: 'us' },
  RATES: { label: '금리', market: 'us' },
  BANKING: { label: '은행 스트레스', market: 'us' },
  TREASURY_AUCTION: { label: '미 국채 입찰', market: 'us' },
  FOREIGN_TREASURY_DEMAND: { label: '해외 미 국채 수요', market: 'us' },
  GROWTH: { label: '성장', market: 'us' },
  LABOR: { label: '고용', market: 'us' },
  HOUSING: { label: '주택시장', market: 'us' },
  KOREA_FIN_STAB: { label: '금융안정', market: 'korea' },
  KOREA_MARKET_INTERNALS: { label: '시장 내부수급', market: 'korea' },
  KOREA_MACRO: { label: '거시경제', market: 'korea' },
  CRYPTO_DERIVATIVES: { label: '파생시장', market: 'crypto' },
  USD: { label: '달러', market: 'global' },
  LIQUIDITY: { label: '유동성', market: 'global' },
  FUNDING: { label: '자금조달', market: 'global' }
};

const HEATMAP_MARKETS = {
  us: { label: '미국 시장', short: '미국' },
  korea: { label: '한국 시장', short: '한국' },
  crypto: { label: '코인 시장', short: '코인' },
  global: { label: '글로벌 공통', short: '글로벌' }
};'''
regex_once(
    'src/dashboard.js',
    r"const NODE_LABELS = \{.*?\n\};",
    node_meta,
)

heatmap_fn = r'''function renderRiskHeatmap(nodes) {
  const container = $('riskHeatmap');
  clear(container);
  const grouped = { us: [], korea: [], crypto: [], global: [] };
  for (const [name, value] of Object.entries(nodes)) {
    if (finite(value) === null) continue;
    const meta = NODE_META[name] || { label: name, market: 'global' };
    (grouped[meta.market] || grouped.global).push([name, value, meta]);
  }

  for (const market of ['us', 'korea', 'crypto', 'global']) {
    const entries = grouped[market].sort((a, b) => Number(b[1]) - Number(a[1]));
    if (!entries.length) continue;
    const info = HEATMAP_MARKETS[market];
    const group = el('section', `heat-group market-${market}`);
    const header = el('header', 'heat-group-header');
    header.append(el('strong', '', info.label), el('span', '', `${entries.length}개 신호`));
    const grid = el('div', 'heat-grid');

    for (const [, value, meta] of entries) {
      const state = riskState(value);
      const cell = el('div', 'heat-cell');
      cell.style.setProperty('--risk-color', state.color);
      cell.style.background = `color-mix(in srgb, ${state.color} ${Math.round(12 + clamp(value) * .28)}%, #070707)`;
      const top = el('div', 'heat-cell-top');
      top.append(el('span', 'heat-market-tag', info.short), el('span', 'heat-risk-state', state.label));
      cell.append(top, el('span', 'heat-node-label', meta.label), el('strong', '', score(value)));
      grid.append(cell);
    }
    group.append(header, grid);
    container.append(group);
  }
}'''
regex_once(
    'src/dashboard.js',
    r"function renderRiskHeatmap\(nodes\) \{.*?\n\}\n\nfunction renderProprietary",
    heatmap_fn + "\n\nfunction renderProprietary",
)

# Distinct market colors, while keeping red/yellow/green for risk severity.
replace_once(
    'src/dashboard.css',
    '  --cyan: #29c7e8;\n',
    '  --cyan: #29c7e8;\n  --market-us: #4ea1ff;\n  --market-korea: #c66bff;\n  --market-crypto: #ff9f00;\n  --market-global: #29c7e8;\n',
)
regex_once(
    'src/dashboard.css',
    r"\.risk-heatmap \{.*?\.heat-cell strong \{ font-size: 17px; \}",
    r'''.risk-heatmap { padding: 8px; display: grid; gap: 8px; }
.heat-group { --market-accent: var(--market-global); border: 1px solid #2d2d2d; background: #050505; }
.heat-group.market-us { --market-accent: var(--market-us); }
.heat-group.market-korea { --market-accent: var(--market-korea); }
.heat-group.market-crypto { --market-accent: var(--market-crypto); }
.heat-group.market-global { --market-accent: var(--market-global); }
.heat-group-header { min-height: 30px; padding: 6px 8px; display: flex; align-items: center; justify-content: space-between; gap: 8px; border-bottom: 1px solid color-mix(in srgb, var(--market-accent) 65%, #252525); background: color-mix(in srgb, var(--market-accent) 9%, #090909); }
.heat-group-header strong { color: var(--market-accent); font-size: 10px; }
.heat-group-header span { color: var(--muted); font-size: 8px; }
.heat-grid { padding: 6px; display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 5px; }
.heat-cell { min-height: 72px; padding: 7px 8px; border: 1px solid var(--risk-color, #333); border-left: 4px solid var(--market-accent); display: flex; flex-direction: column; justify-content: space-between; }
.heat-cell-top { display: flex; align-items: center; justify-content: space-between; gap: 6px; }
.heat-market-tag { color: var(--market-accent) !important; font-size: 8px !important; font-weight: 900; }
.heat-risk-state { color: var(--risk-color) !important; font-size: 8px !important; }
.heat-node-label { color: #eee !important; font-size: 10px !important; font-weight: 700; white-space: normal !important; overflow: visible !important; }
.heat-cell strong { font-size: 17px; }''',
)

# Ticker: do not imply that yesterday's KRX EOD value is live. KRX's official
# distribution page states that daytime EOD is transmitted at 16:00 and 18:10.
replace_once(
    'src/server.rs',
    'use crate::{dashboard, db::Db, refresh::RefreshControl};\nuse serde_json::{json, Value};',
    'use crate::{dashboard, db::Db, refresh::RefreshControl};\nuse chrono::{Datelike, FixedOffset, NaiveDate, Timelike, Utc, Weekday};\nuse serde_json::{json, Value};',
)
old = '''        let freshness = indicator
            .get("freshness")
            .and_then(Value::as_str)
            .unwrap_or("UNKNOWN")
            .to_string();
        let date = indicator
            .get("observed_at")
            .and_then(Value::as_str)
            .and_then(|value| value.get(..10))
            .unwrap_or("NO DATE")
            .to_string();'''
new = '''        let mut freshness = indicator
            .get("freshness")
            .and_then(Value::as_str)
            .unwrap_or("UNKNOWN")
            .to_string();
        let date = indicator
            .get("observed_at")
            .and_then(Value::as_str)
            .and_then(|value| value.get(..10))
            .unwrap_or("NO DATE")
            .to_string();
        let source = indicator
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if source == "KRX" {
            let kst = FixedOffset::east_opt(9 * 60 * 60).expect("KST offset is valid");
            let now = Utc::now().with_timezone(&kst);
            let today = now.date_naive();
            let observed = NaiveDate::parse_from_str(&date, "%Y-%m-%d").ok();
            if observed == Some(today) {
                freshness = "KRX EOD TODAY".into();
            } else if observed.is_some_and(|value| value < today)
                && !matches!(now.weekday(), Weekday::Sat | Weekday::Sun)
            {
                let minute = now.hour() * 60 + now.minute();
                freshness = if minute < 16 * 60 {
                    "KRX EOD(오늘 16:00 이후)".into()
                } else if minute < 18 * 60 + 15 {
                    "KRX EOD(당일 공개 확인 중)".into()
                } else {
                    "KRX EOD(최신 공개 종가)".into()
                };
            }
        }'''
replace_once('src/server.rs', old, new)

# Release notes / docs.
readme = Path('README.md')
text = readme.read_text(encoding='utf-8')
needle = '# ECONOMICS Radar'
if needle in text and 'v0.4.4' not in text:
    text = text.replace(needle, needle + '\n\n> v0.4.4: risk heatmap market grouping and explicit KRX EOD publication status.', 1)
readme.write_text(text, encoding='utf-8')
