# ECONOMICS Radar

> v0.4.4: risk heatmap market grouping and explicit KRX EOD publication status.

시장·거시경제 위험을 공식 데이터와 발표 시점 기준으로 평가하는 Rust/SQLite 감시기입니다.

## v0.4.3

v0.4.3은 화면에 오래된 값을 최신 시세처럼 보여 주던 경로와 ALFRED 오류가 현재 데이터 오류처럼 섞이던 문제를 정리한 freshness 중심 릴리스입니다.

- 정규 룰북 `Market_Economy_Radar_Rulebook_v4_ULTRA.txt` 원문을 정규 입력으로 사용
- 룰북 SHA-256 `2f2a3a189c594fdb2a581e6f052123a0dc778e8065677e88d5764f9c813b0b56`
- 85개 규칙군 / 27,494개 규칙 / 중복 ID 0 / 구문 구조 오류 0 검증
- `PRIMARY`, `CONFIRMATION`, `COUNTER_SIGNAL`, `DATA_QUALITY` 채널과 억제 규칙 적용
- stress / vulnerability / resilience 독립 계산
- 동일 날짜의 데이터 개정본 보존 및 `released_at` 기준 as-of 조회
- 결측값은 추정하지 않고 `UNKNOWN`으로 처리
- API 키·URL의 비밀값은 출력하지 않으며 `.env`는 저장소에 포함하지 않음

### 데이터 갱신 주기

기본값은 API 성격과 호출 부담에 맞춰 소스별로 분리됩니다.

- Binance 공개 시장 데이터: **30초**
- 현재 FRED / Treasury / KRX 최신 공개 일별값: **5분**
- ECOS: **30분**
- KRX 전체 이력 / 설정형 공식 어댑터: **6시간**
- ALFRED 빈티지: 자동 실시간 갱신에서 제외하고 **명시적 `collect-alfred` 실행 시에만** 수집

`.env`에서 조정할 수 있습니다.

```env
ECONOMICS_CRYPTO_REFRESH_SECONDS=30
ECONOMICS_REFRESH_MINUTES=5
ECONOMICS_MACRO_REFRESH_MINUTES=30
ECONOMICS_FULL_REFRESH_HOURS=6
```

짧게 설정해도 프로그램 내부 최소값보다 낮아지지 않습니다.

### KRX 데이터의 의미

KRX Open API의 `KOSPI 시리즈 일별시세정보`, `KOSDAQ 시리즈 일별시세정보`는 **일별 데이터**입니다. v0.4.3은 더 이상 임의로 이틀 전까지만 조회하지 않고 **한국시간 기준 오늘 → 직전 영업일 순서로 가장 최근 공개된 행을 먼저 확인**합니다. 다만 KRX Open API가 장중 실시간 지수 틱을 제공하는 것은 아니므로, 장중에는 가장 최근 공개된 EOD 값이 표시될 수 있습니다.

프로그램은 각 숫자에 `LIVE`, `TODAY`, `LATEST EOD`, `STALE Nd` 같은 freshness 상태와 기준일을 표시해 오래된 값을 실시간 시세처럼 보이지 않게 합니다. 진짜 장중 실시간 KOSPI/KOSDAQ이 필요하면 KRX가 허용한 실시간/지연시세 공급계약 또는 승인된 벤더 데이터가 별도로 필요합니다. 이 프로그램은 비공식 웹스크래핑으로 실시간값을 위조하지 않습니다.

### FRED / ALFRED

FRED 현재값은 자동 갱신합니다. ALFRED는 과거 시점 재현용 빈티지 데이터이므로 현재 시세 갱신과 분리했습니다. `SP500`, `DJIA`처럼 빈티지 조회가 불안정한 계열과 중단된 `EQTA`는 자동 ALFRED 폴링 대상에서 제외합니다. FRED 오류가 발생하면 HTTP 상태뿐 아니라 가능한 범위에서 공식 응답의 오류 메시지도 함께 남기며 키는 가립니다.

## Windows 빠른 시작

Release의 `EconomicsRadar-Windows-x64.zip`을 별도 폴더에 풀고 PowerShell에서 실행합니다.

```powershell
Copy-Item .env.example .env
.\EconomicsRadar.exe keys
.\EconomicsRadar.exe rulebook
.\EconomicsRadar.exe launch
```

`.env`에 필요한 키를 넣습니다.

```env
FRED_API_KEY=
ECOS_API_KEY=
KRX_API_KEY=
```

- `FRED_API_KEY`: FRED 수집 및 명시적 ALFRED 수집
- `ECOS_API_KEY`: 한국은행 ECOS
- `KRX_API_KEY`: KRX Open API 인증키. 서비스별 활용승인이 별도로 필요
- `OFFICIAL_ADAPTERS_FILE`: OFR·NY Fed·CFTC·TIC·BIS 등 설정형 공식 JSON 어댑터

실제 키는 GitHub나 Release ZIP에 포함되지 않습니다.

## 주요 명령

```text
launch
keys
rulebook
collect-fred [start] [series]
collect-alfred [start] [series]
collect-public
collect-ecos [series]
collect-krx [api-id]
collect-krx-live
collect-official
collect-all [start]
run [as-of]
backtest <start> <end> [max-points]
serve
demo
```

`collect-krx-live`는 KOSPI·KOSDAQ 등 화면에 직접 쓰이는 KRX 핵심 계열을 오늘부터 역순으로 빠르게 확인합니다. `collect-krx`는 31개 승인 서비스의 일별 이력·파생 지표를 수집합니다.

인자 없이 `EconomicsRadar.exe`를 실행하면 `launch`와 동일하게 로컬 서버를 시작하고 기본 브라우저를 엽니다. 기본 주소는 `http://127.0.0.1:8765`입니다.

엔드포인트는 `/`, `/api/dashboard`, `/api/snapshot`, `/api/refresh-status`, `/api/refresh`, `/health`입니다.

## 대시보드

- `F1 종합`
- `F2 미국`
- `F3 한국`
- `F4 코인`

상단 티커와 세부 표는 각 값의 실제 출처와 기준일을 보여 줍니다. 여러 후보 소스가 있는 지표는 단순히 첫 번째 소스를 고르지 않고 **실제로 더 최신인 관측치**를 선택합니다. 예를 들어 ECOS 원/달러가 오래되고 FRED DEXKOUS가 더 최신이면 최신 FRED 관측치를 사용합니다.

`최신 데이터 수집` 버튼은 전체 갱신을 즉시 요청합니다. 자동 갱신 중 오류가 일부 발생해도 성공한 최신값은 그대로 보존하며, 역사 데이터 오류와 현재값 오류를 가능한 한 분리합니다.

## 백테스트

```powershell
.\EconomicsRadar.exe collect-alfred 2000-01-01
.\EconomicsRadar.exe backtest 2020-01-01 2026-08-20
```

`run 2025-12-31`처럼 날짜 또는 RFC3339 시각을 넘기면 그 시점에 발표된 데이터만 사용합니다. 기본 최대 관측일 수는 5,000개입니다.

## 개발 검증

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo run -- rulebook
```

CI는 Ubuntu와 Windows에서 동일한 검사를 수행합니다. Release workflow는 Windows x64 EXE를 빌드한 뒤 정규 룰북·설정 예제·문서와 함께 ZIP으로 패키징하고, 패키지 내부의 EXE로 룰북 검증을 다시 수행한 후 GitHub Release를 만듭니다.

이 프로젝트는 시장 위험 감시·연구 도구이며 투자수익을 보장하거나 주문을 실행하지 않습니다.
