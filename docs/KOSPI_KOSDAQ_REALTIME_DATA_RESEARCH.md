# KOSPI·KOSDAQ 장중 지수 수신 검토

검토일: 2026-08-24
대상: ECONOMICS Radar 개인용 로컬 대시보드

## 결론

이 앱은 토스증권 Open API를 KOSPI·KOSDAQ 장중 현재지수의 1순위 소스로 사용하고, 기존 KRX OPEN API 일별값을 자동 대체값으로 유지합니다.

토스증권 공식 API는 `KOSPI,KOSDAQ`을 한 요청으로 조회하며 원본 `lastPrice`와 nullable 데이터 `timestamp`를 반환합니다. 운영 응답에서 `timestamp`가 null이면 같은 값의 최신 공식 1분봉 시각만 보완 근거로 사용하며, 값이 다르면 시각을 추정하지 않습니다. 시장 지표 조회는 계좌번호 없이 OAuth 토큰만 필요합니다. 토스 계좌 보유자는 WTS의 Open API 설정에서 클라이언트 키를 발급하고 이 노트북의 공인 IP를 허용 목록에 등록할 수 있습니다. 자세한 명세는 [토스증권 공식 Open API 안내](https://p.tossinvest.com/ko/open-api), [공식 OpenAPI 원문](https://openapi.tossinvest.com/openapi-docs/latest/openapi.json), [공식 연동 안내](https://openapi.tossinvest.com/openapi-docs/overview.md)를 기준으로 했습니다.

## 왜 기존 KRX 키만으로는 안 되는가

현재 KRX OPEN API 서비스 목록은 KOSPI·KOSDAQ 지수를 **일별시세정보**로 명시합니다. KRX의 직접 실시간 피드는 별도 데이터 분배 상품이며, 전문 이용은 KRX 승인과 코스콤 계약 절차가 필요합니다. 따라서 기존 KRX 키는 이력과 공식 EOD 확인에 계속 사용하되 장중 실시간값으로 표시하지 않습니다. [KRX OPEN API 서비스 목록](https://openapi.krx.co.kr/contents/OPP/INFO/service/OPPINFO004.cmd), [KRX 데이터 수신방법](https://openapi.krx.co.kr/contents/OPP/DATA/OPPDATA003.jsp)

## 앱의 데이터 선택 규칙

1. 토스 키 두 개가 설정되면 30초 주기로 KOSPI·KOSDAQ 현재지수를 함께 조회합니다.
2. 토스의 원본 데이터 시각이 KRX EOD보다 실제로 최신일 때만 화면을 덮어씁니다.
3. 같은 날짜의 토스 장중값과 KRX EOD는 두 개의 일간 관측치로 중복 계산하지 않습니다.
4. 장중 수신이 멈추면 `DELAYED` 또는 `STALE`, 장 마감값이면 `SESSION CLOSE`, 다음 장 시작 전 최근 종가면 `LATEST CLOSE`로 표시합니다.
5. 토스 키 미설정·허용 IP 오류·일시 장애 때는 최신 검증 KRX EOD를 유지하며 값을 추정하지 않습니다.

## 다른 공식 대안

한국투자증권도 공식 `국내업종 현재지수` API에서 KOSPI `0001`, KOSDAQ `1001`을 제공합니다. 다만 이 사용자는 이미 토스증권 계좌를 보유하고 있고 토스가 지수 전용 심볼과 원본 타임스탬프를 직접 제공하므로 토스를 우선 선택했습니다. [한국투자증권 공식 예제](https://github.com/koreainvestment/open-trading-api/blob/main/examples_llm/domestic_stock/inquire_index_price/inquire_index_price.py)

## 사용 범위

이 구현은 개인 로컬 화면의 참고용입니다. 외부 재배포나 상업적 시세 제공은 별도 데이터 이용 권한을 확인해야 합니다.
