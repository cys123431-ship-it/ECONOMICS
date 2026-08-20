# ECONOMICS Radar

Windows 중심의 저메모리 Rust 금융시장 위험 감시기입니다. v4 ULTRA 27,494개 룰을 단일 gzip으로 내장하고 전체 룰을 `Vec<Rule>`로 상주시킬 필요 없이 64 KiB 스트리밍 버퍼로 평가합니다.

## 설계
- Rust 네이티브 단일 EXE
- v4 ULTRA 27,494 rules, single gzip asset
- SQLite WAL, 4 MiB cache, `mmap_size=0`, recent series max 256 samples
- blocking HTTP: 수집 시점에만 네트워크 client 생성
- FRED/ALFRED, Treasury FiscalData, BOK ECOS, KRX approved endpoint, Binance Futures
- API 키는 `.env` 또는 환경변수에만 저장하고 출력하지 않음
- risk / confidence / diffusion / crisis stage 분리

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

`serve` 기본 주소: http://127.0.0.1:8765

## 안전
이 프로젝트는 리스크 감시/연구 도구이며 투자수익을 보장하거나 자동 매매 주문을 실행하지 않습니다.
