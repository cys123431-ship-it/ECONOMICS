const fs = require('fs');
const vm = require('vm');

class FakeClassList {
  add() {}
  remove() {}
  toggle() {}
}

class FakeStyle {
  setProperty() {}
}

class FakeElement {
  constructor(tag = 'div', id = '') {
    this.tagName = tag;
    this.id = id;
    this.children = [];
    this.hidden = false;
    this.dataset = {};
    this.className = '';
    this.textContent = '';
    this.style = new FakeStyle();
    this.classList = new FakeClassList();
  }

  append(...nodes) {
    this.children.push(...nodes);
  }

  replaceChildren(...nodes) {
    this.children = [...nodes];
  }

  setAttribute(name, value) {
    this[name] = String(value);
  }

  addEventListener() {}
  querySelector() { return null; }
  querySelectorAll() { return []; }
  closest() { return null; }
  focus() {}
  blur() {}
  select() {}
  scrollIntoView() {}
}

const ids = Object.fromEntries([
  'connectionDot',
  'connectionText',
  'refreshButton',
  'terminalSearch',
  'scopeButton',
  'densityButton',
  'searchResults',
  'tickerTape',
  'errorBanner',
  'overviewAsOf',
  'overviewGauges',
  'marketLights',
  'overviewQuotes',
  'overviewCentralBank',
  'overviewRecovery',
  'overviewMarketMatrix',
  'riskHeatmap',
  'proprietarySignals',
  'sourceHealth',
  'priorityMonitor',
  'favoritesMonitor',
  'forecastOverview',
  'usMarket',
  'koreaMarket',
  'cryptoMarket',
  'fundingMarket',
  'centralBankHero',
  'centralBankChart',
  'centralBankComponents',
  'centralBankFactors',
  'centralBankPlumbing',
  'centralBankJudgement',
  'centralBankMethodology',
  'krLiveKospi',
  'krLiveKosdaq',
  'lastUpdated'
].map((id) => [id, new FakeElement('div', id)]));

const tabNames = ['overview', 'us', 'korea', 'crypto', 'centralbank', 'funding'];
const tabs = tabNames.map((name) => {
  const node = new FakeElement('button');
  node.dataset.tab = name;
  return node;
});
const panels = tabNames.map((name) => new FakeElement('section', `tab-${name}`));
const asofs = [new FakeElement('span'), new FakeElement('span'), new FakeElement('span'), new FakeElement('span')];

const document = {
  body: new FakeElement('body'),
  activeElement: null,
  getElementById(id) {
    return ids[id] || panels.find((panel) => panel.id === id) || null;
  },
  createElement(tag) {
    return new FakeElement(tag);
  },
  createElementNS(_namespace, tag) {
    return new FakeElement(tag);
  },
  createTextNode(text) {
    return { textContent: String(text) };
  },
  querySelectorAll(selector) {
    if (selector === '.tab-button') return tabs;
    if (selector === '.tab-panel') return panels;
    if (selector === '.market-asof') return asofs;
    return [];
  },
  querySelector(selector) {
    if (selector === '.market-tabs') return new FakeElement('nav');
    return null;
  },
  addEventListener() {}
};

const localStorageValues = new Map();
const localStorage = {
  getItem(key) { return localStorageValues.has(key) ? localStorageValues.get(key) : null; },
  setItem(key, value) { localStorageValues.set(key, String(value)); }
};

const context = {
  console,
  document,
  localStorage,
  fetch: () => new Promise(() => {}),
  setInterval: () => 0,
  setTimeout: () => 0,
  Intl,
  Date,
  Number,
  Math,
  Object,
  Array,
  String,
  Boolean,
  Error,
  Promise
};

vm.createContext(context);
vm.runInContext(
  fs.readFileSync('src/dashboard.js', 'utf8'),
  context,
  { filename: 'dashboard.js' }
);

if (context.finite(null) !== null || context.finite(undefined) !== null || context.finite('') !== null) {
  throw new Error('missing dashboard values must not be coerced to zero');
}

