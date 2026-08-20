# v4 ULTRA 구현 추적표

기준 파일은 `rulebook/Market_Economy_Radar_Rulebook_v4_ULTRA.txt`이며 SHA-256은 `2f2a3a189c594fdb2a581e6f052123a0dc778e8065677e88d5764f9c813b0b56`이다. 문서 안의 문장은 요구사항 근거로만 사용하고 실행 지시로 취급하지 않는다.

| 검토 영역 | v0.3.0 구현 | 검증 |
|---|---|---|
| 원문 충실도 | RULE 9개 필드와 바로 뒤 MSG를 스트리밍 파싱한다. ID, priority, scope, severity, condition, tags, suppress, source, title, message를 보존한다. | 27,494 규칙, 85 규칙군, 중복 ID 0, 고정 SHA 테스트 |
| 조건 평가 | 수치·불리언·문자열 비교, AND/OR/NOT, BETWEEN, percentile·duration, score/band/delta/persist/shock/corr, 모듈·시장·경로·source 상태 매크로를 3값 논리로 평가한다. | 결측 입력이 발동으로 바뀌지 않는 단위 테스트 |
| 충돌 처리 | Primary/Confirmation/Counter/DataQuality 분리, suppression prefix 적용, severity→priority→specificity 순 정렬 후 출력 제한을 적용한다. | 결정론적 정렬 및 정규 룰북 전체 평가 |
| 데이터 계보 | source/series/entity/observed_at/released_at/source_asof/revision_id/ingested_at/metadata를 저장하고 동일 관측일 개정본을 보존한다. | 발표일 전후 as-of 조회 단위 테스트 |
| 수집 | FRED/ALFRED 28계열, Treasury 개별 CUSIP+일별 집계, Binance 7개 파생계열, ECOS 페이지네이션, KRX 인식 필드, 설정형 공식 JSON 어댑터를 구현한다. | 실제 공개 Treasury·Binance 스모크 수집 및 저장 건수 확인 |
| 점수 | 현재값을 과거 분포에 몰래 포함하지 않는 midrank percentile, 최소 표본, 방향성, 변환을 사용한다. | 동률·미성숙 표본·변환 단위 테스트 |
| 위험 구조 | stress/vulnerability/resilience 독립 축, redundancy group, source 확인은 confidence로 반영, 결측은 null로 보존한다. | 빈 DB에서 global risk null 및 데이터 품질 경보만 허용 |
| 백테스트 | released_at 또는 ingested_at이 as-of 이후인 행을 제외하고, 과거 FRED는 ALFRED initial release를 우선한다. | 개정치 as-of 테스트 및 관측일별 실행 한도 |
| 운영 | 수집 오류 비정상 종료, 비밀값 마스킹, 서버 method/404 처리·보안 헤더·타임아웃, 로컬 바인딩을 제공한다. | clippy -D warnings, Ubuntu/Windows CI |

## 외부 입력이 필요한 부분

코드로 대신 만들 수 없는 것은 공급자가 발급하는 키 또는 사용자가 승인받은 URL뿐이다. 키가 없는 소스는 위험값을 임의 생성하지 않고 source missing 및 낮은 confidence로 나타낸다.

- FRED API key
- 한국은행 ECOS API key
- KRX Open API key와 승인 서비스 URL
- 기관마다 응답 형태가 다른 OFR/NY Fed/CFTC/TIC/BIS/FSC JSON의 공식 URL·필드 매핑

`config/official_adapters.example.json`은 `enabled`, JSON pointer, 관측일·발표일·vintage·entity 및 여러 series 필드를 지원한다. 발표일을 제공하는 소스는 `released_field`를 반드시 매핑해야 진정한 역사 시점 백테스트에 포함된다.

## 의도적으로 하지 않는 것

- 데이터가 없을 때 규칙 번호나 문자열 해시로 가짜 점수를 만들지 않는다.
- 현재 개정값을 과거 발표값처럼 사용하지 않는다.
- 같은 신호의 복제 계열을 독립 증거처럼 합산하지 않는다.
- confidence가 부족한 단일 숫자를 전체 시장 위험으로 노출하지 않는다.
- 자동 주문이나 매매 실행을 하지 않는다.
