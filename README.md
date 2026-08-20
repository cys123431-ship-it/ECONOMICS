# ECONOMICS Radar

Windows 중심의 저메모리 Rust 금융시장 위험 감시기입니다. v4 ULTRA의 **85개 canonical 규칙군과 총 27,494개 규칙 topology**를 Rust 코드에 고정하고, 실행 시 조건을 절차적으로 생성·평가합니다. 10MB급 텍스트 룰북을 메모리에 올리지 않기 때문에 규칙 수가 많아도 상시 RAM 사용량을 작게 유지할 수 있습니다.

## 설계
- Rust 네이티브 단일 EXE
- v4 ULTRA topology: 85 rule families / 27,494 deterministic rules
- 대형 텍스트 룰북 상주 없음, 규칙을 순차 생성·평가하고 발동 결과만 최대 64개 보관
- SQLite WAL, 4 MiB cache, `mmap_size=0`, recent series max 256 samples
- blocking HTTP: 수집 시점에만 네트워크 client 생성
- FRED/ALFRED, Treasury FiscalData, BOK ECOS, KRX approved endpoint, Binance Futures
- API 키는 `.env` 또는 환경변수에만 저장하고 값 자체는 출력하지 않음
- risk / confidence / diffusion / crisis stage 분리

> v0.2.0부터는 v4 ULTRA 원문 TXT를 바이트 단위로 내장하는 방식이 아니라, 원문에서 추출한 규칙군 구조와 개수를 기반으로 동일한 27,494개 슬롯을 결정론적으로 생성하는 저메모리 엔진을 사용합니다. 따라서 원문의 모든 CONDITION/MSG 문구를 그대로 보존하는 형식은 아닙니다.

## Windows
Release의 `EconomicsRadar-Windows-x64.zip`을 풀고 `.env.example`을 `.env`로 복사한 뒤 키를 입력합니다.

```powershell
.\EconomicsRadar.exe keys
.\EconomicsRadar.exe rulebook
.\EconomicsRadar.exe collect-fred 2000-01-01
.\EconomicsRadar.exe collect-alfred 2000-01-01
.\EconomicsRadar.exe collect-official
.\EconomicsRadar.exe run
.\EconomicsRadar.exe serve
```

`rulebook`은 `rules=27494 families=85 mode=procedural-v4-ultra low_memory=true`를 검증합니다.

`serve` 기본 주소: http://127.0.0.1:8765

## 안전
이 프로젝트는 금융시장 리스크 감시/연구 도구이며 투자수익을 보장하거나 자동 매매 주문을 실행하지 않습니다.
