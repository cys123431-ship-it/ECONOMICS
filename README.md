# ECONOMICS Radar

시장·거시경제 위험을 데이터 발표 시점 기준으로 평가하는 Rust/SQLite 감시기입니다. v0.3.3은 `Market_Economy_Radar_Rulebook_v4_ULTRA.txt` 원문을 정규 입력으로 사용하며, 원문의 규칙 ID·조건·우선순위·억제자·제목·메시지를 그대로 파싱합니다.

## v0.3.3에서 보장하는 것

- 정규 룰북 SHA-256 `2f2a3a189c594fdb2a581e6f052123a0dc778e8065677e88d5764f9c813b0b56`
- 85개 규칙군, 27,494개 규칙, 중복 ID 0개, 구문 구조 오류 0개를 시작 시 검증
- 조건이 계산되지 않으면 `UNKNOWN`으로 처리하여 경보를 만들지 않는 3값 논리
- `PRIMARY`, `CONFIRMATION`, `COUNTER_SIGNAL`, `DATA_QUALITY` 채널과 우선순위·구체성·억제 규칙 적용
- stress / vulnerability / resilience를 독립 계산하고, 표본 또는 신뢰도가 부족하면 `null` 반환
- 동일 날짜의 데이터 개정본을 보존하고 `released_at` 기준 as-of 조회로 미래정보 유입 방지
- 중복 계열은 같은 redundancy group 안에서 먼저 합산하여 위험을 이중 계상하지 않음
- FRED 현재치와 빈티지 날짜를 분할 조회하는 ALFRED 최초발표치, Treasury FiscalData, BOK ECOS, 승인된 KRX JSON API, Binance Futures 공개 API 및 설정형 공식 JSON 어댑터
- API 키·URL의 비밀값은 출력하지 않고 수집 오류는 비밀값을 가린 뒤 비정상 종료

상세 구현 추적표는 [docs/IMPLEMENTATION_AUDIT.md](docs/IMPLEMENTATION_AUDIT.md)에 있습니다.

## Windows 빠른 시작

Release의 `EconomicsRadar-Windows-x64.zip`을 별도 폴더에 풀고 PowerShell에서 실행합니다.

```powershell
Copy-Item .env.example .env
.\EconomicsRadar.exe keys
.\EconomicsRadar.exe rulebook
.\EconomicsRadar.exe collect-official
.\EconomicsRadar.exe run
.\EconomicsRadar.exe serve
```

`collect-official`의 Treasury·Binance 수집에는 키가 필요하지 않습니다. 아래 입력은 해당 소스를 사용할 때만 필요합니다.

- `FRED_API_KEY`: FRED/ALFRED 수집
- `ECOS_API_KEY`: 한국은행 ECOS 수집
- `KRX_API_KEY`: KRX 인증키. 사용할 KRX 서비스는 별도 활용승인이 필요하며 URL은 프로그램에 내장됩니다.
- `OFFICIAL_ADAPTERS_FILE`: OFR·NY Fed·CFTC·TIC·BIS 등 공식 JSON 응답의 필드 매핑 파일

KRX는 인증키 발급과 서비스 활용승인이 별도입니다. 프로그램은 사용자별 URL을 요구하지 않고 아래 공식 서비스 URL을 내장합니다. KRX 사이트에서 실제로 사용할 서비스의 활용승인을 받아야 하며, 승인되지 않은 서비스는 이름과 함께 HTTP 401 오류로 보고됩니다.

- KOSPI 시리즈 일별시세정보
- KOSDAQ 시리즈 일별시세정보
- 유가증권 일별매매정보
- 코스닥 일별매매정보
- 선물 일별매매정보 (주식선물外)
- 옵션 일별매매정보 (주식옵션外)

키를 입력한 뒤 전체 수집은 다음과 같습니다.

```powershell
.\EconomicsRadar.exe collect-fred 2000-01-01
.\EconomicsRadar.exe collect-alfred 2000-01-01
.\EconomicsRadar.exe collect-public
.\EconomicsRadar.exe collect-ecos
.\EconomicsRadar.exe collect-krx
.\EconomicsRadar.exe collect-all 2000-01-01
```

## 명령

```text
keys
rulebook
collect-fred [start] [series]
collect-alfred [start] [series]
collect-public
collect-ecos [series]
collect-krx
collect-official
collect-all [start]
run [as-of]
backtest <start> <end> [max-points]
serve
demo
```

`run 2025-12-31`처럼 날짜 또는 RFC3339 시각을 넘기면 그 시점에 발표된 데이터만 사용합니다. `backtest`도 같은 기준을 사용하며, 실수로 과도한 실행을 하지 않도록 기본 최대 관측일 수를 5,000개로 제한합니다.

FRED/ALFRED의 마지막 `series`는 선택사항입니다. 예를 들어 `collect-alfred 2000-01-01 ANFCI`는 해당 계열만 재수집합니다.

ECOS도 `collect-ecos KR_USD_KRW`처럼 마지막 `series`를 지정해 한 계열만 재수집할 수 있습니다.

서버 기본 주소는 `http://127.0.0.1:8765`입니다. 엔드포인트는 `/`, `/api/snapshot`, `/health`이며 로컬 바인딩이 기본값입니다.

## 데이터 결과 해석

- `global_risk`: 신뢰도 35 미만이면 `null`
- `stress`, `vulnerability`, `resilience`: 충분한 독립 표본이 있는 축만 숫자
- `confidence`: 전체 설계 가중치 중 실제 계산 가능한 비율과 신선도
- `data_quality`: 추적 중인 공식 소스 중 신선한 소스 비율
- `rules_indeterminate`: 결측 때문에 참/거짓을 확정하지 못한 규칙 수
- `rule_hits`: 원문 규칙의 ID·조건·메시지를 보존한 발동 결과

## 개발 검증

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo run -- rulebook
```

CI는 Ubuntu와 Windows에서 같은 검사를 수행합니다. 태그 워크플로는 테스트 후 EXE, 정규 룰북, 설정 예제, 문서와 라이선스를 ZIP에 넣어 GitHub Release를 생성합니다.

이 프로젝트는 시장 위험 감시·연구 도구이며 투자수익을 보장하거나 주문을 실행하지 않습니다.
