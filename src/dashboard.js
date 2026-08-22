function $(id) {
  const node = document.getElementById(id);
  if (!node) throw new Error(`DOM element #${id} not found`);
  return node;
}

const NODE_META = {
  VALUATION: { label: '미국 주식 가격·밸류에이션', market: 'us' },
  BUSINESS_DEBT: { label: '미국 대출·연체 취약성', market: 'us' },
  CREDIT: { label: '신용시장', market: 'us' },
  VOLATILITY: { label: '시장 변동성', market: 'us' },
  FINCOND: { label: '금융여건', market: 'us' },
  LEVERAGE: { label: '레버리지', market: 'us' },
  RATES: { label: '금리', market: 'us' },
  BANKING: { label: '은행 스트레스', market: 'us' },
  TREASURY_AUCTION: { label: '미 국채 입찰', market: 'us' },
  FOREIGN_TREASURY_DEMAND: { label: '해외 미 국채 수요', market: 'us' },
  GROWTH: { label: '실물 성장', market: 'us' },
  LABOR: { label: '고용시장', market: 'us' },
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
};

const FACTOR_LABEL_OVERRIDES = {
  us: {
    VALUATION: '미국 주식 가격·밸류에이션',
    BUSINESS_DEBT: '미국 대출·연체 취약성',
    CREDIT: '미국 신용시장',
    VOLATILITY: '미국 시장 변동성',
    FINCOND: '미국 금융여건',
    LEVERAGE: '미국 레버리지',
    RATES: '미국 금리',
    BANKING: '미국 은행 스트레스',
    TREASURY_AUCTION: '미 국채 입찰',
    FOREIGN_TREASURY_DEMAND: '해외 미 국채 수요',
    GROWTH: '미국 성장',
    LABOR: '미국 고용시장',
    HOUSING: '미국 주택시장',
    USD: '달러·미국 금융환경'
  },
  korea: {
    KOREA_FIN_STAB: '한국 금융안정',
    KOREA_MARKET_INTERNALS: '한국 시장 내부수급',
    KOREA_MACRO: '한국 거시경제',
    USD: '달러·원화 외부요인',
    LIQUIDITY: '글로벌 유동성(한국 영향)',
    CREDIT: '글로벌 신용시장(한국 영향)',
    BANKING: '글로벌 은행 스트레스(한국 영향)',
    RATES: '글로벌 금리(한국 영향)'
  },
  crypto: {
    CRYPTO_DERIVATIVES: '코인 파생시장',
    LIQUIDITY: '글로벌 유동성(코인 영향)',
    USD: '달러(코인 영향)',
    LEVERAGE: '시장 레버리지(코인 영향)',
    VOLATILITY: '시장 변동성(코인 영향)',
    FUNDING: '코인 자금조달·펀딩'
  }
};

const MARKET_CONFIG = {
  us: {
    title: '미국 시장',
    riskKey: 'US_EQUITY',
    target: 'usMarket',
    factors: [
      'VALUATION', 'BUSINESS_DEBT', 'CREDIT', 'VOLATILITY', 'FINCOND', 'LEVERAGE',
      'RATES', 'BANKING', 'USD', 'TREASURY_AUCTION', 'FOREIGN_TREASURY_DEMAND',
      'GROWTH', 'LABOR', 'HOUSING'
    ],
    sections: [
      ['미국 주식·옵션 공포', 'US EQUITY / VOL', ['sp500', 'nasdaq', 'dow', 'vix']],
      ['미국 국채금리·수익률곡선', 'US RATES / CURVE', ['us10y', 'us2y', 'curve_10y2y', 'curve_10y3m']],
      ['미국 신용스프레드·금융여건', 'US CREDIT / CONDITIONS', ['hy_spread', 'ig_spread', 'ofr_fsi', 'stlfsi', 'nfci', 'anfci', 'nfci_leverage']],
      ['미국 성장·고용 선행지표', 'US GROWTH / LABOR', ['wei', 'cfnai', 'sahm', 'initial_claims', 'continued_claims']],
      ['미국 주택·은행·대출건전성', 'US HOUSING / BANKING', ['mortgage30', 'card_delinquency', 'loan_delinquency', 'bank_capital']],
      ['연준 유동성·은행대출', 'FED LIQUIDITY / CREDIT', ['fed_assets', 'rrp', 'total_reserves', 'reserve_balances', 'business_loans', 'total_loans']],
      ['미 국채 입찰·딜러·담보조건', 'TREASURY / PLUMBING', ['treasury_bid_cover', 'auction_dealer', 'auction_direct', 'auction_indirect', 'dealer_fails', 'margin_tightening']],
      ['달러·글로벌 달러신용', 'USD / GLOBAL CREDIT', ['usd_index', 'usdkrw', 'global_dollar_credit']]
    ]
  },
  korea: {
    title: '한국 시장',
    riskKey: 'KOREA_EQUITY',
    target: 'koreaMarket',
    factors: ['KOREA_FIN_STAB', 'KOREA_MARKET_INTERNALS', 'KOREA_MACRO', 'USD', 'LIQUIDITY', 'CREDIT', 'BANKING', 'RATES'],
    sections: [
      ['한국 주가지수·공식 등락률·환율', 'KR EQUITY / FX', ['kospi', 'kospi_return', 'kosdaq', 'kosdaq_return', 'usdkrw', 'kr_base_rate']],
      ['한국 시장 내부체력·외부수요', 'KR BREADTH / MACRO', ['kospi_breadth', 'kosdaq_breadth', 'krx_breadth', 'kr_cli', 'cn_cli']],
      ['코스피·코스닥 시장규모', 'KR MARKET SCALE', ['kospi_value', 'kosdaq_value', 'kospi_volume', 'kosdaq_volume', 'kospi_cap', 'kosdaq_cap', 'kospi_issues', 'kosdaq_issues']],
      ['코스피200 선물·옵션 원본', 'K200 FUTURES / OPTIONS', ['krx_basis', 'krx_futures_oi', 'krx_futures_volume', 'krx_futures_value', 'krx_put_call', 'krx_option_iv', 'krx_options_oi', 'krx_options_volume', 'krx_options_value']],
      ['한국 채권시장', 'KR FIXED INCOME', ['krx_bond_yield', 'krx_kts_yield', 'krx_bond_basket_yield', 'krx_small_bond_yield', 'krx_bond_duration', 'krx_bond_convexity', 'krx_bond_value', 'krx_kts_value']],
      ['ETF·ETN·ELW 위험선호', 'KR ETP / LEVERAGED', ['etf_breadth', 'etf_value', 'etf_cap', 'etn_breadth', 'etn_value', 'etn_cap', 'elw_breadth', 'elw_value']],
      ['금·석유·배출권 실물시장', 'KR COMMODITIES / ETS', ['gold_value', 'gold_volume', 'oil_price', 'oil_value', 'emissions_breadth', 'emissions_value']],
      ['ESG·SRI·코넥스 보조시장', 'KR SECONDARY MARKETS', ['esg_breadth', 'esg_index_return', 'sri_issues', 'sri_amount', 'konex_breadth', 'konex_cap']]
    ]
  },
  crypto: {
    title: '코인 시장',
    riskKey: 'CRYPTO',
    target: 'cryptoMarket',
    factors: ['CRYPTO_DERIVATIVES', 'LIQUIDITY', 'USD', 'LEVERAGE', 'VOLATILITY', 'FUNDING'],
    sections: [
      ['비트코인 현물 24시간 원본', 'BTC SPOT / 24H', ['btc', 'btc_spot_change', 'btc_spot_high', 'btc_spot_low', 'btc_spot_volume', 'btc_spot_quote_volume']],
      ['BTC 무기한선물 가격·미결제약정', 'BTC PERPETUAL / OI', ['btc_perp_price', 'btc_mark_price', 'btc_index_price', 'btc_oi']],
      ['BTC 펀딩·베이시스 방향과 위험크기', 'BTC FUNDING / BASIS', ['btc_current_funding', 'btc_funding', 'btc_funding_abs', 'btc_basis', 'btc_basis_abs']],
      ['BTC 전체·상위계정 포지셔닝', 'BTC POSITIONING', ['btc_global_ls', 'btc_top_position', 'btc_top_account']],
      ['BTC 공격적 주문흐름', 'BTC TAKER FLOW', ['btc_taker']]
    ]
  }
};

