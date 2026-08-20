const $ = (id) => document.getElementById(id);

const LABELS = {
  US_EQUITY: '미국 주식', KOREA_EQUITY: '한국 주식', CRYPTO: '가상자산',
  GROWTH: '성장', CREDIT: '신용', USD: '달러', HOUSING: '주택',
  VOLATILITY: '변동성', FOREIGN_TREASURY_DEMAND: '해외 국채수요',
  TREASURY_AUCTION: '미 국채 입찰', RATES: '금리', LABOR: '고용',
  LEVERAGE: '레버리지', BANKING: '은행', KOREA_FIN_STAB: '한국 금융안정',
  FINCOND: '금융여건', CRYPTO_DERIVATIVES: '가상자산 파생',
  KOREA_MARKET_INTERNALS: '한국시장 내부수급', BUSINESS_DEBT: '기업부채',
  LIQUIDITY: '유동성'
};

const CORE_METRICS = [
  ['stress', '시장 스트레스', '현재 충격과 압력의 강도'],
  ['vulnerability', '구조적 취약성', '충격을 증폭할 수 있는 기반 위험'],
  ['resilience', '회복 탄력성', '정책·유동성·완충 여력'],
  ['diffusion', '위험 확산 단계', '여러 시장으로 번진 위험의 단계']
];

function finite(value) {
  const number = Number(value);
  return Number.isFinite(number) ? number : null;
}

function score(value, digits = 1) {
  const number = finite(value);
  return number === null ? '—' : number.toFixed(digits);
}

function percent(value) {
  const number = finite(value);
  return number === null ? '—' : `${number.toFixed(1)}%`;
}

function clamp(value) {
  const number = finite(value);
  return number === null ? 0 : Math.max(0, Math.min(100, number));
}

function formatTime(value) {
  if (!value) return '—';
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return String(value);
  return new Intl.DateTimeFormat('ko-KR', {
    dateStyle: 'medium', timeStyle: 'short', timeZone: 'Asia/Seoul'
  }).format(date);
}

