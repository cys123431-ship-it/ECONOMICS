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
  KOREA_FIN_STAB: { label: '한국 금융시스템 위험', market: 'korea' },
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
    KOREA_FIN_STAB: '한국 금융시스템 위험',
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
      ['미국 국채금리·수익률곡선', 'US RATES / CURVE', ['us3m', 'us2y', 'us5y', 'us10y', 'us30y', 'curve_10y2y', 'curve_10y3m']],
      ['미국 실질금리·시장 기대인플레이션', 'US REAL RATES / INFLATION', ['us_real5y', 'us_real10y', 'us_be5y', 'us_be10y', 'us_5y5y']],
      ['미국 신용스프레드·금융여건', 'US CREDIT / CONDITIONS', ['hy_spread', 'ig_spread', 'ofr_fsi', 'stlfsi', 'nfci', 'anfci', 'nfci_leverage']],
      ['미국 성장·고용 선행지표', 'US GROWTH / LABOR', ['wei', 'cfnai', 'sahm', 'initial_claims', 'continued_claims']],
      ['미국 주택·은행·대출건전성', 'US HOUSING / BANKING', ['mortgage30', 'card_delinquency', 'loan_delinquency', 'bank_capital']],
      ['연준 유동성·은행대출', 'FED LIQUIDITY / CREDIT', ['fed_assets', 'rrp', 'total_reserves', 'reserve_balances', 'iorb', 'business_loans', 'total_loans']],
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
      ['한국 물가·통화·국채금리', 'KR MACRO / RATES', ['kr_cpi', 'kr_m2', 'kr_ktb3y', 'kr_ktb10y']],
      ['한국 시장 내부체력·외부수요', 'KR BREADTH / LEADING', ['kospi_breadth', 'kosdaq_breadth', 'krx_breadth', 'kr_cli', 'cn_cli']],
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
      ['BTC 공격적 주문흐름', 'BTC TAKER FLOW', ['btc_taker']],
      ['BTC 옵션 기대변동성·풋콜', 'BTC OPTIONS / DVOL', ['btc_dvol', 'btc_option_oi', 'btc_option_put_call', 'btc_option_volume']],
      ['BTC 온체인 활동·가치평가', 'BTC ON-CHAIN', ['btc_active_addresses', 'btc_tx_count', 'btc_mvrv', 'btc_fees', 'btc_hash_rate', 'btc_market_cap']],
      ['코인시장 달러 유동성', 'STABLECOIN LIQUIDITY', ['stablecoin_cap']]
    ]
  }
};

const FUNDING_CONFIG = {
  title: '자금·포지션',
  target: 'fundingMarket',
  sections: [
    ['미국 단기자금 조달금리·꼬리위험', 'OFR MONEY MARKET', ['sofr', 'effr', 'sofr_tail', 'sofr_effr', 'sofr_volume']],
    ['미국 레포 시장 규모', 'OFR DVP REPO', ['repo_outstanding', 'repo_volume']],
    ['CFTC 미국 주가지수 기관 포지션', 'CFTC EQUITY TFF', ['cftc_spx_oi', 'cftc_spx_asset', 'cftc_spx_lev', 'cftc_ndx_asset', 'cftc_ndx_lev']],
    ['CFTC 미 국채·달러 기관 포지션', 'CFTC RATES / USD TFF', ['cftc_ust_asset', 'cftc_ust_lev', 'cftc_dxy_lev']],
    ['CFTC·Deribit 비트코인 파생', 'BTC INSTITUTIONAL / OPTIONS', ['cftc_btc_oi', 'cftc_btc_lev', 'btc_dvol', 'btc_option_oi', 'btc_option_put_call', 'btc_option_volume']],
    ['비트코인 네트워크·스테이블코인 유동성', 'ON-CHAIN / LIQUIDITY', ['btc_active_addresses', 'btc_tx_count', 'btc_mvrv', 'btc_fees', 'btc_hash_rate', 'btc_market_cap', 'stablecoin_cap']]
  ]
};

const PRICE_DIRECTION_CONFIG = {
  us: {
    title: '미국', period: '최근 1거래일', sidewaysBand: 0.10,
    components: [
      ['sp500', 'S&P 500', 'change_pct'],
      ['nasdaq', '나스닥 종합', 'change_pct'],
      ['dow', '다우존스', 'change_pct']
    ]
  },
  korea: {
    title: '한국', period: '최근 공식 거래일', sidewaysBand: 0.10,
    components: [
      ['kospi', 'KOSPI', 'change_pct'],
      ['kosdaq', 'KOSDAQ', 'change_pct']
    ]
  },
  crypto: {
    title: '코인', period: '최근 24시간', sidewaysBand: 0.25,
    components: [['btc_spot_change', 'BTC 현물', 'value']]
  }
};

const INDICATOR_RULES = {
  vix: [25, 35, '20 미만이면 기대 변동성이 낮은 구간', true],
  hy_spread: [4.5, 5, '4.5% 미만·5일 축소이면 신용위험 하락', true],
  ig_spread: [1.2, 1.8, '스프레드 하락은 투자등급 신용위험 감소', true],
  ofr_fsi: [0, 1, '0 미만이면 장기평균보다 시장 스트레스가 낮음', true],
  stlfsi: [0, 1, '0 미만이면 장기평균보다 금융 스트레스가 낮음', true],
  nfci: [0, 0.5, '0 미만이면 금융여건 완화', true],
  anfci: [0, 0.5, '0 미만이면 경제여건 대비 완화', true],
  sahm: [0.5, 0.75, '0.50%p 미만이면 침체 경보가 꺼진 구간', true],
  kospi_breadth: [40, 25, '50% 이상이면 상승 종목이 하락 종목보다 많음', false],
  kosdaq_breadth: [40, 25, '50% 이상이면 상승 종목이 하락 종목보다 많음', false],
  krx_breadth: [40, 25, '50% 이상이면 상승 확산', false],
  etf_breadth: [40, 25, '50% 이상이면 상승 ETF가 하락 ETF보다 많음', false],
  etn_breadth: [40, 25, '50% 이상이면 상승 ETN이 하락 ETN보다 많음', false],
  krx_basis: [0, -5, '0p 이상이면 선물이 현물보다 높은 구간', false],
  krx_put_call: [1.2, 1.6, '비율 상승은 풋 거래 또는 포지션 비중 증가', true],
  krx_option_iv: [45, 65, '최근 범위 60백분위 아래이면 옵션 변동성 위험 하락', true],
  curve_10y2y: [0.25, 0, '+0.25%p 이상이면 10년 금리가 2년 금리보다 높음', false],
  curve_10y3m: [0.25, 0, '+0.25%p 이상이면 10년 금리가 3개월 금리보다 높음', false],
  sofr_tail: [0.10, 0.25, 'SOFR 99백분위와 중앙값 차이 확대는 조달비용 분산과 꼬리 마찰 증가', true],
  sofr_effr: [0.10, 0.25, 'SOFR가 EFFR보다 크게 높아지면 담보부 조달시장의 상대적 압박 가능성', true],
  btc_dvol: [60, 80, 'DVOL 상승은 BTC 옵션시장이 반영하는 기대변동성 확대', true],
  btc_option_put_call: [1.0, 1.3, '풋 미결제약정/콜 미결제약정이 1보다 높으면 풋 비중 우세', true],
  btc_mvrv: [3.0, 4.0, 'MVRV가 높을수록 시가총액이 실현총액보다 크게 높은 구간', true]
};

