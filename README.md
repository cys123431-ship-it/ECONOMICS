# ECONOMICS Radar

> v0.6.0: account-free Toss market intelligence across Korea and the U.S.

시장·거시경제 위험을 공식 데이터와 발표 시점 기준으로 평가하는 Rust/SQLite 감시기입니다.

## v0.6.0

v0.6.0은 토스증권 Open API에서 계좌번호 없이 조회할 수 있는 시장정보를 단순 지수 현재가에서 한국·미국 시장 인텔리전스로 확장합니다. 개인 계좌·보유자산·주문 API는 호출하지 않습니다.

- KOSPI·KOSDAQ과 한국 국채 2·3·5·10·20·30년 수익률을 30초 주기로 한 번에 조회
- USD/KRW 1분 참고 매매기준율·매수환율·스프레드를 원본 유효시각과 함께 표시
- KOSPI·KOSDAQ 30일 일봉 OHLCV와 개인·외국인 전체·기관·연기금 순매수 시계열 추가
- 수급 전용 표에서 개인·외국인·기관 7개 세부 분류·기타법인의 원본 매수액·매도액·순매수액을 모두 표시
- 한국 KRX+NXT와 미국 데이·프리·정규·애프터마켓 운영시간 및 현재 세션 판정
- 한국·미국 시장별 실시간 거래대금, 1일 급상승, 1일 급하락 TOP 10과 현재가·등락률·거래량·거래대금 표시
- 랭킹 종목코드를 종목 기본정보 API로 다시 조회해 한글 종목명·상장시장·상품유형까지 표시
- 종합 탭에 양 시장의 장상태와 거래대금·상승·하락 1위 요약 추가
- 화면 지표를 113개에서 137개로 확대하고, 장상태·랭킹·상세 수급은 별도 고밀도 패널로 분리
- OAuth 토큰 재사용, 401 1회 재발급, 429 `Retry-After` 1회 재시도, 랭킹 그룹 호출속도 제한 준수
- 토스 키 미설정·허용 IP·일시 장애 시 기존 KRX·ECOS·FRED 값과 마지막 정상 시장 스냅샷 유지

구현 범위와 공식 API 근거는 [토스증권 시장정보 확장](docs/TOSS_OPEN_API_MARKET_EXPANSION.md)에 정리했습니다.

## v0.5.1

v0.5.1은 토스증권 공식 Open API의 KOSPI·KOSDAQ 현재지수를 선택형 장중 소스로 추가하고, 기존 KRX 일별값은 이력·EOD 자동 대체값으로 유지합니다.

- 토스증권 `KOSPI,KOSDAQ` 현재지수를 한 요청으로 30초마다 갱신
- 토스 원본 `lastPrice`와 RFC3339 데이터 시각을 그대로 저장·표시
- 토스 시각이 KRX EOD보다 실제로 최신일 때만 화면에 적용
- 같은 날짜의 토스 장중값과 KRX EOD를 중복 일간 관측치로 계산하지 않고 전일 KRX 종가와 비교
- `LIVE`, `DELAYED`, `SESSION CLOSE`, `LATEST CLOSE`, `STALE` 상태 구분
- OAuth 토큰 24시간 캐시, 401 한 차례 재발급, 허용 IP·요청 제한 오류의 비밀값 비노출
- 날짜 문자열과 RFC3339 시각의 최신성 비교를 실제 UTC 시간 비교로 수정
- 토스 키가 없거나 수집에 실패하면 최신 검증 KRX EOD를 계속 표시

설계 근거와 공식 출처는 [KOSPI·KOSDAQ 장중 지수 수신 검토](docs/KOSPI_KOSDAQ_REALTIME_DATA_RESEARCH.md)에 정리했습니다.

## v0.5.0

v0.5.0은 종합·미국·한국·코인 탭을 원본값 중심의 고밀도 터미널로 확장하고, “위기 탈출 목표 접근도”를 원본 위험 구성요소와 함께 보여 줍니다.

