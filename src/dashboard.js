function $(id) {
  const node = document.getElementById(id);
  if (!node) throw new Error(`DOM element #${id} not found`);
  return node;
}

const NODE_META = {
  VALUATION: { label: '주식 밸류에이션', market: 'us' },
  BUSINESS_DEBT: { label: '기업부채', market: 'us' },
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
    VALUATION: '미국 주식 밸류에이션',
    BUSINESS_DEBT: '미국 기업부채',
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
      ['주식·공포지수', 'EQUITY / VOL', ['sp500', 'nasdaq', 'dow', 'vix']],
      ['채권·금리', 'RATES / CREDIT', ['us10y', 'us2y', 'curve_10y2y', 'hy_spread', 'treasury_bid_cover']],
      ['달러·금융환경', 'FX / CONDITIONS', ['usd_index', 'usdkrw']]
    ]
  },
  korea: {
    title: '한국 시장',
    riskKey: 'KOREA_EQUITY',
    target: 'koreaMarket',
    factors: ['KOREA_FIN_STAB', 'KOREA_MARKET_INTERNALS', 'KOREA_MACRO', 'USD', 'LIQUIDITY', 'CREDIT', 'BANKING', 'RATES'],
    sections: [
      ['주식·환율', 'EQUITY / FX', ['kospi', 'kosdaq', 'usdkrw', 'kr_base_rate']],
      ['시장폭·수급', 'BREADTH / INTERNALS', ['kospi_breadth', 'kosdaq_breadth', 'krx_breadth']],
      ['선물·옵션', 'FUTURES / OPTIONS', ['krx_basis', 'krx_futures_oi', 'krx_put_call', 'krx_option_iv']],
      ['채권', 'FIXED INCOME', ['krx_bond_yield', 'krx_kts_yield']]
    ]
  },
  crypto: {
    title: '코인 시장',
    riskKey: 'CRYPTO',
    target: 'cryptoMarket',
    factors: ['CRYPTO_DERIVATIVES', 'LIQUIDITY', 'USD', 'LEVERAGE', 'VOLATILITY', 'FUNDING'],
    sections: [
      ['비트코인 현물', 'SPOT', ['btc']],
      ['선물·펀딩', 'FUTURES / FUNDING', ['btc_funding', 'btc_oi', 'btc_basis']],
      ['포지셔닝·주문흐름', 'POSITIONING / FLOW', ['btc_global_ls', 'btc_top_position', 'btc_top_account', 'btc_taker']]
    ]
  }
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
    case 'contracts':
      return compact(value, 2);
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
  if (pct !== null && !['percent', 'rate', 'ratio'].includes(indicator.unit)) {
    return `${pct >= 0 ? '+' : ''}${pct.toFixed(2)}%`;
  }
  if (change === null) return '—';
  const adjusted = indicator.unit === 'rate' ? change * 100 : change;
  return `${adjusted >= 0 ? '+' : ''}${adjusted.toFixed(Number(indicator.decimals ?? 2))}`;
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
    chip.append(
      el('strong', '', name),
      document.createTextNode(state?.fresh ? '  ● LIVE' : '  ○ WAIT')
    );
    container.append(chip);
  }
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
  for (const title of ['지표', '현재', '변화', '추세', '기준']) {
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
      el('span', 'indicator-label', indicator.label)
    );
    row.append(
      nameCell,
      el('td', finite(indicator.value) === null ? 'no-data' : '', formatValue(indicator))
    );
    row.append(
      el('td', direction(indicator), `${indicator.change_period || ''} ${formatChange(indicator)}`)
    );
    const sparkCell = document.createElement('td');
    sparkCell.append(sparkline(indicator.history, 'mini-spark'));
    row.append(
      sparkCell,
      el(
        'td',
        'indicator-label',
        indicator.observed_at ? String(indicator.observed_at).slice(0, 10) : '—'
      )
    );
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
      ? status.errors.slice(0, 3).map((error) => `일부 데이터 수집 실패: ${error}`)
      : [];

    if (!running && workerWasRunning) {
      await loadDashboard();
    }

    if (running) {
      $('connectionDot').className = 'status-dot working';
      $('connectionText').textContent = status.queued
        ? 'REFRESH QUEUED'
        : 'COLLECTING LIVE DATA';
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