const TICKER_KEYS = ['usdkrw', 'btc', 'sp500', 'nasdaq', 'dow', 'kospi', 'kosdaq'];
const TAB_NAMES = ['overview', 'us', 'korea', 'crypto', 'centralbank', 'funding'];
let workerWasRunning = false;
let dashboardErrors = [];
let collectionErrors = [];
let officialKrIndices = new Map();
let liveKrIndices = new Map();
let lastPayload = null;
let lastIndicators = {};
let terminalScope = localStorage.getItem('economics-radar-scope') || 'all';
let terminalDensity = localStorage.getItem('economics-radar-density') || 'compact';
let favoriteKeys = loadFavorites();

function loadFavorites() {
  const defaults = ['sp500', 'vix', 'kospi', 'kosdaq', 'usdkrw', 'btc', 'sofr', 'btc_dvol'];
  try {
    const raw = localStorage.getItem('economics-radar-favorites');
    if (raw === null) return new Set(defaults);
    const parsed = JSON.parse(raw);
    return new Set(Array.isArray(parsed) ? parsed.filter((key) => typeof key === 'string') : defaults);
  } catch (_error) {
    return new Set(defaults);
  }
}

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
  if (value === null || value === undefined || value === '') return null;
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
    case 'percentage_points':
      return `${value.toFixed(digits)}%p`;
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
    case 'krw_billion':
      return `₩${(value / 1e3).toLocaleString('ko-KR', { maximumFractionDigits: digits })}조`;
    case 'contracts':
      return compact(value, 2);
    case 'count':
      return compact(value, 2);
    case 'btc':
      return `${compact(value, 2)} BTC`;
    case 'hashrate_th':
      return `${compact(value, 2)} TH/s`;
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
  if (pct !== null && !['percent', 'percentage_points', 'rate', 'ratio', 'points', 'fraction_percent'].includes(indicator.unit)) {
    return `${pct >= 0 ? '+' : ''}${pct.toFixed(2)}%`;
  }
  if (change === null) return '—';
  const adjusted = ['rate', 'fraction_percent'].includes(indicator.unit) ? change * 100 : change;
  const suffix = indicator.unit === 'percent' || indicator.unit === 'percentage_points' || indicator.unit === 'rate' || indicator.unit === 'fraction_percent'
    ? 'pp'
    : indicator.unit === 'points' ? 'p' : '';
  return `${adjusted >= 0 ? '+' : ''}${adjusted.toFixed(Number(indicator.decimals ?? 2))}${suffix}`;
}

function signed(value, digits = 2, suffix = '') {
  const number = finite(value);
  return number === null ? '—' : `${number >= 0 ? '+' : ''}${number.toFixed(digits)}${suffix}`;
}

function numberValue(value, digits = 0) {
  const number = finite(value);
  return number === null
    ? '—'
    : new Intl.NumberFormat('en-US', { maximumFractionDigits: digits }).format(number);
}

function centralBankState(value) {
  const number = finite(value);
  if (number === null) return { key: 'unknown', label: 'DATA WAIT' };
  if (number >= 15) return { key: 'long', label: number >= 40 ? '강한 롱 우세' : '롱 우세' };
  if (number <= -15) return { key: 'short', label: number <= -40 ? '강한 숏 우세' : '숏 우세' };
  return { key: 'neutral', label: '중립·혼조' };
}

function krChangeClass(value) {
  const number = finite(value);
  if (number === null || number === 0) return 'flat';
  return number > 0 ? 'up' : 'down';
}

function formatKrIndex(value) {
  const number = finite(value);
  return number === null
    ? '—'
    : number.toLocaleString('en-US', {
        minimumFractionDigits: 2,
        maximumFractionDigits: 2
      });
}

function signedKrIndex(value) {
  const number = finite(value);
  if (number === null) return '—';
  return `${number > 0 ? '+' : ''}${number.toLocaleString('en-US', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2
  })}`;
}

const KR_INDEX_CARDS = {
  KOSPI: {
    target: 'krLiveKospi',
    name: '코스피 종합지수',
    detail: 'https://m.stock.naver.com/domestic/index/KOSPI'
  },
  KOSDAQ: {
    target: 'krLiveKosdaq',
    name: '코스닥 종합지수',
    detail: 'https://m.stock.naver.com/domestic/index/KOSDAQ'
  }
};

function renderKrIndexCard(code) {
  const config = KR_INDEX_CARDS[code];
  const live = liveKrIndices.get(code);
  const official = officialKrIndices.get(code.toLowerCase());
  const quote = live || official;
  const target = $(config.target);
  const direction = krChangeClass(quote?.change);
  target.className = `kr-live-index-card ${direction}`;
  clear(target);

  const header = el('div', 'kr-live-header');
  const title = el('div', 'kr-live-title');
  title.append(el('span', '', code), el('strong', '', config.name));
  const status = live
    ? `${live.delay_label || '지연 여부 미표시'} · ${live.market_status_label || '시장상태 확인'}`
    : official
      ? '외부 연결 실패 · KRX 공식 EOD'
      : '시세 연결 실패';
  header.append(title, el('span', 'kr-live-status', status));

  const priceRow = el('div', 'kr-live-price-row');
  priceRow.append(
    el('strong', 'kr-live-price', formatKrIndex(quote?.value)),
    el(
      'span',
      `kr-live-change ${direction}`,
      `${signedKrIndex(quote?.change)} (${signedKrIndex(quote?.change_pct)}%)`
    )
  );

  const observedAt = live
    ? formatTime(live.observed_at)
    : official?.observed_at || '기준시각 없음';
  const source = live ? 'Npay 증권 장중 참고' : official ? 'KRX 공식 최신 종가' : '데이터 없음';
  const meta = el('div', 'kr-live-meta');
  meta.append(el('span', '', observedAt), el('span', '', source));

  const links = el('div', 'kr-live-links');
  const detail = el('a', '', '실시간 상세·차트 ↗');
  detail.href = live?.source_url || config.detail;
  detail.target = '_blank';
  detail.rel = 'noopener noreferrer';
  const krx = el('a', '', 'KRX 공식 데이터 ↗');
  krx.href = 'https://data.krx.co.kr/contents/MDC/MAIN/main/index.cmd';
  krx.target = '_blank';
  krx.rel = 'noopener noreferrer';
  links.append(detail, krx);

  target.append(header, priceRow, meta, links);
}

function renderKrIndexCards() {
  Object.keys(KR_INDEX_CARDS).forEach(renderKrIndexCard);
}

async function loadKrIndexReference() {
  try {
    const response = await fetch('/api/kr-indices-live', { cache: 'no-store' });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const payload = await response.json();
    liveKrIndices = new Map(
      (Array.isArray(payload.quotes) ? payload.quotes : [])
        .filter((quote) => quote && quote.code)
        .map((quote) => [quote.code, quote])
    );
  } catch (_error) {
    liveKrIndices = new Map();
  }
  renderKrIndexCards();
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
      label: '위험도 계산 불가',
      color: 'var(--yellow)',
      message: '모델 위험점수를 계산할 데이터가 부족합니다. 가격 방향을 뜻하지 않습니다.'
    };
  }
  if (risk >= 65) {
    return {
      key: 'red',
      label: '위험 높음',
      color: 'var(--red)',
      message: `모델 위험점수 ${risk.toFixed(1)}/100으로 높습니다. 이 점수는 가격 하락 확률이 아닙니다.`
    };
  }
  if (risk >= 45) {
    return {
      key: 'amber',
      label: '위험 중간',
      color: 'var(--yellow)',
      message: `모델 위험점수 ${risk.toFixed(1)}/100으로 중간입니다. 가격 방향은 별도 방향판을 보세요.`
    };
  }
  return {
    key: 'green',
    label: '위험 낮음',
    color: 'var(--green)',
    message: `모델 위험점수 ${risk.toFixed(1)}/100으로 낮습니다. 이것은 가격 상승 신호가 아닙니다.`
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
  const numericValue = finite(value);
  const state = riskState(numericValue === null ? null : safeHigh ? 100 - clamp(numericValue) : numericValue);
  const card = el('article', 'gauge-card');
  const dial = el('div', 'dial');
  dial.style.setProperty('--value', numericValue === null ? 0 : clamp(numericValue));
  dial.style.setProperty('--gauge', state.color);
  const needle = el('span', 'needle');
  needle.style.setProperty('--value', numericValue === null ? 0 : clamp(numericValue));
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
    state === 'red' ? '위험 높음' : state === 'amber' ? '위험 중간' : '위험 낮음'
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
  const diffusion = finite(snapshot.diffusion);
  const diffusionScore = diffusion === null ? null : clamp(diffusion * 12.5);
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
      diffusion === null ? '—' : `${diffusion}개`,
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
  renderOverviewCentralBank(payload.dashboard?.central_bank);
  renderOverviewDirection(indicators);
  renderOverviewMatrix(indicators);
  renderPriority(indicators);
  renderFavorites(indicators);
  renderForecastOverview(payload);
}

