# ECONOMICS — Market & Economy Radar

AI API 비용 없이 동작하는 **규칙 기반 미국·글로벌·한국·코인 금융위험 레이더**입니다. FRED/ALFRED를 중심으로 미국 재무부, OFR, 뉴욕연은, CFTC, BIS, 한국은행 ECOS, KRX, Binance 공식 데이터원을 결합하도록 설계했습니다.

현재 저장소에는 대화에서 설계한 **Market & Economy Radar v4 ULTRA 룰북 27,494개**와 이를 실제로 읽고 평가하는 Python 엔진이 함께 들어 있습니다.

## 무엇을 보여주나

```text
GLOBAL RISK      72/100 🟠
Stress           68/100
Vulnerability    84/100 🔴
Resilience       43/100
Confidence       91/100
Diffusion        11/18
Crisis Stage     3/6 — 전염 진행

US Equity        70
Crypto           82
Korea Equity     67

주요 원인/전염경로와 사전작성된 한국어 설명 메시지
```

위 숫자는 폭락 확률이 아니라 **현재 스트레스와 구조적 취약성의 위험 점수**입니다.

## 구현 상태

- v4 ULTRA **27,494개 룰 파서/무결성 검사**
- 안전한 rule DSL evaluator (`AND`, 비교, `BETWEEN`, score/module/horizon 함수, 시장 조합 등)
- 18개 핵심 risk dimension
- Stress / Vulnerability / Resilience 분리
- 4개 시간축 proxy + 위기 Stage 0~6
- 확산도(diffusion), 시장별 US/CRYPTO/KOREA score
- FRED baseline 자동수집/SQLite 저장/percentile score
- ALFRED vintage 호출 지원
- Treasury FiscalData auction/buyback 수집
- CFTC TFF Treasury leveraged-fund public proxy
- Binance OI/funding/positioning/taker metric 수집
- ECOS / KRX / OFR / NY Fed / BIS 클라이언트
- direct official metric이 없을 때는 낮은 confidence의 proxy module 사용, direct data가 들어오면 자동 대체
- SQLite WAL persistence
- 비밀키가 절대 출력되지 않는 credential status
- dependency-free local dashboard + JSON snapshot endpoint
- offline unit tests + GitHub Actions CI

## 설치

Python 3.11+:

```bash
python -m venv .venv
# Linux/macOS
source .venv/bin/activate
# Windows PowerShell
# .\.venv\Scripts\Activate.ps1

python -m pip install -e .
cp .env.example .env
```

Windows에서는 `.env.example`을 `.env`로 복사해도 됩니다.

## API 키

`.env`에는 **앞뒤 공백 없이** 넣습니다.

```env
FRED_API_KEY=
ECOS_API_KEY=
KRX_API_KEY=
BINANCE_API_KEY=
```

`.env`는 `.gitignore`되어 있습니다. 실제 API Key/Secret은 GitHub에 커밋하지 마세요.

키 존재 여부만 확인:

```bash
economics-radar keys
```

실제 키 값은 출력하지 않습니다.

## 1. 룰북 검증

```bash
economics-radar rulebook
```

기대 결과:

```json
{
  "rules": 27494,
  "exact_duplicates": 0,
  "logical_conflicts": 0
}
```

## 2. FRED 수집

```bash
economics-radar collect-fred --start 2000-01-01
```

FRED baseline은 WEI, CFNAI, Sahm Rule, claims, inflation, rates, Treasury curve, HY spreads, SLOOS, NFCI/STLFSI, leverage, VIX, dollar, housing, consumer, liquidity, Korea/China CLI 등으로 구성됩니다.

## 3. ALFRED 초기발표값 수집 / 과거 재생

```bash
economics-radar collect-alfred --start 2000-01-01
```

초기 발표값을 별도 `alfred` source/vintage로 보관합니다. 이후 특정 기간을 당시 공개정보 기준으로 재생할 수 있습니다.

```bash
economics-radar backtest --start 2007-01-01 --end 2010-12-31 --step-days 7 --csv runtime/gfc_replay.csv
```

ALFRED 데이터가 충분하지 않은 실험에서만 `--allow-revised-fallback`을 사용하세요. 실전 calibration에는 revised fallback을 권장하지 않습니다.

## 4. 공식 시장 미시구조 데이터 수집

```bash
economics-radar collect-official --start 2000-01-01
```

현재 자동 연결된 그룹:

- **U.S. Treasury FiscalData**: auctions / buybacks
- **CFTC TFF**: Treasury futures leveraged-fund proxy
- **Binance USD-M Futures**: BTC OI / funding / positioning / taker imbalance

하나를 제외하려면 `--skip-binance`, `--skip-cftc`, `--skip-treasury`를 사용합니다.

## 5. 위험 레이더 실행

```bash
economics-radar run
```

SQLite 기본 위치는 `runtime/economics.db`입니다.

데이터 없이 엔진/룰 동작만 확인하려면:

```bash
economics-radar demo
```

## 6. 대시보드

```bash
economics-radar serve
```

브라우저에서:

```text
http://127.0.0.1:8765
```

JSON:

```text
http://127.0.0.1:8765/api/snapshot
```

## 설계 원칙

### Risk ≠ Confidence

데이터가 누락되었다고 안전한 것이 아닙니다. 위험점수와 데이터 신뢰도를 별도로 표시합니다.

### Stress ≠ Vulnerability

- **Stress**: 지금 실제 시장이 흔들리는가
- **Vulnerability**: 아직 조용해도 레버리지·부채·펀딩 구조가 취약한가
- **Resilience**: 충격을 흡수할 완충력이 있는가

따라서 `낮은 Stress + 높은 Vulnerability`는 **조용한 취약성**으로 별도 분류할 수 있습니다.

### 같은 신호를 여러 번 세지 않는다

NFCI/STLFSI/OFR/credit spread처럼 같은 금융스트레스를 측정하는 데이터는 이후 source-correlation calibration에서 중복가중치를 줄이고, 독립 소스의 교차 확인은 **risk 점수 폭증보다 confidence 상승**에 우선 사용합니다.

### ALFRED / event-time 백테스트

과거 테스트는 현재 수정된 데이터만 사용하지 않고 당시 공개되어 있던 vintage와 release lag를 사용해야 합니다. FRED client는 vintage 호출을 지원하며 DB는 vintage를 별도 저장합니다.

## 저장소 구조

```text
rulebooks/               v4 ULTRA 원본 룰북
src/economics_radar/
  engine.py              전체 orchestration
  dsl.py                 rule condition evaluator
  scoring.py             percentile / dimension scoring
  official.py            Treasury/CFTC/Binance direct metrics
  db.py                  SQLite
  dashboard.py           local UI
  sources/               공식 데이터원 adapters
docs/                    architecture / data source notes
tests/                   offline regression tests
```

## 중요한 제한

이 프로젝트는 금융시장 **리스크 감시/연구 도구**이며 투자수익을 보장하거나 매수·매도 신호를 제공하는 시스템이 아닙니다. v4의 많은 임계값과 가중치는 설계 초안이므로 실제 운영 전에 ALFRED/event-time walk-forward 백테스트로 false positive, lead time, persistence를 보정해야 합니다.

또한 KRX는 인증키와 별개로 사용할 API 서비스별 이용신청/승인이 필요할 수 있습니다. 승인되지 않은 KRX 서비스 ID를 코드에 임의로 하드코딩하지 않았습니다.