- 화면 지표 31개에서 113개로 확대: 미국 39개, 한국 56개, 코인 19개 행(공통 항목 포함)
- 모든 상세 행에 표시값, `RAW` 원본 숫자, 출처·시계열 ID, 관측시각, 발표주기, 신선도, 최근 범위·위치 표시
- 미국 주식·금리·신용·금융여건·고용·은행·연준 유동성·국채입찰·딜러 시장 패널 추가
- KRX 승인 데이터의 주식 시장규모·시장폭·선물·옵션·채권·ETF·ETN·ELW·금·석유·배출권·ESG·SRI·코넥스 패널 추가
- 중복합산 KRX 지수총액, 의미 없는 개별주식선물 평균 베이시스, 갱신 중단 옵션 P/C는 화면에서 제외
- Binance 현물 시간봉·24시간 고가/저가/거래량/거래대금과 무기한선물 mark/index/funding 원본 수집
- 기존 `BTC_PRICE_USD`가 Binance 무기한선물 가격이었다는 의미를 바로잡고 현물 `BTC_SPOT_PRICE_USD`를 분리
- BTC OI 24시간 변화 백분위, 가격 모멘텀·신고가, 펀딩·베이시스·상위포지션·테이커 z-score를 규칙 엔진 문맥에 구현
- 종합·미국·한국·코인의 탈출 조건을 `≤35 통과`, `≥55 차단`, `≥75 핵심차단`, 결측으로 구분
- 단일 시점은 “탈출 확정”으로 과장하지 않고, 저장된 7일 전(코인은 24시간 전) 스냅샷이 있을 때만 개선·악화 추세 표시
- 금리·수익률·비율의 변화는 잘못된 퍼센트 변화 대신 `pp`/포인트로 표시
- 수집 오류의 전체 URL을 화면에 노출하지 않고 짧은 소스·HTTP 상태와 직전값 유지 여부만 표시

## v0.4.7

v0.4.7은 같은 UTC 시각의 `Z`·소수초·`+00:00` 표기를 문자열이 아닌 실제 시간값으로 비교합니다. 최신 KRX 파생값이 같은 초에 생성된 스냅샷에서 이전 개정값으로 보일 수 있던 경계를 수정하고 회귀 테스트를 추가했습니다.

## v0.4.6

v0.4.6은 오래된 KOSPI가 최신값처럼 보이던 문제를 포함해 수집·시점·파생계산 경로를 전수 정비한 데이터 무결성 릴리스입니다.

- 시작 시 KRX 최신 영업일을 우선 확인한 뒤 같은 시점의 스냅샷으로 화면 구성
- KRX 조회 종료일을 한국시간 오늘로 수정하고, 공식 API가 확인한 최신 EOD 날짜를 별도 기록
- 코스피200 선물에서 야간·스프레드를 제외하고 정규장 단순선물만 사용
- 선물 베이시스는 정규장 최근월물, 미결제약정은 정규장 단순선물 전체 월물 합계로 계산
- 옵션 풋/콜과 내재변동성은 정규장 최근월물만 사용하며 IV는 거래량 가중
- KOSPI/KOSDAQ 시장폭은 단순 평균 대신 실제 종목 수로 가중
- Binance 현재가는 덮어쓰기형 live quote로 분리하고, 이력은 종료된 시간봉만 저장
- 관측시각·발표시각·수집시각을 모두 만족하는 as-of 조회와 동일 발표시각 개정본 선택 수정
- 결측률·신선도·사용 가능 비율에 따라 신뢰도를 계산하며 인위적인 최저점 제거
- OFR FSI, NY Fed 프라이머리딜러 결제실패, Fed SCOOS, BIS 글로벌 달러신용 내장 공식 수집기 추가
- 소스별 예상/가용/결측/지연 계열을 화면에서 구분

## v0.4.5

v0.4.5는 종합·미국·한국·코인 탭 전체 렌더링 경로를 다시 점검한 안정화 릴리스입니다.