function saveFavorites() {
  localStorage.setItem('economics-radar-favorites', JSON.stringify([...favoriteKeys]));
}

function tabForIndicator(indicator) {
  return ['us', 'korea', 'crypto'].includes(indicator?.market) ? indicator.market : 'funding';
}

function jumpToIndicator(key) {
  const indicator = lastIndicators[key];
  if (!indicator) return;
  selectTab(tabForIndicator(indicator));
  const row = document.querySelector(`[data-indicator-key="${key}"]`);
  const section = row?.closest('details');
  if (section) section.open = true;
  if (row) {
    row.scrollIntoView({ behavior: 'smooth', block: 'center' });
    row.classList.add('indicator-highlight');
    setTimeout(() => row.classList.remove('indicator-highlight'), 2200);
  }
  $('searchResults').hidden = true;
}

function toggleFavorite(key) {
  if (favoriteKeys.has(key)) favoriteKeys.delete(key);
  else favoriteKeys.add(key);
  saveFavorites();
  document.querySelectorAll(`[data-favorite-key="${key}"]`).forEach((button) => {
    const active = favoriteKeys.has(key);
    button.classList.toggle('active', active);
    button.setAttribute('aria-pressed', String(active));
    button.textContent = active ? '★' : '☆';
  });
  renderFavorites(lastIndicators);
}

function monitorRow(indicator, reason = '') {
  const button = el('button', 'monitor-row');
  button.type = 'button';
  button.addEventListener('click', () => jumpToIndicator(indicator.key));
  button.append(
    el('span', 'monitor-symbol', indicator.symbol || indicator.key.toUpperCase()),
    el('span', 'monitor-label', indicator.label || indicator.key),
    el('strong', '', formatValue(indicator)),
    el('span', direction(indicator), formatChange(indicator)),
    el('small', '', reason || `${indicator.source_series || 'NO SOURCE'} · ${indicator.freshness || 'UNKNOWN'}`)
  );
  return button;
}

function renderFavorites(indicators) {
  const container = $('favoritesMonitor');
  clear(container);
  const rows = [...favoriteKeys].map((key) => indicators[key]).filter(Boolean);
  if (!rows.length) {
    container.append(el('p', 'monitor-empty', '관심 지표가 없습니다. 각 지표 표의 ☆ 버튼을 누르면 이 모니터에 고정됩니다.'));
    return;
  }
  rows.forEach((indicator) => container.append(monitorRow(indicator)));
}

function priorityReason(indicator) {
  const freshness = String(indicator?.freshness || 'UNKNOWN');
  if (freshness === 'NO DATA') return { priority: 4, text: `결측 · ${indicator.cadence || 'UNKNOWN'} 데이터 대기` };
  if (freshness.startsWith('STALE')) return { priority: 3, text: `${freshness} · 출처 갱신 확인 필요` };
  const reading = indicatorReading(indicator);
  if (reading.key === 'red') return { priority: 2, text: `${reading.label} · ${reading.hint}` };
  const changePct = finite(indicator?.change_pct);
  if (changePct !== null && Math.abs(changePct) >= 3) return { priority: 1, text: `비교기간 큰 변화 ${signedPercent(changePct)}` };
  return null;
}

function renderPriority(indicators) {
  const container = $('priorityMonitor');
  clear(container);
  const rows = Object.values(indicators)
    .map((indicator) => [indicator, priorityReason(indicator)])
    .filter(([, reason]) => reason)
    .sort((a, b) => b[1].priority - a[1].priority || Math.abs(finite(b[0].change_pct) || 0) - Math.abs(finite(a[0].change_pct) || 0))
    .slice(0, 12);
  if (!rows.length) {
    container.append(el('p', 'monitor-empty', '현재 우선 확인 규칙에 걸린 결측·지연·위험구간·큰 변화가 없습니다.'));
    return;
  }
  rows.forEach(([indicator, reason]) => container.append(monitorRow(indicator, reason.text)));
}

function updateSearchResults(query) {
  const container = $('searchResults');
  const normalized = String(query || '').trim().toLowerCase();
  clear(container);
  if (!normalized) {
    container.hidden = true;
    return;
  }
  const rows = Object.values(lastIndicators)
    .filter((indicator) => [indicator.key, indicator.symbol, indicator.label, indicator.asset_class, indicator.source_series]
      .some((value) => String(value || '').toLowerCase().includes(normalized)))
    .slice(0, 20);
  const heading = el('div', 'search-results-heading');
  heading.append(el('strong', '', `검색결과 ${rows.length}건`), el('span', '', 'Enter 첫 결과 · Esc 닫기'));
  container.append(heading);
  rows.forEach((indicator) => container.append(monitorRow(indicator)));
  if (!rows.length) container.append(el('p', 'monitor-empty', '일치하는 지표가 없습니다. 심볼, 한글명, 출처를 바꿔 검색하세요.'));
  container.hidden = false;
}

function applyWorkspacePreferences() {
  document.body.classList.toggle('readable-density', terminalDensity === 'readable');
  const coreMode = terminalScope === 'core';
  document.querySelectorAll('[data-secondary-section="true"]').forEach((section) => {
    const hasPriority = Boolean(section.querySelector('.indicator-exception'));
    const hasFavorite = [...section.querySelectorAll('[data-favorite-key]')]
      .some((button) => favoriteKeys.has(button.dataset.favoriteKey));
    section.hidden = coreMode && !hasPriority && !hasFavorite;
  });
  const scopeButton = $('scopeButton');
  scopeButton.textContent = coreMode ? '핵심만 표시 중' : '전체 표시 중';
  scopeButton.setAttribute('aria-pressed', String(coreMode));
  const densityButton = $('densityButton');
  densityButton.textContent = terminalDensity === 'readable' ? '큰 글자 표시 중' : '고밀도 표시 중';
  densityButton.setAttribute('aria-pressed', String(terminalDensity === 'readable'));
}