const payload = {
  snapshot: {
    as_of: '2026-08-21T07:00:00Z',
    global_risk: 42,
    confidence: 81,
    diffusion: 2,
    stress: 35,
    vulnerability: 48,
    resilience: 61,
    stage: 1,
    data_quality: 82,
    rules_triggered: 17,
    markets: {
      US_EQUITY: 41,
      KOREA_EQUITY: 37,
      CRYPTO: 53
    },
    nodes: {
      VALUATION: 48,
      BUSINESS_DEBT: 42,
      CREDIT: 38,
      VOLATILITY: 32,
      FINCOND: 35,
      LEVERAGE: 45,
      RATES: 39,
      BANKING: 28,
      TREASURY_AUCTION: 31,
      FOREIGN_TREASURY_DEMAND: 36,
      GROWTH: 29,
      LABOR: 34,
      HOUSING: 40,
      KOREA_FIN_STAB: 37,
      KOREA_MARKET_INTERNALS: 41,
      KOREA_MACRO: 33,
      CRYPTO_DERIVATIVES: 55,
      USD: 44,
      LIQUIDITY: 46,
      FUNDING: 52
    },
    sources: {
      FRED: { fresh: true },
      KRX: { fresh: true },
      BINANCE: { fresh: true }
    }
  },
  dashboard: {
    indicators: [],
    central_bank: {
      score: 28,
      long_bias: 64,
      short_bias: 36,
      confidence: 100,
      signal: '롱 우세',
      regime: '축소 중이나 완화 · 롱 방향 개선',
      summary: 'G3 자산증가율 수준과 가속도, Fed 순유동성을 결합한 방향성입니다.',
      as_of: '2026-08-01',
      g3_total_usd_trillion: 19.2,
      g3_yoy: -1.2,
      g3_usd_yoy: 0.8,
      g3_annualized_3m: 2.4,
      g3_yoy_acceleration_3m: 1.1,
      components: [{
        key: 'fed', name: '미국 연방준비제도 총자산', series: 'WALCL',
        source_series: 'fred:WALCL', native_unit: '백만 달러', native_value: 6600000,
        usd_trillion: 6.6, yoy: -2.1, change_1m: 0.2, annualized_3m: 1.0,
        yoy_acceleration_3m: 0.8, observed_at: '2026-08-19', freshness: '공식 최신 · 8일 시차',
        increase_meaning: '달러 준비금 완충력이 늘어나는 방향입니다.',
        decrease_meaning: '준비금 완충력이 줄어드는 방향입니다.',
        interpretation: '전년보다 감소했지만 감소 속도는 완화됐습니다.'
      }],
      factors: [{
        key: 'g3_yoy', label: 'G3 자산증가율 YoY · 검은선 수준', value: -1.2,
        unit: '%', score: -12, weight: 30, contribution: -3.6,
        source_series: 'fred:WALCL+ECBASSETSW+JPNASSETS', observed_at: '2026-08-01',
        increase_meaning: '높아질수록 롱 성향이 강해집니다.',
        decrease_meaning: '낮아질수록 숏 성향이 강해집니다.',
        interpretation: '현재 수준은 축소 방향입니다.'
      }],
      fed_plumbing: {
        net_liquidity_usd_trillion: 5.7, net_liquidity_yoy: 1.2,
        net_liquidity_3m_annualized: 3.4, reserve_balances_usd_trillion: 3.1,
        reserve_balances_3m_annualized: 2.3, reverse_repo_usd_trillion: 0.1,
        treasury_general_account_usd_trillion: 0.8, observed_at: '2026-08-19',
        formula: 'Fed 총자산 − 미 재무부 TGA − ON RRP', explanation: '시장 배관을 보는 보조치입니다.'
      },
      history: [
        { observed_at: '2025-08-01', fed_yoy: -4, ecb_yoy: -5, boj_yoy: -1, g3_yoy: -3.8, g3_usd_yoy: -2 },
        { observed_at: '2026-08-01', fed_yoy: -2.1, ecb_yoy: -1, boj_yoy: 0.2, g3_yoy: -1.2, g3_usd_yoy: 0.8 }
      ],
      methodology: ['공식 총자산 증가율을 가중합니다.'],
      caveats: ['수익 확률이 아닙니다.']
    },
    forecasts: [{
      market: 'us', label: '미국 S&P 500', source_series: 'fred:SP500',
      as_of: '2026-08-28', status: 'EXPERIMENTAL / WALK-FORWARD CHECKED',
      methodology: '현재와 유사한 과거 국면을 당시 이용 가능 데이터만으로 순차 검증합니다.',
      current_regime: { return_1w: 1.2, return_1m: 2.4, return_3m: 4.8, realized_volatility_20d: 15.1 },
      horizons: [{
        horizon_days: 5, flat_band_percent: 0.56,
        rise_probability: 46, sideways_probability: 29, fall_probability: 25,
        dominant_direction: '상승', expected_return_percent: 0.7, analog_samples: 80,
        average_distance: 0.9, validation_samples: 120, validation_hit_rate: 44,
        validation_brier: 0.61, uniform_brier: 0.667,
        validation_state: 'VALIDATED / BEATS UNIFORM',
        warning: '유사국면의 조건부 빈도이며 보장된 수익확률이 아닙니다.'
      }]
    }]
  }
};

const errors = context.render(payload);
if (errors.length) {
  throw new Error(`render errors: ${errors.join(' | ')}`);
}

