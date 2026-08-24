# KOSPI·KOSDAQ 장중 참고 시세 검토

검토일: 2026-08-24
대상: ECONOMICS Radar 개인용 로컬 대시보드

## 결론

TradingView의 KRX 지수 위젯은 사용하지 않는다. TradingView 공식 FAQ는 “This symbol is only available on TradingView”가 해당 심볼을 어떤 시간 범위에서도 외부 위젯에 표시할 권한이 없다는 의미라고 설명한다. 유료 TradingView 요금제로도 웹사이트 위젯의 실시간 데이터 권한은 생기지 않는다. [TradingView 데이터 FAQ](https://www.tradingview.com/widget-docs/faq/data/)

대신 앱 서버가 Npay 증권의 공개 KOSPI·KOSDAQ 조회 응답을 30초마다 확인해 다음 항목을 자체 카드에 표시한다.

- 지수 원본값
- 전일 대비 등락폭과 등락률
- 장중·장마감 상태
- 제공 지연 표기와 기준시각
- Npay 증권 상세 차트 및 KRX 정보데이터시스템 링크

별도 API 키, 증권사 계좌, 허용 IP 또는 고정 공인 IP는 필요하지 않다.

## 공식 계산값과 외부 참고값의 분리

KRX Open API의 KOSPI·KOSDAQ 서비스는 일별시세정보다. 앱은 한국시간 오늘부터 역순으로 최신 공개 행을 확인하고 `LATEST VERIFIED`, `PUBLISHED EOD`, `STALE` 상태와 기준일을 명시한다. 이 KRX 값만 앱의 공식 데이터 계보, SQLite 이력, 룰 엔진, 위기 점수와 백테스트에 사용한다. [KRX Open API 서비스 목록](https://openapi.krx.co.kr/contents/OPP/INFO/service/OPPINFO004.cmd)

Npay 증권 페이지는 지수를 실시간으로 제공한다고 표시하며 국내 증시 기본 데이터의 출처를 KRX로 밝힌다. 앱의 장중 카드는 이 공개 조회값을 화면 참고용으로만 사용하고 SQLite에 저장하지 않는다. [Npay 증권 KOSPI](https://finance.naver.com/sise/sise_index.naver?code=KOSPI), [Npay 증권 KOSDAQ](https://finance.naver.com/sise/sise_index.naver?code=KOSDAQ)

외부 조회가 실패하거나 값의 숫자 형식이 유효하지 않으면 값을 추정하거나 0으로 채우지 않는다. 해당 카드에는 DB에 보존된 KRX 최신 공식 종가와 기준일을 대신 표시한다.

## 보안과 장애 경계

브라우저는 `/api/kr-indices-live`라는 로컬 same-origin 엔드포인트만 호출한다. 외부 조회는 Rust 로컬 서버가 제한된 시간 안에 수행하므로 Npay 도메인을 브라우저 Content Security Policy에 추가하지 않는다. TradingView 외부 스크립트·WebSocket·프레임 허용도 모두 제거한다.

외부 참고값 오류는 FRED·ECOS·KRX 수집과 위기 계산을 중단시키지 않는다. 응답에는 `used_in_risk_engine: false`를 명시해 계산 경계를 기계적으로도 드러낸다.

## 사용 범위

이 기능은 사용자 개인 노트북의 로컬 참고 화면을 위한 것이다. Npay 증권은 시세의 오류·지연 가능성과 무단 배포 제한을 고지하므로 공개 웹서비스나 상업적 재배포로 전환할 경우 데이터 제공자 및 거래소와 별도 계약·권한을 확인해야 한다.