function renderOverviewCentralBank(centralBank) {
  const container = $('overviewCentralBank');
  clear(container);
  const state = centralBankState(centralBank?.score);
  const lead = el('div', `central-bank-overview-lead ${state.key}`);
  lead.append(
    el('span', 'central-bank-kicker', 'CENTRAL BANK LIQUIDITY DIRECTION'),
    el('strong', 'central-bank-overview-score', centralBank?.score == null ? '—' : signed(centralBank.score, 1)),
    el('b', `central-bank-signal ${state.key}`, centralBank?.signal || state.label),
    el('small', '', '−100 숏 압력 · 0 중립 · +100 롱 지원')
  );

  const summary = el('div', 'central-bank-overview-summary');
  summary.append(
    el('h3', '', centralBank?.regime || '중앙은행 데이터 대기'),
    el('p', '', centralBank?.summary || 'Fed·ECB·BOJ 공식 자산과 환율 이력이 모이면 방향성 점수를 계산합니다.')
  );
  const metrics = el('div', 'central-bank-overview-metrics');
  for (const [label, value, hint] of [
    ['G3 FX중립 YoY', signed(centralBank?.g3_yoy, 2, '%'), '검은선 수준'],
    ['3개월 가속도', signed(centralBank?.g3_yoy_acceleration_3m, 2, '%p'), '검은선 기울기'],
    ['최근 3개월 연율', signed(centralBank?.g3_annualized_3m, 2, '%'), '빠른 흐름'],
    ['계산 신뢰도', `${score(centralBank?.confidence)}%`, '가용 가중치']
  ]) {
    const card = el('div', 'central-bank-overview-metric');
    card.append(el('span', '', label), el('strong', '', value), el('small', '', hint));
    metrics.append(card);
  }
  summary.append(metrics);

  const bias = el('div', 'central-bank-bias');
  const longValue = clamp(centralBank?.long_bias);
  const shortValue = clamp(centralBank?.short_bias);
  bias.append(
    el('span', 'central-bank-bias-title', 'DIRECTIONAL BIAS · 확률 아님'),
    biasRow('LONG 지원', longValue, 'long'),
    biasRow('SHORT 압력', shortValue, 'short'),
    el('small', '', `기준일 ${centralBank?.as_of || '—'} · F5 중앙은행 탭에서 원수치와 산식을 확인`)
  );
  container.append(lead, summary, bias);
}

function biasRow(label, value, state) {
  const row = el('div', `central-bank-bias-row ${state}`);
  const heading = el('div');
  heading.append(el('span', '', label), el('strong', '', `${value.toFixed(1)}`));
  const bar = el('div', 'central-bank-bias-track');
  const fill = el('i');
  fill.style.setProperty('--bias-width', `${value}%`);
  bar.append(fill);
  row.append(heading, bar);
  return row;
}

