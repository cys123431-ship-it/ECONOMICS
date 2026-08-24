# KOSPI·KOSDAQ 장중 참고 시세 검토

검토일: 2026-08-24
대상: ECONOMICS Radar 개인용 로컬 대시보드

## 결론

앱의 공식 원본값과 위기 계산은 기존처럼 KRX Open API의 KOSPI·KOSDAQ 일별시세정보를 사용한다. 장중 가격을 빠르게 확인하는 용도로만 한국 탭에 TradingView Single Ticker 위젯 두 개를 별도로 표시한다.

- 코스피 위젯 심볼: `KRX:KOSPI`
- 코스닥 위젯 심볼: `KRX:KOSDAQ`
- 위젯 값은 SQLite에 저장하지 않고 규칙 엔진·위기 점수·백테스트에 전달하지 않는다.
- 카드나 링크를 누르면 TradingView의 해당 지수 전체 페이지를 새 탭으로 연다.
- 증권사 OAuth, API 키, 허용 IP, 고정 공인 IP는 필요하지 않다.

TradingView 공식 문서는 Single Ticker를 한 심볼의 가격과 등락률을 보여 주는 위젯으로 정의하며, 위젯 자체 데이터가 포함되어 별도 API 연결이 필요하지 않다고 설명한다. [Single Ticker 문서](https://www.tradingview.com/widget-docs/widgets/tickers/single-ticker/), [위젯 시작 안내](https://www.tradingview.com/widget-docs/getting-started/)

## KRX와 TradingView의 역할 분리

KRX Open API의 KOSPI·KOSDAQ 서비스는 일별시세정보다. 앱은 한국시간 오늘부터 역순으로 최신 공개 행을 확인하고 `LATEST VERIFIED`, `PUBLISHED EOD`, `STALE` 상태와 기준일을 명시한다. 이 값이 앱의 공식 데이터 계보와 위기 계산에 사용된다. [KRX Open API 서비스 목록](https://openapi.krx.co.kr/contents/OPP/INFO/service/OPPINFO004.cmd)

TradingView 위젯은 브라우저가 TradingView 서버에 직접 연결해 `KRX:KOSPI`, `KRX:KOSDAQ`을 표시한다. 거래소 및 제공자 정책에 따라 지연될 수 있으므로 화면에 “외부 장중 참고 시세”와 “위기 점수 미사용”을 함께 표기한다. [KOSPI 심볼](https://www.tradingview.com/symbols/KRX-KOSPI/), [KOSDAQ 심볼](https://www.tradingview.com/symbols/KRX-KOSDAQ/)

## 보안과 장애 경계

Content Security Policy는 앱 자체 리소스 외에 TradingView 위젯 모듈, 이미지·글꼴, HTTPS·WebSocket 시세 연결, 위젯 데이터 프레임만 허용한다. 위젯이 Shadow DOM에 자체 스타일을 적용하므로 `style-src`의 인라인 스타일도 허용하지만 외부 스크립트와 프레임은 TradingView 위젯 호스트로 제한한다. TradingView 스크립트가 차단되거나 인터넷이 끊기면 카드 안의 직접 링크가 남으며, KRX·FRED·ECOS 수집과 위기 계산은 영향을 받지 않는다.

TradingView는 위젯이 쿠키를 설정하지 않지만 정상 동작을 위해 임베드 페이지 URL, 위젯 유형, 표시 심볼, IP 주소를 처리한다고 안내한다. [TradingView 위젯 일반 FAQ](https://www.tradingview.com/widget-docs/faq/general/)

## 사용 범위

외부 위젯은 개인 로컬 화면의 참고용이다. 외부 재배포나 상업적 시세 제공은 각 데이터 제공자의 이용 조건과 거래소 권한을 별도로 확인해야 한다.
