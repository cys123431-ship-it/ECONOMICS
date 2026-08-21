const fs = require('fs');
const vm = require('vm');

class FakeClassList {
  add() {}
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
}

const ids = Object.fromEntries([
  'connectionDot',
  'connectionText',
  'refreshButton',
  'tickerTape',
  'errorBanner',
  'overviewAsOf',
  'overviewGauges',
  'marketLights',
  'overviewQuotes',
  'riskHeatmap',
  'proprietarySignals',
  'sourceHealth',
  'usMarket',
  'koreaMarket',
  'cryptoMarket',
  'lastUpdated'
].map((id) => [id, new FakeElement('div', id)]));

const tabNames = ['overview', 'us', 'korea', 'crypto'];
const tabs = tabNames.map((name) => {
  const node = new FakeElement('button');
  node.dataset.tab = name;
  return node;
});
const panels = tabNames.map((name) => new FakeElement('section', `tab-${name}`));
const asofs = [new FakeElement('span'), new FakeElement('span'), new FakeElement('span')];

const document = {
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
  addEventListener() {}
};

const context = {
  console,
  document,
  fetch: () => new Promise(() => {}),
  setInterval: () => 0,
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
    indicators: []
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
  'cryptoMarket'
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

if (fs.readFileSync('src/dashboard.js', 'utf8').includes('NODE_LABELS')) {
  throw new Error('legacy NODE_LABELS reference remains');
}

console.log('dashboard smoke test passed');