function renderOverviewDirection(indicators) {
  const container = $('overviewRecovery');
  clear(container);
  const markets = ['us', 'korea', 'crypto'].map((name) => priceDirectionModel(name, indicators));
  container.append(priceDirectionCard(overallPriceDirection(markets), true));
  for (const model of markets) container.append(priceDirectionCard(model, true));
  container.append(el(
    'p',
    'recovery-disclaimer',
    '상승·하락·횡보는 표시된 최근 가격 변화만 분류한 결과입니다. 상승확률·예측·매매 권고가 아니며, 모델 위험점수와 별개입니다.'
  ));
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
    ['충격 흡수력', snapshot.resilience, '정책·유동성·완충 여력'],
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

function directionState(returnPct, sidewaysBand) {
  const value = finite(returnPct);
  if (value === null) return { key: 'unknown', label: '데이터 부족', color: '#777' };
  if (value > sidewaysBand) return { key: 'up', label: '상승', color: 'var(--green)' };
  if (value < -sidewaysBand) return { key: 'down', label: '하락', color: 'var(--red)' };
  return { key: 'flat', label: '횡보', color: 'var(--yellow)' };
}

function componentReturn(indicators, definition) {
  const [key, label, field] = definition;
  const indicator = indicators[key];
  return {
    key,
    label,
    returnPct: finite(indicator?.[field]),
    observedAt: indicator?.observed_at || null
  };
}

function priceDirectionModel(name, indicators) {
  const config = PRICE_DIRECTION_CONFIG[name];
  const components = config.components.map((definition) => componentReturn(indicators, definition));
  const known = components.filter((item) => item.returnPct !== null);
  const averageReturn = known.length
    ? known.reduce((sum, item) => sum + item.returnPct, 0) / known.length
    : null;
  return {
    name,
    title: config.title,
    period: config.period,
    sidewaysBand: config.sidewaysBand,
    returnPct: averageReturn,
    state: directionState(averageReturn, config.sidewaysBand),
    components,
    known: known.length
  };
}

function overallPriceDirection(markets) {
  const known = markets.filter((market) => market.returnPct !== null);
  const counts = { up: 0, down: 0, flat: 0 };
  for (const market of known) counts[market.state.key] += 1;
  let key = 'flat';
  if (counts.up > counts.down && counts.up > counts.flat) key = 'up';
  if (counts.down > counts.up && counts.down > counts.flat) key = 'down';
  const state = known.length ? directionState(key === 'up' ? 1 : key === 'down' ? -1 : 0, 0.5) : directionState(null, 0);
  return {
    name: 'overall',
    title: '전체 시장',
    period: '미국·한국·코인 최신 가격',
    sidewaysBand: null,
    returnPct: null,
    state,
    components: markets.map((market) => ({
      key: market.name,
      label: market.title,
      returnPct: market.returnPct,
      directionLabel: market.state.label,
      directionKey: market.state.key
    })),
    known: known.length,
    counts
  };
}

function signedPercent(value, digits = 2) {
  const number = finite(value);
  return number === null ? '—' : `${number > 0 ? '+' : ''}${number.toFixed(digits)}%`;
}

function priceDirectionCard(model, compactMode = false) {
  const card = el('article', `recovery-card direction-${model.state.key}`);
  card.style.setProperty('--direction-color', model.state.color);
  const head = el('div', 'recovery-head');
  const title = el('div');
  title.append(
    el('strong', '', `${model.title} 가격 방향`),
    el('small', '', `${model.period} · 데이터 ${model.known}/${model.components.length}`)
  );
  const result = el('div', `price-direction-result ${model.state.key}`);
  result.append(
    el('b', '', model.state.label),
    el('small', '', model.name === 'overall' ? '시장 수 기준' : `대표가격 평균 ${signedPercent(model.returnPct, 3)}`)
  );
  head.append(title, result);
  card.append(head);
  card.append(el(
    'p',
    'recovery-caption',
    model.name === 'overall'
      ? `상승 ${model.counts?.up || 0} · 하락 ${model.counts?.down || 0} · 횡보 ${model.counts?.flat || 0}`
      : `횡보 기준 ±${model.sidewaysBand.toFixed(2)}% · 가격 변화의 방향만 표시`
  ));
  const list = el('div', compactMode ? 'recovery-condition-grid compact' : 'recovery-condition-grid');
  for (const component of model.components) {
    const state = component.directionKey
      ? { key: component.directionKey, label: component.directionLabel }
      : directionState(component.returnPct, model.sidewaysBand ?? 0);
    const row = el('div', `recovery-condition ${state.key}`);
    row.append(
      el('span', '', component.label),
      el('strong', '', signedPercent(component.returnPct)),
      el('em', '', state.label)
    );
    list.append(row);
  }
  card.append(list);
  return card;
}

function forecastList(payload) {
  return Array.isArray(payload?.dashboard?.forecasts) ? payload.dashboard.forecasts : [];
}

function forecastProbabilityRow(label, value, className) {
  const row = el('div', `forecast-probability ${className}`);
  const number = finite(value) || 0;
  row.append(el('span', '', label), el('strong', '', `${number.toFixed(1)}%`));
  const track = el('div', 'forecast-probability-track');
  const fill = el('i');
  fill.style.width = `${Math.max(0, Math.min(100, number))}%`;
  track.append(fill);
  row.append(track);
  return row;
}

function forecastHorizonCard(horizon) {
  const validationGood = horizon.validation_state === 'VALIDATED / BEATS UNIFORM';
  const card = el('article', `forecast-horizon ${validationGood ? 'validated' : 'unvalidated'}`);
  const header = document.createElement('header');
  header.append(
    el('span', '', `${horizon.horizon_days}거래일 전망`),
    el('strong', '', horizon.dominant_direction || '데이터 부족')
  );
  card.append(header);
  const probabilities = el('div', 'forecast-probabilities');
  probabilities.append(
    forecastProbabilityRow('상승', horizon.rise_probability, 'rise'),
    forecastProbabilityRow('횡보', horizon.sideways_probability, 'sideways'),
    forecastProbabilityRow('하락', horizon.fall_probability, 'fall')
  );
  card.append(probabilities);
  const metrics = el('dl', 'forecast-metrics');
  for (const [label, value] of [
    ['조건부 평균수익', signedPercent(horizon.expected_return_percent)],
    ['횡보기준', `±${Number(horizon.flat_band_percent || 0).toFixed(2)}%`],
    ['유사국면', `${horizon.analog_samples || 0}개`],
    ['순차검증 적중', finite(horizon.validation_hit_rate) === null ? '검증부족' : `${Number(horizon.validation_hit_rate).toFixed(1)}%`],
    ['Brier / 균등', finite(horizon.validation_brier) === null ? '—' : `${Number(horizon.validation_brier).toFixed(3)} / ${Number(horizon.uniform_brier).toFixed(3)}`]
  ]) {
    metrics.append(el('dt', '', label), el('dd', '', value));
  }
  card.append(metrics);
  const validation = el('p', `forecast-validation ${validationGood ? 'good' : 'weak'}`,
    validationGood ? '과거 순차검증에서 균등확률보다 오차가 작음' : '검증 우위 없음 또는 표본 부족 · 방향판단에 사용 금지');
  card.append(validation, el('small', 'forecast-warning', horizon.warning));
  return card;
}

function renderMarketForecast(payload, market) {
  const forecast = forecastList(payload).find((item) => item.market === market);
  const panel = el('section', 'terminal-panel market-forecast-panel');
  const heading = el('div', 'panel-heading');
  heading.append(el('span', '', 'PROBABILITY LAB'), el('strong', '', `${forecast?.label || market} 다중기간 확률`));
  panel.append(heading);
  if (!forecast || !Array.isArray(forecast.horizons) || !forecast.horizons.length) {
    panel.append(el('p', 'forecast-empty', `${forecast?.status || 'NO FORECAST'} · 최소 과거 표본이 모일 때까지 확률을 만들지 않습니다.`));
    return panel;
  }
  const regime = forecast.current_regime;
  if (regime) {
    const current = el('div', 'forecast-regime');
    current.append(
      metricSummary('최근 1주', signedPercent(regime.return_1w), '가격수익률'),
      metricSummary('최근 1개월', signedPercent(regime.return_1m), '가격수익률'),
      metricSummary('최근 3개월', signedPercent(regime.return_3m), '가격수익률'),
      metricSummary('20일 실현변동성', `${Number(regime.realized_volatility_20d).toFixed(1)}%`, '연율화')
    );
    panel.append(current);
  }
  const grid = el('div', 'forecast-horizon-grid');
  forecast.horizons.forEach((horizon) => grid.append(forecastHorizonCard(horizon)));
  panel.append(grid, el('p', 'forecast-methodology', `${forecast.methodology} · 원천 ${forecast.source_series} · 기준 ${forecast.as_of || '—'}`));
  return panel;
}

function renderForecastOverview(payload) {
  const container = $('forecastOverview');
  clear(container);
  for (const forecast of forecastList(payload)) {
    const horizon = forecast.horizons?.find((item) => item.horizon_days === 5) || forecast.horizons?.[0];
    const card = el('article', 'forecast-summary-card');
    card.append(el('span', '', forecast.label), el('strong', '', horizon?.dominant_direction || '계산 보류'));
    if (horizon) {
      card.append(el('p', '', `상승 ${Number(horizon.rise_probability).toFixed(1)} · 횡보 ${Number(horizon.sideways_probability).toFixed(1)} · 하락 ${Number(horizon.fall_probability).toFixed(1)}`));
      card.append(el('small', '', horizon.validation_state === 'VALIDATED / BEATS UNIFORM'
        ? `5거래일 · 순차검증 ${Number(horizon.validation_hit_rate).toFixed(1)}% · 검증우위 있음`
        : '검증우위 없음 또는 표본 부족 · 관찰용'));
    } else {
      card.append(el('p', '', forecast.status), el('small', '', '표본을 임의 추정하지 않음'));
    }
    card.addEventListener('click', () => selectTab(forecast.market));
    container.append(card);
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
  summary.append(el('h3', '', `${config.title} 모델 위험도 · ${state.label}`));
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

  const directionPanel = el('section', 'terminal-panel market-recovery-panel');
  const directionHeading = el('div', 'panel-heading');
  directionHeading.append(el('span', '', 'PRICE DIRECTION'), el('strong', '', `${config.title} 상승·하락·횡보`));
  const directionName = config.riskKey === 'US_EQUITY' ? 'us' : config.riskKey === 'KOREA_EQUITY' ? 'korea' : 'crypto';
  directionPanel.append(directionHeading, priceDirectionCard(priceDirectionModel(directionName, indicators)));
  container.append(directionPanel);
  container.append(renderMarketForecast(payload, directionName));

  if (directionName === 'crypto') {
    container.append(renderCryptoRegime(indicators));
  }

  const assetGrid = el('div', 'asset-grid');
  config.sections.forEach(([title, code, keys], sectionIndex) => {
    const panel = el('details', 'terminal-panel market-section');
    panel.open = true;
    panel.dataset.secondarySection = String(sectionIndex >= 4);
    const heading = el('summary', 'panel-heading');
    heading.append(el('span', '', code), el('strong', '', title));
    panel.append(heading, indicatorTable(keys, indicators, title));
    assetGrid.append(panel);
  });
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
  applyWorkspacePreferences();
}

function renderFunding(indicators) {
  const container = $(FUNDING_CONFIG.target);
  clear(container);
  const available = FUNDING_CONFIG.sections
    .flatMap(([, , keys]) => keys)
    .map((key) => indicators[key])
    .filter((indicator) => finite(indicator?.value) !== null);
  const missing = FUNDING_CONFIG.sections
    .flatMap(([, , keys]) => keys)
    .filter((key) => finite(indicators[key]?.value) === null).length;
  const hero = el('section', 'funding-hero');
  hero.append(
    metricSummary('수집 지표', `${available.length}개`, 'OFR·CFTC·Deribit·Coin Metrics'),
    metricSummary('결측', `${missing}개`, '추정값으로 채우지 않음'),
    metricSummary('SOFR 꼬리차', formatValue(indicators.sofr_tail || {}), '99백분위 − 중앙값'),
    metricSummary('BTC DVOL', formatValue(indicators.btc_dvol || {}), '옵션 기대변동성'),
    metricSummary('스테이블코인', formatValue(indicators.stablecoin_cap || {}), 'USDT+USDC 합산')
  );
  container.append(hero);
  const assetGrid = el('div', 'asset-grid');
  FUNDING_CONFIG.sections.forEach(([title, code, keys], sectionIndex) => {
    const panel = el('details', 'terminal-panel market-section');
    panel.open = true;
    panel.dataset.secondarySection = String(sectionIndex >= 4);
    const heading = el('summary', 'panel-heading');
    heading.append(el('span', '', code), el('strong', '', title));
    panel.append(heading, indicatorTable(keys, indicators, title));
    assetGrid.append(panel);
  });
  container.append(assetGrid);
  applyWorkspacePreferences();
}

function metricSummary(label, value, note) {
  const card = el('article', 'funding-metric');
  card.append(el('span', '', label), el('strong', '', value), el('small', '', note));
  return card;
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
      body = '가격과 미결제약정이 함께 하락 중입니다. OI 감소가 멈추는지와 현물가격이 상승·하락·횡보 중 무엇으로 바뀌는지 확인하세요.';
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

function indicatorTable(keys, indicators, label = '시장 지표') {
  const table = el('table', 'indicator-table');
  table.setAttribute('aria-label', label);
  const head = document.createElement('thead');
  const headRow = document.createElement('tr');
  for (const title of ['지표', '현재값 / RAW', '직전 변화', '최근 범위', '쉬운 해석', '추세', '기준·출처']) {
    const cell = el('th', '', title);
    cell.scope = 'col';
    headRow.append(cell);
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
    row.dataset.indicatorKey = key;
    row.classList.toggle('indicator-exception', Boolean(priorityReason(indicator)));
    const nameCell = document.createElement('td');
    const favorite = el('button', `favorite-button${favoriteKeys.has(key) ? ' active' : ''}`, favoriteKeys.has(key) ? '★' : '☆');
    favorite.type = 'button';
    favorite.dataset.favoriteKey = key;
    favorite.setAttribute('aria-label', `${indicator.label} 관심 지표 ${favoriteKeys.has(key) ? '제거' : '추가'}`);
    favorite.setAttribute('aria-pressed', String(favoriteKeys.has(key)));
    favorite.addEventListener('click', () => toggleFavorite(key));
    nameCell.append(
      favorite,
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
    const freshness = String(indicator.freshness || 'UNKNOWN');
    const freshnessClass = freshness.startsWith('STALE') || freshness === 'NO DATA'
      ? 'stale'
      : ['DELAYED', 'SESSION CLOSE', 'LATEST CLOSE'].includes(freshness)
        ? 'delayed'
        : '';
    sourceCell.append(
      el('span', '', indicator.observed_at ? String(indicator.observed_at).slice(0, 16) : '—'),
      el('small', '', indicator.source_series || 'NO SOURCE'),
      el('small', `freshness ${freshnessClass}`, `${indicator.cadence || 'UNKNOWN'} · ${freshness}`)
    );
    row.append(sourceCell);
    body.append(row);
  }

  table.append(head, body);
  return table;
}

function centralBankMetric(label, value, note, state = '') {
  const card = el('div', `central-bank-metric ${state}`.trim());
  card.append(el('span', '', label), el('strong', '', value), el('small', '', note));
  return card;
}

function centralBankLineChart(history) {
  const wrap = el('div', 'central-bank-chart-wrap');
  const rows = Array.isArray(history) ? history : [];
  const definitions = [
    ['g3_yoy', 'G3 FX중립 · 검은선', 'series-g3'],
    ['fed_yoy', 'Fed YoY', 'series-fed'],
    ['ecb_yoy', 'ECB YoY', 'series-ecb'],
    ['boj_yoy', 'BOJ YoY', 'series-boj'],
    ['g3_usd_yoy', 'G3 USD환산 YoY', 'series-usd']
  ];
  const values = rows.flatMap((row) => definitions.map(([key]) => finite(row[key])).filter((value) => value !== null));
  if (rows.length < 2 || values.length < 2) {
    wrap.append(el('p', 'central-bank-empty', '최소 1년 이상의 Fed·ECB·BOJ 공식 이력이 수집되면 증가율 차트가 표시됩니다.'));
    return wrap;
  }

  const width = 1000;
  const height = 300;
  const left = 62;
  const right = 18;
  const top = 18;
  const bottom = 42;
  const minimum = Math.min(0, ...values);
  const maximum = Math.max(0, ...values);
  const padding = Math.max(2, (maximum - minimum) * 0.08);
  const low = minimum - padding;
  const high = maximum + padding;
  const x = (index) => left + (index / Math.max(1, rows.length - 1)) * (width - left - right);
  const y = (value) => top + ((high - value) / Math.max(0.001, high - low)) * (height - top - bottom);
  const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  svg.setAttribute('viewBox', `0 0 ${width} ${height}`);
  svg.setAttribute('role', 'img');
  svg.setAttribute('aria-label', 'G3 중앙은행 총자산 증가율 추이');

  for (let index = 0; index <= 4; index += 1) {
    const value = high - ((high - low) * index / 4);
    const grid = document.createElementNS('http://www.w3.org/2000/svg', 'line');
    grid.setAttribute('x1', left);
    grid.setAttribute('x2', width - right);
    grid.setAttribute('y1', y(value));
    grid.setAttribute('y2', y(value));
    grid.setAttribute('class', Math.abs(value) < (high - low) / 8 ? 'central-bank-zero' : 'central-bank-gridline');
    const label = document.createElementNS('http://www.w3.org/2000/svg', 'text');
    label.setAttribute('x', left - 8);
    label.setAttribute('y', y(value) + 4);
    label.setAttribute('class', 'central-bank-axis-label');
    label.setAttribute('text-anchor', 'end');
    label.textContent = `${value.toFixed(1)}%`;
    svg.append(grid, label);
  }

  const zero = document.createElementNS('http://www.w3.org/2000/svg', 'line');
  zero.setAttribute('x1', left);
  zero.setAttribute('x2', width - right);
  zero.setAttribute('y1', y(0));
  zero.setAttribute('y2', y(0));
  zero.setAttribute('class', 'central-bank-zero');
  svg.append(zero);

  for (const [key, , className] of definitions) {
    let drawing = false;
    let pathData = '';
    rows.forEach((row, index) => {
      const value = finite(row[key]);
      if (value === null) {
        drawing = false;
        return;
      }
      pathData += `${drawing ? ' L' : ' M'} ${x(index).toFixed(1)} ${y(value).toFixed(1)}`;
      drawing = true;
    });
    const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
    path.setAttribute('d', pathData.trim());
    path.setAttribute('class', `central-bank-series ${className}`);
    svg.append(path);
  }

  for (const index of [0, Math.floor((rows.length - 1) / 2), rows.length - 1]) {
    const label = document.createElementNS('http://www.w3.org/2000/svg', 'text');
    label.setAttribute('x', x(index));
    label.setAttribute('y', height - 12);
    label.setAttribute('class', 'central-bank-axis-label');
    label.setAttribute('text-anchor', index === 0 ? 'start' : index === rows.length - 1 ? 'end' : 'middle');
    label.textContent = String(rows[index]?.observed_at || '').slice(0, 7);
    svg.append(label);
  }

  const legend = el('div', 'central-bank-chart-legend');
  for (const [, label, className] of definitions) {
    const item = el('span', className);
    item.append(el('i'), document.createTextNode(label));
    legend.append(item);
  }
  wrap.append(svg, legend, el('p', 'central-bank-chart-note', 'G3 FX중립선은 각 중앙은행의 현지통화 YoY를 현재 달러환산 자산규모로 가중합니다. USD환산선은 환율 효과까지 포함하므로 정책 방향 판단에는 FX중립선을 우선합니다.'));
  return wrap;
}

function renderCentralBank(centralBank) {
  const state = centralBankState(centralBank?.score);
  const hero = $('centralBankHero');
  clear(hero);
  const scoreBlock = el('div', `central-bank-score-block ${state.key}`);
  scoreBlock.append(
    el('span', '', 'LIQUIDITY DIRECTION SCORE'),
    el('strong', '', centralBank?.score == null ? '—' : signed(centralBank.score, 1)),
    el('b', '', centralBank?.signal || state.label),
    el('small', '', '−100 숏 압력 · +100 롱 지원 · 수익 확률 아님')
  );
  const heroCopy = el('div', 'central-bank-hero-copy');
  heroCopy.append(
    el('span', 'central-bank-kicker', 'G3 BALANCE SHEET / RATE OF CHANGE'),
    el('h3', '', centralBank?.regime || '공식 데이터 수집 대기'),
    el('p', '', centralBank?.summary || 'Fed·ECB·BOJ 공식 총자산 이력이 충분하지 않아 방향을 단정하지 않습니다.')
  );
  const heroMetrics = el('div', 'central-bank-hero-metrics');
  for (const metric of [
    centralBankMetric('G3 총자산', finite(centralBank?.g3_total_usd_trillion) === null ? '—' : `$${score(centralBank.g3_total_usd_trillion, 2)}T`, '현재 환율 환산 절대규모'),
    centralBankMetric('G3 FX중립 YoY', signed(centralBank?.g3_yoy, 2, '%'), '검은선의 현재 높이'),
    centralBankMetric('YoY 3개월 가속도', signed(centralBank?.g3_yoy_acceleration_3m, 2, '%p'), '검은선이 올라가는지'),
    centralBankMetric('G3 최근 3개월 연율', signed(centralBank?.g3_annualized_3m, 2, '%'), '최근 자산 흐름'),
    centralBankMetric('G3 USD환산 YoY', signed(centralBank?.g3_usd_yoy, 2, '%'), '환율 효과 포함'),
    centralBankMetric('계산 신뢰도', finite(centralBank?.confidence) === null ? '—' : `${score(centralBank.confidence)}%`, '가용 가중치')
  ]) heroMetrics.append(metric);
  heroCopy.append(heroMetrics);
  const heroBias = el('div', 'central-bank-hero-bias');
  heroBias.append(
    el('span', '', 'LONG / SHORT BIAS'),
    biasRow('LONG 지원', clamp(centralBank?.long_bias), 'long'),
    biasRow('SHORT 압력', clamp(centralBank?.short_bias), 'short'),
    el('small', '', `종합 기준일 ${centralBank?.as_of || '—'}`)
  );
  hero.append(scoreBlock, heroCopy, heroBias);

  const chart = $('centralBankChart');
  clear(chart);
  chart.append(centralBankLineChart(centralBank?.history));

  const components = $('centralBankComponents');
  clear(components);
  const componentRows = Array.isArray(centralBank?.components) ? centralBank.components : [];
  if (!componentRows.length) {
    components.append(el('p', 'central-bank-empty', '중앙은행별 공식 데이터가 아직 없습니다.'));
  }
  for (const component of componentRows) {
    const card = el('article', 'central-bank-component');
    const header = el('header');
    const title = el('div');
    title.append(el('span', '', String(component.key || '').toUpperCase()), el('h3', '', component.name || component.series));
    header.append(title, el('b', '', component.freshness || 'NO DATA'));
    const raw = centralBankMetric(
      `원본 ${component.series || ''}`,
      numberValue(component.native_value, 1),
      component.native_unit || '—'
    );
    const metricGrid = el('div', 'central-bank-component-metrics');
    metricGrid.append(
      raw,
      centralBankMetric('달러 환산 자산', finite(component.usd_trillion) === null ? '—' : `$${score(component.usd_trillion, 3)}T`, '현재 환율 적용'),
      centralBankMetric('전년비 YoY', signed(component.yoy, 2, '%'), '증가율 수준'),
      centralBankMetric('1개월 변화', signed(component.change_1m, 2, '%'), '단기 변화'),
      centralBankMetric('3개월 연율', signed(component.annualized_3m, 2, '%'), '최근 속도'),
      centralBankMetric('YoY 가속도', signed(component.yoy_acceleration_3m, 2, '%p'), '3개월 전 YoY 대비')
    );
    const meanings = el('div', 'central-bank-meaning-grid');
    meanings.append(
      centralBankMeaning('증가할 때', component.increase_meaning, 'increase'),
      centralBankMeaning('감소할 때', component.decrease_meaning, 'decrease')
    );
    card.append(
      header,
      metricGrid,
      el('p', 'central-bank-interpretation', component.interpretation || '해석 데이터 없음'),
      meanings,
      el('small', 'central-bank-source', `${component.source_series || 'NO SOURCE'} · ${component.observed_at || '—'}`)
    );
    components.append(card);
  }

  const factors = $('centralBankFactors');
  clear(factors);
  const factorRows = Array.isArray(centralBank?.factors) ? centralBank.factors : [];
  if (!factorRows.length) factors.append(el('p', 'central-bank-empty', '계산 가능한 점수 요인이 없습니다.'));
  for (const factor of factorRows) {
    const factorState = centralBankState(factor.score);
    const card = el('article', `central-bank-factor ${factorState.key}`);
    const heading = el('header');
    const title = el('div');
    title.append(el('span', '', String(factor.key || '').toUpperCase()), el('h3', '', factor.label || factor.key));
    heading.append(title, el('b', '', `가중치 ${score(factor.weight, 0)}%`));
    const scoreTrack = el('div', 'central-bank-factor-track');
    const marker = el('i');
    marker.style.setProperty('--factor-position', `${clamp((finite(factor.score) ?? 0) / 2 + 50)}%`);
    scoreTrack.append(marker);
    const row = el('div', 'central-bank-factor-values');
    row.append(
      centralBankMetric('현재 원수치', signed(factor.value, 2, factor.unit || ''), '공식·파생 관측'),
      centralBankMetric('정규화 점수', signed(factor.score, 1), '−100 ~ +100'),
      centralBankMetric('종합 기여도', signed(factor.contribution, 1), '가중치 적용')
    );
    const meanings = el('div', 'central-bank-meaning-grid');
    meanings.append(
      centralBankMeaning('값이 상승할 때', factor.increase_meaning, 'increase'),
      centralBankMeaning('값이 하락할 때', factor.decrease_meaning, 'decrease')
    );
    card.append(
      heading,
      scoreTrack,
      row,
      el('p', 'central-bank-interpretation', factor.interpretation || '해석 데이터 없음'),
      meanings,
      el('small', 'central-bank-source', `${factor.source_series || 'NO SOURCE'} · ${factor.observed_at || '—'}`)
    );
    factors.append(card);
  }

  renderCentralBankPlumbing(centralBank?.fed_plumbing);
  renderCentralBankJudgement(centralBank, factorRows);
  renderCentralBankMethodology(centralBank);
}

function centralBankMeaning(label, text, state) {
  const box = el('div', `central-bank-meaning ${state}`);
  box.append(el('strong', '', label), el('p', '', text || '—'));
  return box;
}

function renderCentralBankPlumbing(plumbing) {
  const container = $('centralBankPlumbing');
  clear(container);
  if (!plumbing) {
    container.append(el('p', 'central-bank-empty', 'Fed 자금시장 데이터가 아직 없습니다.'));
    return;
  }
  container.append(
    el('div', 'central-bank-formula', plumbing.formula || 'Fed 총자산 − TGA − ON RRP')
  );
  const grid = el('div', 'central-bank-plumbing-grid');
  grid.append(
    centralBankMetric('Fed 순유동성', finite(plumbing.net_liquidity_usd_trillion) === null ? '—' : `$${score(plumbing.net_liquidity_usd_trillion, 3)}T`, '총자산에서 TGA·ON RRP 차감'),
    centralBankMetric('순유동성 YoY', signed(plumbing.net_liquidity_yoy, 2, '%'), '전년 대비'),
    centralBankMetric('순유동성 3개월 연율', signed(plumbing.net_liquidity_3m_annualized, 2, '%'), '최근 방향'),
    centralBankMetric('은행 지급준비금', finite(plumbing.reserve_balances_usd_trillion) === null ? '—' : `$${score(plumbing.reserve_balances_usd_trillion, 3)}T`, 'WRESBAL'),
    centralBankMetric('준비금 3개월 연율', signed(plumbing.reserve_balances_3m_annualized, 2, '%'), '결제·레포 완충재'),
    centralBankMetric('미 재무부 TGA', finite(plumbing.treasury_general_account_usd_trillion) === null ? '—' : `$${score(plumbing.treasury_general_account_usd_trillion, 3)}T`, '증가하면 은행 유동성 흡수'),
    centralBankMetric('ON RRP', finite(plumbing.reverse_repo_usd_trillion) === null ? '—' : `$${score(plumbing.reverse_repo_usd_trillion, 3)}T`, '감소하면 민간시장으로 현금 이동')
  );
  container.append(
    grid,
    el('p', 'central-bank-plumbing-explanation', plumbing.explanation || ''),
    el('small', 'central-bank-source', `Fed 공식 FRED 관측 · 공통 기준일 ${plumbing.observed_at || '—'}`)
  );
}

function renderCentralBankJudgement(centralBank, factors) {
  const container = $('centralBankJudgement');
  clear(container);
  const state = centralBankState(centralBank?.score);
  const heading = el('div', `central-bank-judgement-head ${state.key}`);
  heading.append(
    el('span', '', centralBank?.signal || state.label),
    el('strong', '', centralBank?.score == null ? '—' : signed(centralBank.score, 1)),
    el('small', '', centralBank?.regime || '데이터 부족')
  );
  const available = factors.filter((factor) => finite(factor.score) !== null);
  const supportive = [...available].sort((a, b) => Number(b.score) - Number(a.score)).filter((factor) => factor.score > 0).slice(0, 2);
  const restrictive = [...available].sort((a, b) => Number(a.score) - Number(b.score)).filter((factor) => factor.score < 0).slice(0, 2);
  const drivers = el('div', 'central-bank-driver-grid');
  drivers.append(
    centralBankDriver('롱 지원 요인', supportive, 'long'),
    centralBankDriver('숏 압력 요인', restrictive, 'short')
  );
  const conditions = el('div', 'central-bank-condition-grid');
  conditions.append(
    centralBankCondition('롱 환경 강화', 'G3 YoY가 상승하고, 가속도·최근 3개월 흐름·Fed 순유동성·준비금이 함께 플러스로 정렬될 때 신뢰도가 높아집니다.', 'long'),
    centralBankCondition('숏 환경 강화', 'G3 YoY가 하락하고, 검은선 기울기와 최근 흐름이 음수이며 Fed 순유동성·준비금까지 동시에 감소할 때 경계합니다.', 'short'),
    centralBankCondition('혼조·판단 보류', '자산수준과 속도가 엇갈리거나 가용 가중치가 60% 미만이면 단독 진입 근거로 쓰지 않습니다.', 'neutral')
  );
  container.append(
    heading,
    el('p', 'central-bank-judgement-summary', centralBank?.summary || '종합판단에 필요한 데이터가 부족합니다.'),
    drivers,
    conditions,
    el('p', 'central-bank-warning', '이 신호는 중앙은행 유동성의 방향을 요약하며 주가 수익률의 확률·목표가격·자동 매매 신호가 아닙니다. 시장 가격·신용·변동성 탭과 함께 확인하십시오.')
  );
}

function centralBankDriver(title, factors, state) {
  const box = el('div', `central-bank-driver ${state}`);
  box.append(el('strong', '', title));
  const list = el('ul');
  if (!factors.length) list.append(el('li', '', '뚜렷한 요인 없음'));
  for (const factor of factors) {
    list.append(el('li', '', `${factor.label}: ${signed(factor.value, 2, factor.unit || '')} / 점수 ${signed(factor.score, 1)}`));
  }
  box.append(list);
  return box;
}

function centralBankCondition(title, text, state) {
  const box = el('div', `central-bank-condition ${state}`);
  box.append(el('strong', '', title), el('p', '', text));
  return box;
}

function renderCentralBankMethodology(centralBank) {
  const container = $('centralBankMethodology');
  clear(container);
  const method = el('section');
  method.append(el('h3', '', '계산 방법'));
  const methodList = el('ol');
  for (const item of centralBank?.methodology || ['공식 데이터가 수집되면 계산 방법을 표시합니다.']) {
    methodList.append(el('li', '', item));
  }
  method.append(methodList);
  const caveats = el('section', 'central-bank-caveats');
  caveats.append(el('h3', '', '반드시 함께 볼 한계'));
  const caveatList = el('ul');
  for (const item of centralBank?.caveats || ['결측값은 임의로 채우지 않습니다.']) {
    caveatList.append(el('li', '', item));
  }
  caveats.append(caveatList);
  container.append(method, caveats);
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
  lastPayload = payload;
  lastIndicators = indicators;
  officialKrIndices = new Map(
    ['kospi', 'kosdaq']
      .map((key) => [key, indicators[key]])
      .filter(([, indicator]) => indicator)
  );
  renderKrIndexCards();
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
    '중앙은행 탭',
    () => renderCentralBank(payload.dashboard?.central_bank),
    errors
  );
  renderSafely(
    '자금·포지션 탭',
    () => renderFunding(indicators),
    errors
  );
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

function selectTab(name, moveFocus = false) {
  if (!TAB_NAMES.includes(name)) return;
  document.querySelectorAll('.tab-button').forEach((button) => {
    const selected = button.dataset.tab === name;
    button.classList.toggle('active', selected);
    button.setAttribute('aria-selected', String(selected));
    button.tabIndex = selected ? 0 : -1;
    if (selected && moveFocus) button.focus();
  });
  document.querySelectorAll('.tab-panel').forEach((panel) => {
    panel.hidden = panel.id !== `tab-${name}`;
  });
}

document.querySelectorAll('.tab-button').forEach((button) => {
  button.addEventListener('click', () => selectTab(button.dataset.tab, true));
});

document.querySelector('.market-tabs').addEventListener('keydown', (event) => {
  const current = TAB_NAMES.indexOf(document.activeElement?.dataset?.tab);
  if (current < 0 || !['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
  event.preventDefault();
  const next = event.key === 'Home'
    ? 0
    : event.key === 'End'
      ? TAB_NAMES.length - 1
      : (current + (event.key === 'ArrowRight' ? 1 : -1) + TAB_NAMES.length) % TAB_NAMES.length;
  selectTab(TAB_NAMES[next], true);
});

document.addEventListener('keydown', (event) => {
  const shortcut = {
    F1: 'overview',
    F2: 'us',
    F3: 'korea',
    F4: 'crypto',
    F5: 'centralbank',
    F6: 'funding'
  }[event.key];
  if (shortcut) {
    event.preventDefault();
    selectTab(shortcut);
  }
  if (event.key === '/' && document.activeElement !== $('terminalSearch')) {
    event.preventDefault();
    $('terminalSearch').focus();
    $('terminalSearch').select();
  }
  if (event.key === 'Escape') {
    $('searchResults').hidden = true;
    $('terminalSearch').blur();
  }
});

$('refreshButton').addEventListener('click', requestRefresh);
$('terminalSearch').addEventListener('input', (event) => updateSearchResults(event.target.value));
$('terminalSearch').addEventListener('keydown', (event) => {
  if (event.key !== 'Enter') return;
  const first = $('searchResults').querySelector('.monitor-row');
  if (first) {
    event.preventDefault();
    first.click();
  }
});
$('scopeButton').addEventListener('click', () => {
  terminalScope = terminalScope === 'all' ? 'core' : 'all';
  localStorage.setItem('economics-radar-scope', terminalScope);
  applyWorkspacePreferences();
});
$('densityButton').addEventListener('click', () => {
  terminalDensity = terminalDensity === 'compact' ? 'readable' : 'compact';
  localStorage.setItem('economics-radar-density', terminalDensity);
  applyWorkspacePreferences();
});
selectTab('overview');
applyWorkspacePreferences();
loadDashboard();
loadKrIndexReference();
loadRefreshStatus();
setInterval(loadDashboard, 60000);
setInterval(loadKrIndexReference, 30000);
setInterval(loadRefreshStatus, 3000);