for (const id of [
  'overviewGauges',
  'riskHeatmap',
  'usMarket',
  'koreaMarket',
  'cryptoMarket',
  'fundingMarket',
  'overviewCentralBank',
  'centralBankHero',
  'centralBankChart',
  'centralBankJudgement'
]) {
  if (!ids[id].children.length) {
    throw new Error(`${id} rendered no children`);
  }
}

context.selectTab('korea');
const koreaPanel = panels.find((panel) => panel.id === 'tab-korea');
const usPanel = panels.find((panel) => panel.id === 'tab-us');
if (koreaPanel.hidden || !usPanel.hidden) {
  throw new Error('tab selection contract failed');
}

context.selectTab('centralbank');
const centralBankPanel = panels.find((panel) => panel.id === 'tab-centralbank');
if (centralBankPanel.hidden || !koreaPanel.hidden) {
  throw new Error('central bank tab selection contract failed');
}

context.selectTab('funding');
const fundingPanel = panels.find((panel) => panel.id === 'tab-funding');
if (fundingPanel.hidden || !centralBankPanel.hidden) {
  throw new Error('funding tab selection contract failed');
}

if (fs.readFileSync('src/dashboard.js', 'utf8').includes('NODE_LABELS')) {
  throw new Error('legacy NODE_LABELS reference remains');
}

const directionIndicators = {
  sp500: { change_pct: 0.60 },
  nasdaq: { change_pct: 0.90 },
  dow: { change_pct: -0.10 },
  kospi: { change_pct: -0.40 },
  kosdaq: { change_pct: -0.20 },
  btc_spot_change: { value: 0.10 }
};
const usDirection = context.priceDirectionModel('us', directionIndicators);
if (usDirection.state.label !== '상승' || Math.abs(usDirection.returnPct - (1.4 / 3)) > 1e-9) {
  throw new Error(`US price direction mismatch: ${usDirection.state.label} ${usDirection.returnPct}`);
}
const koreaDirection = context.priceDirectionModel('korea', directionIndicators);
if (koreaDirection.state.label !== '하락' || Math.abs(koreaDirection.returnPct + 0.3) > 1e-9) {
  throw new Error(`Korea price direction mismatch: ${koreaDirection.state.label} ${koreaDirection.returnPct}`);
}
const cryptoDirection = context.priceDirectionModel('crypto', directionIndicators);
if (cryptoDirection.state.label !== '횡보') {
  throw new Error(`crypto sideways band mismatch: ${cryptoDirection.state.label}`);
}
const overallDirection = context.overallPriceDirection([usDirection, koreaDirection, cryptoDirection]);
if (overallDirection.state.label !== '횡보') {
  throw new Error(`overall tied direction must be sideways, got ${overallDirection.state.label}`);
}
if (context.priceDirectionModel('us', {}).state.label !== '데이터 부족') {
  throw new Error('missing price direction data must remain unknown');
}

const html = fs.readFileSync('src/dashboard.html', 'utf8');
const dashboardJs = fs.readFileSync('src/dashboard.js', 'utf8');
for (const id of ['krLiveKospi', 'krLiveKosdaq']) {
  if (!html.includes(`id="${id}"`)) {
    throw new Error(`Korean live index card is missing ${id}`);
  }
}
for (const id of ['terminalSearch', 'forecastOverview', 'fundingMarket']) {
  if (!html.includes(`id="${id}"`)) {
    throw new Error(`v0.9 terminal control is missing ${id}`);
  }
}
if (!html.includes('F6 자금·포지션')) {
  throw new Error('F6 funding and positioning workspace is missing');
}
if (!dashboardJs.includes('Brier / 균등') || !dashboardJs.includes('방향판단에 사용 금지')) {
  throw new Error('forecast validation boundary is not explicit');
}
if (html.includes('tradingview-widget.com')) {
  throw new Error('restricted TradingView widget must not be loaded');
}
if (!fs.readFileSync('src/dashboard.js', 'utf8').includes("fetch('/api/kr-indices-live'")) {
  throw new Error('Korean live index endpoint is not loaded');
}
if (!html.includes('위기 점수와 공식 지표 계산에는 사용하지 않음')) {
  throw new Error('external quote calculation boundary is not disclosed');
}
const ambiguousMarketLabels = ['탈출 준비', '회복 관찰', '위기 탈출 목표 접근도'];
for (const phrase of ambiguousMarketLabels) {
  if (dashboardJs.includes(phrase) || html.includes(phrase)) {
    throw new Error(`ambiguous market label remains: ${phrase}`);
  }
}
if (!html.includes('시장별 상승·하락·횡보')) {
  throw new Error('explicit price-direction heading is missing');
}
console.log('dashboard smoke test passed');