const RECOVERY_CONFIG = {
  overall: {
    title: '종합', riskKey: 'global_risk', prior: 'week',
    conditions: [
      ['stress', '시장 스트레스'], ['vulnerability', '구조적 취약성'],
      ['resilience_risk', '회복탄력성 부족'], ['US_EQUITY', '미국 시장 위험'],
      ['KOREA_EQUITY', '한국 시장 위험'], ['CRYPTO', '코인 시장 위험']
    ]
  },
  us: {
    title: '미국', riskKey: 'US_EQUITY', prior: 'week',
    conditions: [['VALUATION', '주식 가격·밸류에이션'], ['CREDIT', '신용시장'], ['VOLATILITY', '변동성'], ['FINCOND', '금융여건'], ['LEVERAGE', '레버리지']]
  },
  korea: {
    title: '한국', riskKey: 'KOREA_EQUITY', prior: 'week',
    conditions: [['KOREA_FIN_STAB', '금융안정'], ['KOREA_MARKET_INTERNALS', '시장 내부수급'], ['KOREA_MACRO', '거시경제'], ['USD', '달러·원화 외부압력']]
  },
  crypto: {
    title: '코인', riskKey: 'CRYPTO', prior: 'day',
    conditions: [['CRYPTO_DERIVATIVES', '코인 파생시장'], ['LIQUIDITY', '글로벌 유동성'], ['USD', '달러 압력']]
  }
};

const INDICATOR_RULES = {
  vix: [25, 35, '20 미만이면 공포 완화', true],
  hy_spread: [4.5, 5, '4.5% 미만·5일 축소가 탈출 확인', true],
  ig_spread: [1.2, 1.8, '하락 안정이 신용 정상화', true],
  ofr_fsi: [0, 1, '0 미만 10거래일이 안정 조건', true],
  stlfsi: [0, 1, '0 미만이면 장기평균보다 안정', true],
  nfci: [0, 0.5, '0 미만이면 금융여건 완화', true],
  anfci: [0, 0.5, '0 미만이면 경제여건 대비 완화', true],
  sahm: [0.5, 0.75, '0.50%p 미만이 침체신호 해제 조건', true],
  kospi_breadth: [40, 25, '50% 이상 3일이 내부체력 회복', false],
  kosdaq_breadth: [40, 25, '50% 이상 3일이 내부체력 회복', false],
  krx_breadth: [40, 25, '50% 이상이면 상승 확산', false],
  etf_breadth: [40, 25, '50% 이상이면 ETF 위험선호 회복', false],
  etn_breadth: [40, 25, '50% 이상이면 ETN 확산 회복', false],
  krx_basis: [0, -5, '0p 이상 3일이면 선물 위험회피 완화', false],
  krx_put_call: [1.2, 1.6, '중앙 범위 복귀가 옵션 공포 완화', true],
  krx_option_iv: [45, 65, '최근 범위 60백분위 아래가 완화', true],
  curve_10y2y: [0.25, 0, '+0.25%p 이상 지속 시 곡선 정상화', false],
  curve_10y3m: [0.25, 0, '+0.25%p 이상 지속 시 곡선 정상화', false]
};

const TICKER_KEYS = ['usdkrw', 'btc', 'sp500', 'nasdaq', 'dow', 'kospi', 'kosdaq'];
const TAB_NAMES = ['overview', 'us', 'korea', 'crypto'];
let workerWasRunning = false;
let dashboardErrors = [];
let collectionErrors = [];