function riskBand(value) {
  const number = finite(value);
  if (number === null) return { key: 'unknown', label: '데이터 부족', message: '충분한 표본이 없어 위험도를 확정하지 않았습니다.' };
  if (number >= 80) return { key: 'critical', label: '심각', message: '광범위한 고위험 신호가 확인됩니다.' };
  if (number >= 65) return { key: 'danger', label: '위험', message: '복수 영역의 위험 신호를 면밀히 확인해야 합니다.' };
  if (number >= 50) return { key: 'warning', label: '경계', message: '평균보다 높은 위험 신호가 누적되고 있습니다.' };
  if (number >= 35) return { key: 'watch', label: '주의', message: '일부 취약 영역을 중심으로 주의가 필요합니다.' };
  return { key: 'stable', label: '안정', message: '현재 종합 위험도는 비교적 낮은 구간입니다.' };
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

function addProgress(parent, value, label) {
  const progress = document.createElement('progress');
  progress.max = 100;
  progress.value = clamp(value);
  progress.setAttribute('aria-label', label);
  parent.append(progress);
}

function renderCore(snapshot) {
  const container = $('coreMetrics');
  clear(container);
  for (const [key, label, hint] of CORE_METRICS) {
    const raw = snapshot[key];
    const card = el('article', 'metric-card');
    card.append(el('span', 'label', label));
    card.append(el('strong', '', key === 'diffusion' ? score(raw, 0) : score(raw)));
    card.append(el('small', '', hint));
    addProgress(card, key === 'diffusion' ? clamp(Number(raw) * 20) : raw, label);
    container.append(card);
  }
}

function renderRanks(targetId, values, limit) {
  const container = $(targetId);
  clear(container);
  const entries = Object.entries(values || {})
    .filter(([, value]) => finite(value) !== null)
    .sort((left, right) => Number(right[1]) - Number(left[1]))
    .slice(0, limit);
  if (!entries.length) {
    container.append(el('p', 'empty-state', '표시할 데이터가 없습니다.'));
    return;
  }
  for (const [name, value] of entries) {
    const row = el('div', 'rank-row');
    row.append(el('span', 'name', LABELS[name] || name));
    addProgress(row, value, LABELS[name] || name);
    row.append(el('span', 'value', score(value)));
    container.append(row);
  }
}

function renderSources(sources) {
  const container = $('sources');
  clear(container);
  const entries = Object.entries(sources || {}).sort((left, right) => {
    return Number(right[1]?.fresh) - Number(left[1]?.fresh) || left[0].localeCompare(right[0]);
  });
  const freshCount = entries.filter(([, state]) => state?.fresh).length;
  $('sourceSummary').textContent = `${freshCount}/${entries.length} 정상`;
  for (const [name, state] of entries) {
    const row = el('div', 'source-row');
    row.append(el('span', `fresh-dot${state?.fresh ? ' fresh' : ''}`));
    const copy = el('div');
    copy.append(el('div', 'source-name', name));
    copy.append(el('small', '', state?.latest_observed_at ? `최근 관측 ${state.latest_observed_at}` : '수집 데이터 없음'));
    row.append(copy);
    row.append(el('small', '', state?.fresh ? '정상' : '확인 필요'));
    container.append(row);
  }
}

function renderCauses(causes) {
  const container = $('causes');
  clear(container);
  const items = Array.isArray(causes) ? causes.slice(0, 12) : [];
  if (!items.length) {
    container.append(el('li', 'empty-state', '현재 표시할 주요 원인이 없습니다.'));
    return;
  }
  for (const cause of items) container.append(el('li', '', String(cause)));
}

function severityClass(value) {
  const normalized = String(value || 'INFO').toLowerCase();
  return ['red', 'orange', 'yellow', 'green', 'info'].includes(normalized) ? normalized : 'info';
}

function renderRules(snapshot) {
  const container = $('ruleHits');
  clear(container);
  const hits = Array.isArray(snapshot.rule_hits) ? snapshot.rule_hits.slice(0, 15) : [];
  $('ruleSummary').textContent = `${snapshot.rules_triggered ?? hits.length}개 발동`;
  if (!hits.length) {
    const row = document.createElement('tr');
    const cell = el('td', 'empty-state', '현재 발동한 주요 규칙이 없습니다.');
    cell.colSpan = 4;
    row.append(cell);
    container.append(row);
    return;
  }
  for (const hit of hits) {
    const row = document.createElement('tr');
    const severityCell = document.createElement('td');
    severityCell.append(el('span', `severity ${severityClass(hit.severity)}`, hit.severity || 'INFO'));
    row.append(severityCell);
    row.append(el('td', '', `${hit.id || '—'} · ${hit.title || '제목 없음'}`));
    row.append(el('td', '', hit.scope || '—'));
    row.append(el('td', '', hit.message || hit.condition || '—'));
    container.append(row);
  }
}

function renderDashboard(snapshot) {
  const globalRisk = finite(snapshot.global_risk);
  const band = riskBand(globalRisk);
  document.body.dataset.risk = band.key;
  $('globalRisk').textContent = score(globalRisk);
  $('riskBand').textContent = band.label;
  $('riskMessage').textContent = band.message;
  $('riskProgress').value = clamp(globalRisk);
  $('stage').textContent = snapshot.stage == null ? '—' : `Stage ${snapshot.stage}`;
  $('confidence').textContent = percent(snapshot.confidence);
  $('dataQuality').textContent = percent(snapshot.data_quality);
  $('asOf').textContent = formatTime(snapshot.as_of);
  $('rawSnapshot').textContent = JSON.stringify(snapshot, null, 2);
  $('lastUpdated').textContent = `화면 갱신 ${formatTime(new Date().toISOString())}`;

  renderCore(snapshot);
  renderRanks('markets', snapshot.markets, 10);
  renderRanks('nodes', snapshot.nodes, 20);
  renderSources(snapshot.sources);
  renderCauses(snapshot.causes);
  renderRules(snapshot);
}

async function refresh() {
  const button = $('refreshButton');
  const errorBanner = $('errorBanner');
  button.disabled = true;
  errorBanner.hidden = true;
  $('connectionText').textContent = '데이터 갱신 중';
  try {
    const response = await fetch('/api/snapshot', { cache: 'no-store' });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    renderDashboard(await response.json());
    $('connectionDot').className = 'status-dot online';
    $('connectionText').textContent = '로컬 연결 정상';
  } catch (error) {
    $('connectionDot').className = 'status-dot offline';
    $('connectionText').textContent = '연결 오류';
    errorBanner.textContent = `데이터를 불러오지 못했습니다: ${error.message}`;
    errorBanner.hidden = false;
  } finally {
    button.disabled = false;
  }
}

$('refreshButton').addEventListener('click', refresh);
refresh();
setInterval(refresh, 60000);