- 제거된 `NODE_LABELS` 잔존 참조를 `NODE_META` 기반 라벨로 교체
- 한 탭의 JavaScript 예외가 다른 탭까지 중단시키지 않도록 렌더링 격리
- 미국·한국·코인별 위험요인 이름을 시장 문맥에 맞게 명확화
- 미국 탭에 기업부채·은행 스트레스·주택시장 요인 추가
- API 통신 오류, 화면 렌더링 오류, 데이터 수집 오류를 분리 표시
- 종합·미국·한국·코인 탭을 실제로 렌더링하는 Node smoke test를 Ubuntu/Windows CI와 Windows Release 빌드에 추가
- Windows Release 파일명을 `EconomicsRadar-v0.4.5-Windows-x64.zip` 형식으로 버전 명시

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

- Binance 공개 시장 데이터와 토스증권 지수·국채·USD/KRW: **30초**
- 토스증권 장 캘린더·지수 일봉·투자자 수급·한국/미국 랭킹: **5분**
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

토스증권 Open API 키가 설정되면 프로그램은 공식 `KOSPI,KOSDAQ` 현재지수를 장중 30초마다 조회합니다. 토스 키가 없거나 실패하면 KRX 최신 검증 EOD로 자동 대체합니다. 각 숫자에는 `LIVE`, `DELAYED`, `SESSION CLOSE`, `LATEST CLOSE`, `LATEST VERIFIED`, `STALE` 같은 상태와 원본 기준시각을 표시하며 비공식 웹스크래핑으로 실시간값을 만들지 않습니다.

### FRED / ALFRED

FRED 현재값은 자동 갱신합니다. ALFRED는 과거 시점 재현용 빈티지 데이터이므로 현재 시세 갱신과 분리했습니다. `SP500`, `DJIA`처럼 빈티지 조회가 불안정한 계열과 중단된 `EQTA`는 자동 ALFRED 폴링 대상에서 제외합니다. FRED 오류가 발생하면 HTTP 상태뿐 아니라 가능한 범위에서 공식 응답의 오류 메시지도 함께 남기며 키는 가립니다.

## Windows 빠른 시작

Release의 `EconomicsRadar-v0.6.0-Windows-x64.zip`을 별도 폴더에 풀고 PowerShell에서 실행합니다.

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
TOSSINVEST_CLIENT_ID=
TOSSINVEST_CLIENT_SECRET=
```

- `FRED_API_KEY`: FRED 수집 및 명시적 ALFRED 수집
- `ECOS_API_KEY`: 한국은행 ECOS
- `KRX_API_KEY`: KRX Open API 인증키. 서비스별 활용승인이 별도로 필요
- `TOSSINVEST_CLIENT_ID`, `TOSSINVEST_CLIENT_SECRET`: 토스증권 WTS > 설정 > Open API에서 발급. 지수 조회에는 계좌번호가 필요 없지만 이 노트북의 공인 IP를 허용 목록에 등록해야 함
- `OFFICIAL_ADAPTERS_FILE`: 내장되지 않은 CFTC·TIC·FSC 등 공식 JSON 피드를 추가할 때만 쓰는 선택 설정

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
collect-toss-indices
collect-toss-market
collect-official
collect-all [start]
run [as-of]
backtest <start> <end> [max-points]
serve
demo
```

`collect-krx-live`는 KOSPI·KOSDAQ 등 화면에 직접 쓰이는 KRX 핵심 계열을 오늘부터 역순으로 빠르게 확인합니다. `collect-krx`는 31개 승인 서비스의 일별 이력·파생 지표를 수집합니다.

`collect-toss-indices`는 하위호환 명령으로 토스증권 공식 시장지표·환율 현재값을 조회합니다. `collect-toss-market`은 여기에 한국·미국 장 캘린더, 지수 일봉, 투자자 수급, 양 시장 랭킹과 종목명까지 모두 수집합니다. 두 토스 키가 설정되지 않으면 네트워크 호출을 하지 않습니다.

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