function factorLabel(config, name) {
  const market = config?.riskKey === 'US_EQUITY'
    ? 'us'
    : config?.riskKey === 'KOREA_EQUITY'
      ? 'korea'
      : config?.riskKey === 'CRYPTO'
        ? 'crypto'
        : null;
  return FACTOR_LABEL_OVERRIDES[market]?.[name] || NODE_META[name]?.label || name;
}

function el(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

function clear(node) {
  node.replaceChildren();
}

function updateErrorBanner() {
  const banner = $('errorBanner');
  const messages = [...dashboardErrors, ...collectionErrors];
  banner.textContent = messages.join(' / ');
  banner.hidden = messages.length === 0;
}

function finite(value) {
  const number = Number(value);
  return Number.isFinite(number) ? number : null;
}

function clamp(value) {
  const number = finite(value);
  return number === null ? 0 : Math.max(0, Math.min(100, number));
}

function score(value, digits = 1) {
  const number = finite(value);
  return number === null ? '—' : number.toFixed(digits);
}

function formatTime(value) {
  if (!value) return '—';
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return String(value);
  return new Intl.DateTimeFormat('ko-KR', {
    dateStyle: 'medium',
    timeStyle: 'short',
    timeZone: 'Asia/Seoul'
  }).format(date);
}

function compact(value, digits = 1) {
  return new Intl.NumberFormat('en-US', {
    notation: 'compact',
    maximumFractionDigits: digits
  }).format(value);
}

function formatValue(indicator) {
  const value = finite(indicator?.value);
  if (value === null) return '—';
  const digits = Number(indicator.decimals ?? 2);
  switch (indicator.unit) {
    case 'usd':
      return Math.abs(value) >= 1e7
        ? `$${compact(value, 2)}`
        : `$${value.toLocaleString('en-US', { maximumFractionDigits: digits })}`;
    case 'krw':
      return `₩${value.toLocaleString('ko-KR', { maximumFractionDigits: digits })}`;
    case 'percent':
      return `${value.toFixed(digits)}%`;
    case 'rate':
      return `${(value * 100).toFixed(digits)}%`;
    case 'fraction_percent':
      return `${(value * 100).toFixed(digits)}%`;
    case 'usd_million':
      return value >= 1e6
        ? `$${(value / 1e6).toFixed(2)}조`
        : `$${(value / 1e3).toFixed(2)}십억`;
    case 'usd_billion':
      return value >= 1e3
        ? `$${(value / 1e3).toFixed(2)}조`
        : `$${value.toFixed(digits)}십억`;
    case 'krw_amount':
      return Math.abs(value) >= 1e12
        ? `₩${(value / 1e12).toLocaleString('ko-KR', { maximumFractionDigits: 2 })}조`
        : `₩${(value / 1e8).toLocaleString('ko-KR', { maximumFractionDigits: 2 })}억`;
    case 'contracts':
      return compact(value, 2);
    case 'count':
      return compact(value, 2);
    case 'btc':
      return `${compact(value, 2)} BTC`;
    case 'years':
      return `${value.toFixed(digits)}년`;
    case 'ratio':
      return value.toFixed(digits);
    case 'points':
      return value.toFixed(digits);
    default:
      return value.toLocaleString('en-US', { maximumFractionDigits: digits });
  }
}

function direction(indicator) {
  const change = finite(indicator?.change);
  return change === null || Math.abs(change) < 1e-12 ? 'flat' : change > 0 ? 'up' : 'down';
}

function formatChange(indicator) {
  if (!indicator || finite(indicator.value) === null) return 'NO DATA';
  const pct = finite(indicator.change_pct);
  const change = finite(indicator.change);
  if (pct !== null && !['percent', 'rate', 'ratio', 'points', 'fraction_percent'].includes(indicator.unit)) {
    return `${pct >= 0 ? '+' : ''}${pct.toFixed(2)}%`;
  }
  if (change === null) return '—';
  const adjusted = ['rate', 'fraction_percent'].includes(indicator.unit) ? change * 100 : change;
  const suffix = indicator.unit === 'percent' || indicator.unit === 'rate' || indicator.unit === 'fraction_percent'
    ? 'pp'
    : indicator.unit === 'points' ? 'p' : '';
  return `${adjusted >= 0 ? '+' : ''}${adjusted.toFixed(Number(indicator.decimals ?? 2))}${suffix}`;
}

function conciseCollectionError(error) {
  const message = String(error || '알 수 없는 오류');
  const source = message.split(':')[0] || '데이터';
  const status = message.match(/status code (\d+)|HTTP\s*(\d+)/i);
  const code = status?.[1] || status?.[2];
  return `${source} 갱신 지연${code ? ` (HTTP ${code})` : ''} · 직전 정상값 유지`;
}

function rawValue(indicator) {
  const value = finite(indicator?.raw_value ?? indicator?.value);
  if (value === null) return 'RAW —';
  return `RAW ${value.toLocaleString('en-US', { maximumFractionDigits: 12, useGrouping: false })} [${indicator.unit || 'number'}]`;
}

function indicatorReading(indicator) {
  const value = finite(indicator?.value);
  if (value === null) return { key: 'unknown', label: 'DATA WAIT', hint: '결측값은 정상으로 간주하지 않습니다.' };
  const rule = INDICATOR_RULES[indicator.key];
  if (rule) {
    const [warning, danger, hint, highBad] = rule;
    const dangerHit = highBad ? value >= danger : value <= danger;
    const warningHit = highBad ? value >= warning : value <= warning;
    return {
      key: dangerHit ? 'red' : warningHit ? 'amber' : 'green',
      label: dangerHit ? '위험구간' : warningHit ? '주의구간' : '정상범위',
      hint
    };
  }
  const pos = finite(indicator.range_position);
  const position = pos === null ? '' : `최근 ${indicator.observations || 0}개 범위 ${pos.toFixed(0)}% 위치.`;
  return { key: 'neutral', label: '원본 관측', hint: position || '방향은 다른 지표와 함께 해석합니다.' };
}

function riskState(value) {
  const risk = finite(value);
  if (risk === null) {
    return {
      key: 'amber',
      label: 'DATA WAIT',
      color: 'var(--yellow)',
      message: '충분한 데이터가 아직 없습니다.'
    };
  }
  if (risk >= 65) {
    return {
      key: 'red',
      label: '위험',
      color: 'var(--red)',
      message: '복수 위험 신호가 강하게 켜져 있습니다.'
    };
  }
  if (risk >= 45) {
    return {
      key: 'amber',
      label: '주의',
      color: 'var(--yellow)',
      message: '주의 구간입니다. 변동과 위험 요인을 함께 확인하세요.'
    };
  }
  return {
    key: 'green',
    label: '안정',
    color: 'var(--green)',
    message: '현재 종합 신호는 비교적 안정적입니다.'
  };
}

function indicatorMap(payload) {
  return Object.fromEntries(
    (payload?.dashboard?.indicators || []).map((item) => [item.key, item])
  );
}

function sparkline(values, className = 'sparkline') {
  const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  svg.setAttribute('viewBox', '0 0 100 30');
  svg.setAttribute('preserveAspectRatio', 'none');
  svg.classList.add(className);
  const numbers = Array.isArray(values)
    ? values.map(finite).filter((value) => value !== null)
    : [];
  if (numbers.length < 2) return svg;
  const min = Math.min(...numbers);
  const max = Math.max(...numbers);
  const span = Math.max(max - min, Math.abs(max) * 0.001, 1e-9);
  const points = numbers.map((value, index) => {
    const x = 100 * index / (numbers.length - 1);
    const y = 27 - 24 * (value - min) / span;
    return `${x.toFixed(1)},${y.toFixed(1)}`;
  }).join(' ');
  const line = document.createElementNS('http://www.w3.org/2000/svg', 'polyline');
  line.setAttribute('points', points);
  svg.append(line);
  return svg;
}

function gaugeCard(label, value, display, description, safeHigh = false) {
  const state = riskState(safeHigh ? 100 - clamp(value) : value);
  const card = el('article', 'gauge-card');
  const dial = el('div', 'dial');
  dial.style.setProperty('--value', clamp(value));
  dial.style.setProperty('--gauge', state.color);
  const needle = el('span', 'needle');
  needle.style.setProperty('--value', clamp(value));
  const readout = el('div', 'dial-readout');
  readout.append(el('strong', '', display), el('small', '', '0  ·  50  ·  100'));
  dial.append(needle, readout);
  const copy = el('div', 'gauge-copy');
  copy.append(el('h3', '', label), el('p', '', description));
  const status = el('span', `signal-label ${state.key}`, state.label);
  status.style.color = state.color;
  copy.append(status);
  card.append(dial, copy);
  return card;
}

function renderTicker(indicators) {
  const tape = $('tickerTape');
  clear(tape);
  for (const key of TICKER_KEYS) {
    const indicator = indicators[key] || {
      symbol: key.toUpperCase(),
      label: key,
      value: null
    };
    const item = el('div', 'ticker-item');
    item.append(
      el('span', 'ticker-symbol', indicator.symbol),
      el('strong', 'ticker-value', formatValue(indicator))
    );
    item.append(
      el('span', 'ticker-name', indicator.label),
      el('span', `ticker-change ${direction(indicator)}`, formatChange(indicator))
    );
    tape.append(item);
  }
}

function makeTrafficLight(state) {
  const light = el('div', `traffic-light ${state}`);
  light.append(el('i'), el('i'), el('i'));
  light.setAttribute(
    'aria-label',
    state === 'red' ? '위험' : state === 'amber' ? '주의' : '안정'
  );
  return light;
}

function renderOverview(payload, indicators) {
  const snapshot = payload.snapshot || {};
  $('overviewAsOf').textContent = `AS OF ${formatTime(snapshot.as_of)}`;
  document.querySelectorAll('.market-asof').forEach((node) => {
    node.textContent = `AS OF ${formatTime(snapshot.as_of)}`;
  });

  const gauges = $('overviewGauges');
  clear(gauges);
  const vix = finite(indicators.vix?.value);
  const fearScore = vix === null
    ? finite(snapshot.nodes?.VOLATILITY)
    : clamp((vix - 10) * 3.33);
  const diffusionScore = clamp(Number(snapshot.diffusion || 0) * 12.5);
  gauges.append(
    gaugeCard(
      'GLOBAL RISK',
      snapshot.global_risk,
      score(snapshot.global_risk),
      '모든 시장과 거시 위험을 합산한 ECONOMICS Radar 종합점수.'
    ),
    gaugeCard(
      '공포지수 VIX',
      fearScore,
      vix === null ? score(fearScore) : vix.toFixed(2),
      '미국 옵션시장의 기대 변동성과 내부 변동성 위험을 함께 봅니다.'
    ),
    gaugeCard(
      '위험 전염도',
      diffusionScore,
      `${snapshot.diffusion ?? 0}개`,
      '고위험 신호가 여러 시장·모듈로 동시에 번지는 정도입니다.'
    ),
    gaugeCard(
      '데이터 신뢰도',
      snapshot.confidence,
      `${score(snapshot.confidence)}%`,
      '실제 공식 데이터로 계산 가능한 설계 가중치 비율입니다.',
      true
    )
  );

  const lights = $('marketLights');
  clear(lights);
  for (const [key, label] of [
    ['US_EQUITY', '미국'],
    ['KOREA_EQUITY', '한국'],
    ['CRYPTO', '코인']
  ]) {
    const risk = snapshot.markets?.[key];
    const state = riskState(risk);
    const card = el('article', 'market-light-card');
    card.append(makeTrafficLight(state.key));
    const copy = el('div');
    copy.append(
      el('h3', '', `${label} MARKET`),
      el('p', '', state.message)
    );
    card.append(copy, el('strong', `market-risk-number ${state.key}`, score(risk)));
    lights.append(card);
  }

  const quotes = $('overviewQuotes');
  clear(quotes);
  for (const key of TICKER_KEYS) {
    const indicator = indicators[key] || {
      key,
      symbol: key,
      label: key,
      value: null
    };
    const card = el('article', 'quote-card');
    const header = document.createElement('header');
    header.append(
      el('span', 'symbol', indicator.symbol),
      el('span', 'source', indicator.source || 'NO SOURCE')
    );
    card.append(
      header,
      el('strong', 'quote-value', formatValue(indicator)),
      sparkline(indicator.history)
    );
    const meta = el('div', 'quote-meta');
    meta.append(
      el('span', direction(indicator), `${indicator.change_period || ''} ${formatChange(indicator)}`),
      el('span', '', indicator.observed_at ? String(indicator.observed_at).slice(0, 16) : 'NO DATA')
    );
    card.append(meta);
    quotes.append(card);
  }

  renderRiskHeatmap(snapshot.nodes || {});
  renderProprietary(snapshot);
  renderSources(snapshot.sources || {});
  renderOverviewRecovery(payload);
  renderOverviewMatrix(indicators);
}

function renderOverviewRecovery(payload) {
  const container = $('overviewRecovery');
  clear(container);
  for (const name of ['overall', 'us', 'korea', 'crypto']) {
    container.append(recoveryCard(recoveryModel(name, payload), true));
  }
  const note = el('p', 'recovery-disclaimer', '단일 시점 통과는 “탈출 확정”이 아닙니다. 종합·미국·한국은 7일, 코인은 24시간 전 저장 스냅샷과 비교하며 결측 조건은 통과로 세지 않습니다.');
  container.append(note);
}

function renderOverviewMatrix(indicators) {
  const container = $('overviewMarketMatrix');
  clear(container);
  const groups = [
    ['미국', ['sp500', 'vix', 'us10y', 'curve_10y2y', 'hy_spread', 'ofr_fsi']],
    ['한국', ['kospi', 'kosdaq', 'usdkrw', 'krx_breadth', 'krx_basis', 'krx_option_iv']],
    ['코인', ['btc', 'btc_spot_change', 'btc_oi', 'btc_current_funding', 'btc_basis', 'btc_taker']]
  ];
  for (const [title, keys] of groups) {
    const group = el('section', 'matrix-group');
    group.append(el('h3', '', title));
    for (const key of keys) {
      const indicator = indicators[key] || { key, symbol: key.toUpperCase(), label: key, value: null };
      const reading = indicatorReading(indicator);
      const row = el('div', 'matrix-row');
      const label = el('div');
      label.append(el('strong', '', indicator.symbol), el('small', '', indicator.label));
      row.append(
        label,
        el('b', '', formatValue(indicator)),
        el('span', direction(indicator), `${indicator.change_period || ''} ${formatChange(indicator)}`),
        el('em', reading.key, reading.label)
      );
      group.append(row);
    }
    container.append(group);
  }
}

function renderRiskHeatmap(nodes) {
  const container = $('riskHeatmap');
  clear(container);
  const grouped = { us: [], korea: [], crypto: [], global: [] };

  for (const [name, value] of Object.entries(nodes)) {
    if (finite(value) === null) continue;
    const meta = NODE_META[name] || { label: name, market: 'global' };
    (grouped[meta.market] || grouped.global).push([name, value, meta]);
  }

  for (const market of ['us', 'korea', 'crypto', 'global']) {
    const entries = grouped[market]
      .sort((a, b) => Number(b[1]) - Number(a[1]));
    if (!entries.length) continue;

    const info = HEATMAP_MARKETS[market];
    const group = el('section', `heat-group market-${market}`);
    const header = el('header', 'heat-group-header');
    header.append(
      el('strong', '', info.label),
      el('span', '', `${entries.length}개 신호`)
    );
    const grid = el('div', 'heat-grid');

    for (const [, value, meta] of entries) {
      const state = riskState(value);
      const cell = el('div', 'heat-cell');
      cell.style.setProperty('--risk-color', state.color);
      cell.style.background = `color-mix(in srgb, ${state.color} ${Math.round(12 + clamp(value) * 0.28)}%, #070707)`;
      const top = el('div', 'heat-cell-top');
      top.append(
        el('span', 'heat-market-tag', info.short),
        el('span', 'heat-risk-state', state.label)
      );
      cell.append(
        top,
        el('span', 'heat-node-label', meta.label),
        el('strong', '', score(value))
      );
      grid.append(cell);
    }

    group.append(header, grid);
    container.append(group);
  }
}

function renderProprietary(snapshot) {
  const container = $('proprietarySignals');
  clear(container);
  const items = [
    ['시장 스트레스', snapshot.stress, '현재 충격·가격 압력의 강도'],
    ['구조적 취약성', snapshot.vulnerability, '충격을 증폭하는 부채·레버리지 기반'],
    ['회복 탄력성', snapshot.resilience, '정책·유동성·완충 여력'],
    ['위기 단계', snapshot.stage, '히스테리시스를 적용한 위기 단계'],
    ['데이터 품질', snapshot.data_quality, '추적 공식 소스의 신선도'],
    ['발동 신호 수', snapshot.rules_triggered, '내부 룰 엔진에서 현재 참인 신호 수']
  ];

  for (const [label, value, hint] of items) {
    const card = el('div', 'proprietary-card');
    card.append(el('span', '', label));
    card.append(
      el(
        'strong',
        '',
        value === null || value === undefined
          ? '—'
          : label === '위기 단계'
            ? `STAGE ${value}`
            : score(value)
      )
    );
    card.append(el('small', '', hint));
    container.append(card);
  }
}

function renderSources(sources) {
  const container = $('sourceHealth');
  clear(container);
  for (const [name, state] of Object.entries(sources).sort((a, b) => a[0].localeCompare(b[0]))) {
    const chip = el('div', `source-chip${state?.fresh ? ' fresh' : ''}`);
    const expected = Number(state?.expected_series || 0);
    const available = Number(state?.available_series || 0);
    const coverage = expected > 0 ? ` ${available}/${expected}` : '';
    chip.append(
      el('strong', '', name),
      document.createTextNode(state?.fresh ? `  ● LIVE${coverage}` : `  ○ PARTIAL${coverage}`)
    );
    const missing = Array.isArray(state?.missing_series) ? state.missing_series : [];
    const stale = Array.isArray(state?.stale_series) ? state.stale_series : [];
    chip.title = [
      missing.length ? `미수집: ${missing.join(', ')}` : '',
      stale.length ? `지연: ${stale.join(', ')}` : ''
    ].filter(Boolean).join(' / ');
    container.append(chip);
  }
}

function recoveryValue(snapshot, key) {
  if (!snapshot) return null;
  if (key === 'stress' || key === 'vulnerability') return finite(snapshot[key]);
  if (key === 'resilience_risk') {
    const resilience = finite(snapshot.resilience);
    return resilience === null ? null : 100 - resilience;
  }
  if (Object.prototype.hasOwnProperty.call(snapshot.markets || {}, key)) {
    return finite(snapshot.markets[key]);
  }
  return finite(snapshot.nodes?.[key]);
}

function recoveryRisk(snapshot, config) {
  if (!snapshot) return null;
  if (config.riskKey === 'global_risk') return finite(snapshot.global_risk);
  return finite(snapshot.markets?.[config.riskKey]);
}

function recoveryModel(name, payload) {
  const config = RECOVERY_CONFIG[name];
  const current = payload.snapshot || {};
  const prior = payload.snapshot_history?.[config.prior] || null;
  const risk = recoveryRisk(current, config);
  const priorRisk = recoveryRisk(prior, config);
  const delta = risk === null || priorRisk === null ? null : risk - priorRisk;
  const conditions = config.conditions.map(([key, label]) => {
    const value = recoveryValue(current, key);
    return {
      key, label, value,
      state: value === null ? 'unknown' : value >= 75 ? 'critical' : value >= 55 ? 'blocked' : value <= 35 ? 'met' : 'watch'
    };
  });
  const known = conditions.filter((item) => item.value !== null).length;
  const coverage = known / Math.max(conditions.length, 1);
  const met = conditions.filter((item) => item.state === 'met').length;
  const blocked = conditions.filter((item) => ['blocked', 'critical'].includes(item.state)).length;
  const progress = risk === null || coverage < 0.6
    ? null
    : clamp((75 - risk) / 40 * 100);
  let phase = '판단 데이터 부족';
  if (risk !== null && coverage >= 0.6) {
    phase = risk >= 75 ? '위기' : risk >= 55 ? '스트레스' : risk <= 35 ? '탈출 준비' : '회복 관찰';
  }
  const trend = delta === null
    ? '추세 데이터 대기'
    : delta <= -10 ? '빠르게 개선'
      : delta <= -3 ? '개선'
        : delta < 3 ? '정체'
          : delta < 10 ? '악화' : '빠르게 악화';
  return { name, config, risk, priorRisk, delta, conditions, known, coverage, met, blocked, progress, phase, trend };
}

function recoveryCard(model, compactMode = false) {
  const card = el('article', `recovery-card phase-${model.phase.replaceAll(' ', '-')}`);
  const head = el('div', 'recovery-head');
  const title = el('div');
  title.append(el('strong', '', `${model.config.title} · ${model.phase}`), el('small', '', `${model.trend} / 데이터 ${model.known}/${model.conditions.length}`));
  head.append(title, el('b', '', model.progress === null ? '—' : `${model.progress.toFixed(0)}%`));
  card.append(head);
  const bar = el('div', 'recovery-progress');
  const fill = el('i');
  fill.style.width = `${model.progress ?? 0}%`;
  bar.append(fill);
  card.append(bar, el('p', 'recovery-caption', '위기 탈출 목표 접근도 · 모델 위험점수 35 이하 목표'));
  const list = el('div', compactMode ? 'recovery-condition-grid compact' : 'recovery-condition-grid');
  for (const condition of model.conditions) {
    const row = el('div', `recovery-condition ${condition.state}`);
    row.append(
      el('span', '', condition.label),
      el('strong', '', condition.value === null ? '—' : condition.value.toFixed(1)),
      el('em', '', condition.state === 'met' ? '≤35 통과' : condition.state === 'critical' ? '≥75 핵심차단' : condition.state === 'blocked' ? '≥55 차단' : condition.state === 'watch' ? '관찰' : '결측')
    );
    list.append(row);
  }
  card.append(list);
  return card;
}

function renderMarket(config, payload, indicators) {
  const snapshot = payload.snapshot || {};
  const container = $(config.target);
  clear(container);
  const risk = snapshot.markets?.[config.riskKey];
  const state = riskState(risk);

  const hero = el('section', 'market-hero');
  const gauge = el('div', 'market-gauge');
  gauge.append(
    gaugeCard(`${config.title} RISK`, risk, score(risk), state.message)
  );

  const summary = el('div', 'market-summary');
  summary.append(el('h3', '', `${config.title} 상황판 · ${state.label}`));
  summary.append(el('p', '', buildMarketSummary(config, snapshot, indicators, state)));
  const topFactors = config.factors
    .map((name) => [name, finite(snapshot.nodes?.[name])])
    .filter(([, value]) => value !== null)
    .sort((a, b) => b[1] - a[1])
    .slice(0, 4);
  const list = el('ul', 'market-summary-list');
  for (const [name, value] of topFactors) {
    list.append(el('li', '', `${factorLabel(config, name)} ${score(value)}점`));
  }
  summary.append(list);

  const lightArea = el('div', 'large-traffic');
  lightArea.append(makeTrafficLight(state.key), el('strong', '', state.label));
  hero.append(gauge, summary, lightArea);
  container.append(hero);

  const recoveryPanel = el('section', 'terminal-panel market-recovery-panel');
  const recoveryHeading = el('div', 'panel-heading');
  recoveryHeading.append(el('span', '', 'CRISIS EXIT GATES'), el('strong', '', `${config.title} 위기 탈출 조건`));
  const recoveryName = config.riskKey === 'US_EQUITY' ? 'us' : config.riskKey === 'KOREA_EQUITY' ? 'korea' : 'crypto';
  recoveryPanel.append(recoveryHeading, recoveryCard(recoveryModel(recoveryName, payload)));
  container.append(recoveryPanel);

  if (recoveryName === 'crypto') {
    container.append(renderCryptoRegime(indicators));
  }

  const assetGrid = el('div', 'asset-grid');
  for (const [title, code, keys] of config.sections) {
    const panel = el('section', 'terminal-panel');
    const heading = el('div', 'panel-heading');
    heading.append(el('span', '', code), el('strong', '', title));
    panel.append(heading, indicatorTable(keys, indicators));
    assetGrid.append(panel);
  }
  container.append(assetGrid);

  const factorPanel = el('section', 'terminal-panel');
  const heading = el('div', 'panel-heading');
  heading.append(
    el('span', '', 'ECONOMICS RADAR SIGNAL MATRIX'),
    el('strong', '', '내부 위험요인')
  );
  const board = el('div', 'factor-board');
  for (const name of config.factors) {
    const value = finite(snapshot.nodes?.[name]);
    const stateForFactor = riskState(value);
    const card = el('div', 'factor-card');
    const top = el('div', 'factor-top');
    top.append(
      el('span', '', factorLabel(config, name)),
      el('strong', '', score(value))
    );
    const bar = el('div', 'factor-bar');
    const fill = el('i');
    fill.style.width = `${clamp(value)}%`;
    fill.style.background = stateForFactor.color;
    bar.append(fill);
    card.append(top, bar);
    board.append(card);
  }
  factorPanel.append(heading, board);
  container.append(factorPanel);
}

function renderCryptoRegime(indicators) {
  const panel = el('section', 'terminal-panel crypto-regime-panel');
  const heading = el('div', 'panel-heading');
  heading.append(el('span', '', 'PRICE × OPEN INTEREST'), el('strong', '', 'BTC 레버리지 국면 해석'));
  const price = indicators.btc;
  const oi = indicators.btc_oi;
  const priceDelta = finite(price?.change_pct);
  const oiDelta = finite(oi?.change_pct);
  let title = '국면 데이터 대기';
  let body = '가격과 미결제약정의 24시간 비교값이 모두 있어야 결합 국면을 판정합니다.';
  let state = 'unknown';
  if (priceDelta !== null && oiDelta !== null) {
    if (priceDelta >= 0 && oiDelta >= 0) {
      title = '가격↑ + OI↑ · 레버리지 동반 상승';
      body = '추세는 강하지만 포지션이 쌓여 청산 취약성도 커집니다. 펀딩·베이시스 과열 여부를 함께 보세요.';
      state = 'amber';
    } else if (priceDelta < 0 && oiDelta >= 0) {
      title = '가격↓ + OI↑ · 위험 조합';
      body = '신규 숏 또는 손실 중인 롱이 늘 수 있는 국면입니다. 추가 하락과 연쇄청산을 가장 경계합니다.';
      state = 'red';
    } else if (priceDelta < 0 && oiDelta < 0) {
      title = '가격↓ + OI↓ · 디레버리징';
      body = '포지션 청산이 진행 중입니다. OI 급감이 멈추고 현물가격이 안정되는지 확인해야 합니다.';
      state = 'amber';
    } else {
      title = '가격↑ + OI↓ · 숏커버 가능성';
      body = '레버리지 축소 속 반등일 수 있습니다. 현물 거래대금과 테이커 매수 우위의 지속을 확인하세요.';
      state = 'green';
    }
  }
  const content = el('div', `crypto-regime ${state}`);
  content.append(
    el('strong', '', title),
    el('p', '', body),
    el('small', '', `BTC ${formatChange(price || {})} / OI ${formatChange(oi || {})}`)
  );
  panel.append(heading, content);
  return panel;
}

function buildMarketSummary(config, snapshot, indicators, state) {
  const available = config.sections
    .flatMap((section) => section[2])
    .map((key) => indicators[key])
    .filter((item) => finite(item?.value) !== null);
  const movers = available
    .filter((item) => finite(item.change_pct) !== null)
    .sort((a, b) => Math.abs(b.change_pct) - Math.abs(a.change_pct))
    .slice(0, 2);
  const moverText = movers.length
    ? `가장 큰 변화는 ${movers.map((item) => `${item.label} ${formatChange(item)}`).join(', ')}입니다.`
    : '주요 시세의 비교 변화 데이터가 아직 충분하지 않습니다.';
  return `${state.message} ${moverText} 전체 모델 신뢰도는 ${score(snapshot.confidence)}%이며 결측치는 위험 신호로 임의 변환하지 않습니다.`;
}

function indicatorTable(keys, indicators) {
  const table = el('table', 'indicator-table');
  const head = document.createElement('thead');
  const headRow = document.createElement('tr');
  for (const title of ['지표', '현재값 / RAW', '직전 변화', '최근 범위', '쉬운 해석', '추세', '기준·출처']) {
    headRow.append(el('th', '', title));
  }
  head.append(headRow);

  const body = document.createElement('tbody');
  for (const key of keys) {
    const indicator = indicators[key] || {
      key,
      symbol: key.toUpperCase(),
      label: key,
      value: null,
      history: []
    };
    const row = document.createElement('tr');
    const nameCell = document.createElement('td');
    nameCell.append(
      el('span', 'indicator-name', indicator.symbol),
      el('span', 'indicator-label', indicator.label),
      el('span', 'indicator-asset', String(indicator.asset_class || '').toUpperCase())
    );
    const valueCell = el('td', finite(indicator.value) === null ? 'no-data indicator-value-cell' : 'indicator-value-cell');
    valueCell.append(el('strong', '', formatValue(indicator)), el('small', 'raw-value', rawValue(indicator)));
    row.append(nameCell, valueCell);
    row.append(
      el('td', direction(indicator), `${indicator.change_period || ''} ${formatChange(indicator)}`)
    );
    const rangeCell = el('td', 'indicator-range');
    const low = finite(indicator.history_low);
    const high = finite(indicator.history_high);
    rangeCell.append(
      el('span', '', low === null || high === null ? '—' : `${formatValue({ ...indicator, value: low })} – ${formatValue({ ...indicator, value: high })}`),
      el('small', '', finite(indicator.range_position) === null ? '' : `현재 ${Number(indicator.range_position).toFixed(0)}% 위치 · n=${indicator.observations}`)
    );
    row.append(rangeCell);
    const reading = indicatorReading(indicator);
    const readingCell = el('td', 'indicator-reading');
    readingCell.append(el('b', reading.key, reading.label), el('small', '', reading.hint));
    row.append(readingCell);
    const sparkCell = document.createElement('td');
    sparkCell.append(sparkline(indicator.history, 'mini-spark'));
    row.append(sparkCell);
    const sourceCell = el('td', 'indicator-source');
    sourceCell.append(
      el('span', '', indicator.observed_at ? String(indicator.observed_at).slice(0, 16) : '—'),
      el('small', '', indicator.source_series || 'NO SOURCE'),
      el('small', `freshness ${String(indicator.freshness || '').startsWith('STALE') ? 'stale' : ''}`, `${indicator.cadence || 'UNKNOWN'} · ${indicator.freshness || 'UNKNOWN'}`)
    );
    row.append(sourceCell);
    body.append(row);
  }

  table.append(head, body);
  return table;
}

function renderSafely(label, renderFn, errors) {
  try {
    renderFn();
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    errors.push(`${label}: ${message}`);
  }
}

function render(payload) {
  const indicators = indicatorMap(payload);
  const errors = [];
  renderSafely('상단 시세', () => renderTicker(indicators), errors);
  renderSafely('종합 탭', () => renderOverview(payload, indicators), errors);
  for (const config of Object.values(MARKET_CONFIG)) {
    renderSafely(
      `${config.title} 탭`,
      () => renderMarket(config, payload, indicators),
      errors
    );
  }
  renderSafely(
    '결과 시각',
    () => {
      $('lastUpdated').textContent = `RESULT ${formatTime(payload.snapshot?.as_of)}`;
    },
    errors
  );
  return errors;
}

async function loadDashboard() {
  let payload;
  try {
    const response = await fetch('/api/dashboard', { cache: 'no-store' });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    payload = await response.json();
  } catch (error) {
    $('connectionDot').className = 'status-dot offline';
    $('connectionText').textContent = 'CONNECTION ERROR';
    dashboardErrors = [`대시보드 API 실패: ${error.message}`];
    updateErrorBanner();
    return;
  }

  const renderErrors = render(payload);
  dashboardErrors = renderErrors.map((error) => `화면 렌더링 실패: ${error}`);
  if (renderErrors.length) {
    $('connectionDot').className = 'status-dot offline';
    $('connectionText').textContent = 'RENDER PARTIAL';
  }
  updateErrorBanner();
}

async function loadRefreshStatus() {
  const button = $('refreshButton');
  try {
    const response = await fetch('/api/refresh-status', { cache: 'no-store' });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const status = await response.json();
    const running = Boolean(status.running || status.queued);

    button.disabled = running;
    button.textContent = running ? '최신 데이터 수집 중…' : '최신 데이터 수집';
    collectionErrors = Array.isArray(status.errors)
      ? status.errors.slice(0, 3).map(conciseCollectionError)
      : [];

    if (!running && workerWasRunning) {
      await loadDashboard();
    }

    if (running) {
      $('connectionDot').className = 'status-dot working';
      $('connectionText').textContent = status.queued
        ? 'REFRESH QUEUED'
        : `COLLECTING ${status.phase || 'DATA'} · ${status.stored || 0}/${status.attempted || 0}`;
    } else {
      $('connectionDot').className = dashboardErrors.length
        ? 'status-dot offline'
        : 'status-dot online';
      $('connectionText').textContent = dashboardErrors.length
        ? 'RENDER PARTIAL'
        : collectionErrors.length
          ? `COMPLETE / ${collectionErrors.length} ERR`
          : 'AUTO REFRESH ONLINE';
    }

    workerWasRunning = running;
    updateErrorBanner();
  } catch (error) {
    button.disabled = false;
    $('connectionDot').className = 'status-dot offline';
    $('connectionText').textContent = 'STATUS ERROR';
    collectionErrors = [`수집 상태 조회 실패: ${error.message}`];
    updateErrorBanner();
  }
}

async function requestRefresh() {
  const button = $('refreshButton');
  button.disabled = true;
  $('connectionText').textContent = 'REQUESTING REFRESH';
  try {
    const response = await fetch('/api/refresh', {
      method: 'POST',
      cache: 'no-store'
    });
    if (!response.ok && response.status !== 409) {
      throw new Error(`HTTP ${response.status}`);
    }
    collectionErrors = [];
    updateErrorBanner();
    await loadRefreshStatus();
  } catch (error) {
    collectionErrors = [`최신 데이터 수집을 시작하지 못했습니다: ${error.message}`];
    updateErrorBanner();
    button.disabled = false;
  }
}

function selectTab(name) {
  if (!TAB_NAMES.includes(name)) return;
  document.querySelectorAll('.tab-button').forEach((button) => {
    const selected = button.dataset.tab === name;
    button.classList.toggle('active', selected);
    button.setAttribute('aria-selected', String(selected));
  });
  document.querySelectorAll('.tab-panel').forEach((panel) => {
    panel.hidden = panel.id !== `tab-${name}`;
  });
}

document.querySelectorAll('.tab-button').forEach((button) => {
  button.addEventListener('click', () => selectTab(button.dataset.tab));
});

document.addEventListener('keydown', (event) => {
  const shortcut = {
    F1: 'overview',
    F2: 'us',
    F3: 'korea',
    F4: 'crypto'
  }[event.key];
  if (shortcut) {
    event.preventDefault();
    selectTab(shortcut);
  }
});

$('refreshButton').addEventListener('click', requestRefresh);
loadDashboard();
loadRefreshStatus();
setInterval(loadDashboard, 60000);
setInterval(loadRefreshStatus, 3000);
