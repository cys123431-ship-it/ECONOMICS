# 중앙은행 유동성 방향성 모델

## 무엇을 측정하는가

이 화면은 Fed·ECB·BOJ 총자산의 절대규모만 보여 주지 않고 **전년 대비 증가율, 최근 변화 속도, 증가율의 가속·감속**을 함께 계산합니다. 핵심 질문은 “자산이 큰가?”가 아니라 “자산 공급 충격이 지금 늘어나는가, 줄어드는가, 그 속도가 빨라지는가?”입니다.

다만 중앙은행 총자산은 광의통화, 민간신용 또는 투자 가능한 현금과 같은 개념이 아닙니다. 따라서 이 지표는 중앙은행 유동성 방향을 요약하는 보조 신호이며 주가 상승·하락 확률이나 단독 매매 신호가 아닙니다.

## 공식 입력 데이터와 단위

| 항목 | FRED 시계열 | 원본 단위 | 화면 변환 |
|---|---|---|---|
| Fed 총자산 | [WALCL](https://fred.stlouisfed.org/series/WALCL) | 백만 달러, 주간 | 조 달러 |
| 유로시스템 총자산 | [ECBASSETSW](https://fred.stlouisfed.org/series/ECBASSETSW) | 백만 유로, 주간 | DEXUSEU 적용 조 달러 |
| BOJ 총자산 | [JPNASSETS](https://fred.stlouisfed.org/series/JPNASSETS) | 1억 엔, 월간 | DEXJPUS 적용 조 달러 |
| EUR/USD | [DEXUSEU](https://fred.stlouisfed.org/series/DEXUSEU) | 달러/유로 | ECB 달러 환산 |
| USD/JPY | [DEXJPUS](https://fred.stlouisfed.org/series/DEXJPUS) | 엔/달러 | BOJ 달러 환산 |
| 미 재무부 TGA | [WTREGEN](https://fred.stlouisfed.org/series/WTREGEN) | 백만 달러, 주간 | Fed 순유동성 차감 |
| ON RRP | [RRPONTSYD](https://fred.stlouisfed.org/series/RRPONTSYD) | 십억 달러, 일간 | Fed 순유동성 차감 |
| 은행 지급준비금 | [WRESBAL](https://fred.stlouisfed.org/series/WRESBAL) | 백만 달러, 주간 | 조 달러 및 3개월 연율 |

FRED는 각 원기관 통계를 배포하는 공식 데이터 경로로 사용합니다. 서로 다른 발표주기 때문에 종합 기준일은 세 중앙은행 입력 중 가장 느린 공식 관측일로 표시합니다. 결측이나 오래된 값은 임의로 채우지 않습니다.

## 두 개의 G3 증가율

### G3 FX중립 YoY — 화면의 핵심 검은선

각 중앙은행의 **현지통화 기준 YoY**를 현재 달러환산 자산규모로 가중합니다.

```text
G3 FX중립 YoY = Σ(현재 USD환산 자산규모 × 현지통화 YoY) / Σ(현재 USD환산 자산규모)
```

환율 변동이 중앙은행 자산정책처럼 보이는 착시를 줄이기 때문에 정책 유동성 방향 판단에는 이 선을 우선합니다.

### G3 USD환산 YoY — 환율 효과 포함 비교선

현재와 1년 전의 세 중앙은행 자산을 각각 당시 환율로 달러 환산한 뒤 합계의 YoY를 계산합니다. 달러 약세·강세 효과가 포함되므로 글로벌 달러 기준 구매력 변화에는 유용하지만 중앙은행 정책 충격과 동일하게 해석하면 안 됩니다.

## 수준·속도·가속도의 의미

| G3 YoY 수준 | 3개월 가속도 | 해석 |
|---|---:|---|
| 양수 | 양수 | 자산 확장이 빨라지는 강한 롱 지원 환경 |
| 양수 | 음수 | 자산은 여전히 확장 중이나 지원 강도는 둔화 |
| 음수 | 양수 | 자산은 여전히 축소 중이나 축소 속도가 완화되어 롱 방향으로 개선 |
| 음수 | 음수 | 자산 축소가 빨라지는 숏 압력 환경 |

`YoY 3개월 가속도 = 현재 YoY − 3개월 전 YoY`입니다. 예를 들어 YoY가 `−10%`에서 `−4%`로 올라오면 자산이 증가한 것이 아니라 **감소 속도가 완화된 것**입니다. 화면은 이를 “축소 중이나 완화”로 표시합니다.

`최근 3개월 연율`은 최근 약 91일의 자산변화를 연율화합니다. 오래된 기저효과가 남는 YoY보다 최근 방향 전환을 빨리 확인하기 위한 값입니다.

## 방향성 점수

| 요인 | 가중치 | −100/+100 완전점수 구간 |
|---|---:|---:|
| G3 FX중립 YoY 수준 | 30% | −10% / +10% |
| G3 YoY 3개월 가속도 | 25% | −8%p / +8%p |
| G3 최근 3개월 연율 | 20% | −15% / +15% |
| Fed 순유동성 3개월 연율 | 15% | −20% / +20% |
| 미국 지급준비금 3개월 연율 | 10% | −20% / +20% |

각 입력은 위 구간에서 선형으로 `−100~+100`에 제한합니다. 결측 요인은 분모에서 제외하고 가용 가중치를 신뢰도로 표시합니다. 가용 가중치가 60% 미만이면 점수를 내지 않고 `DATA WAIT`로 둡니다.

- `+40 이상`: 강한 롱 우세
- `+15 이상`: 롱 우세
- `−15 초과 +15 미만`: 중립·혼조
- `−40 이하`: 강한 숏 우세
- 그 사이 음수: 숏 우세

화면의 `LONG 지원 = 50 + 점수/2`, `SHORT 압력 = 50 − 점수/2`는 같은 방향성 점수를 읽기 쉽게 양분한 것입니다. 둘 다 수익 확률이 아닙니다.

## Fed 시장 배관 보조치

```text
Fed 순유동성 = Fed 총자산 − 미 재무부 TGA − ON RRP
```

- TGA 증가: 민간 은행계좌에서 재무부 계정으로 현금이 이동하므로 다른 조건이 같으면 은행 준비금을 흡수합니다.
- TGA 감소: 재정지출을 통해 민간으로 현금이 이동하므로 다른 조건이 같으면 준비금을 공급합니다.
- ON RRP 감소: 머니마켓펀드 등이 Fed에 두었던 현금이 민간 자산으로 이동할 여지를 줍니다. 잔액이 이미 낮으면 같은 완충효과가 반복되기 어렵습니다.
- 지급준비금 증가: 결제·레포시장 충격을 흡수할 완충재가 커지는 방향입니다.

Fed도 대차대조표 축소와 TGA·ON RRP 변화가 준비금에 서로 다르게 전달된다고 설명합니다. 관련 공식 설명은 [Federal Reserve Balance Sheet Developments, November 2025](https://www.federalreserve.gov/monetarypolicy/November-2025-Federal-Reserve-Balance-Sheet-Developments.htm)와 [The Central Bank Balance Sheet Trilemma](https://www.federalreserve.gov/econres/notes/feds-notes/the-central-bank-balance-sheet-trilemma-20260114.html)를 참고했습니다.

ECB의 자산 정상화와 기간 프리미엄 영향은 [ECB, The ECB balance sheet: a decade of change and the road ahead](https://www.ecb.europa.eu/press/key/date/2025/html/ecb.sp251106~1133f93311.en.html), BOJ의 JGB 매입축소와 통화기저 경로는 [BOJ, Monetary Policy after the Revival of Inflation](https://www.boj.or.jp/en/about/press/koen_2025/ko250902a.htm) 및 [BOJ Review, April 2026](https://www.boj.or.jp/en/research/wps_rev/rev_2026/rev26e10.htm)을 참고했습니다.

## 판단 경계

- 중앙은행 자산 증가가 곧바로 주식 상승을 보장하지 않습니다. 물가, 금리, 재정, 달러, 신용스프레드, 밸류에이션과 시장 포지셔닝이 전달경로를 바꿉니다.
- 자산매입의 종류와 만기구성이 다르면 같은 총액 변화도 금융시장 효과가 다를 수 있습니다.
- 달러 환산 절대규모는 비교용이며 환율 자체의 변동을 정책으로 오해하지 않습니다.
- 중앙은행 탭의 결과는 미국·한국·코인 탭의 가격·신용·변동성·시장 내부수급과 함께 확인합니다.
