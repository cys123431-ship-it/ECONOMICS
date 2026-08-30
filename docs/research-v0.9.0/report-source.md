# ECONOMICS Radar v0.9.0 Deep Research

Date: 2026-08-28  
Scope: Bloomberg Terminal-inspired workflow, official market-risk sources, and the current v0.8.1 desktop dashboard audit

## Executive conclusion

The current application already exposes a broad set of raw values, sources, timestamps, and market-specific panels. Its main limitation is not the absolute number of rows. The limiting factor is workflow: users cannot search the full terminal, pin a personal monitor, collapse low-priority sections, or surface missing/stale/abnormal observations in one place. The v0.9.0 work should therefore add navigation and triage first, then add official datasets that fill clear analytical gaps.

Bloomberg's public product material describes the Terminal as a multi-asset data-and-analytics workspace. Launchpad provides customized security monitors and alerts; Charts supports multi-instrument comparison and reusable templates; PORT combines positions, risk, performance attribution, scenario analysis, and data validation. The transferable idea is a linked workflow, not visual imitation.

## Current-state audit

Audited build: v0.8.1 installed at `E:\EconomicsRadar` and served at `http://127.0.0.1:8765`.

### What works

- Separate overview, US, Korea, crypto, and central-bank workspaces.
- Raw values, changes, observation date, source, and freshness are visible.
- Market direction uses explicit rising/falling/sideways language and is separated from risk.
- Official KRX/ECOS/FRED collection and crypto market data already cover the core dashboard.
- Tables, headings, tabs, and the central-bank chart have a workable semantic structure.

### Material gaps

- US and Korea pages are several screens long; every section is expanded, and small type makes scanning difficult.
- There is no terminal-wide search, command shortcut, saved monitor, or one-click jump to an indicator.
- Missing, stale, and exceptional data are distributed across many sections instead of being triaged.
- Tabs do not implement the complete keyboard pattern (arrow, Home, End, focus management).
- Short-term funding, institutional futures positioning, crypto options volatility, and on-chain/stablecoin liquidity are not represented.
- Several official Korean macro series available from ECOS are absent, including CPI, M2, and benchmark government yields.

## Evidence and source selection

### Bloomberg workflow references

- [Bloomberg Terminal](https://professional.bloomberg.com/products/bloomberg-terminal/) — multi-asset data, analytics, Launchpad monitors, alerts, and charts.
- [Portfolio & Risk Analytics](https://professional.bloomberg.com/products/bloomberg-terminal/portfolio-analytics/) — unified positions, attribution, scenario stress tests, validation, and reports.
- [Bloomberg Charts](https://professional.bloomberg.com/products/bloomberg-terminal/charts/) — multi-instrument comparison and reusable chart templates.
- [Bloomberg Professional App](https://professional.bloomberg.com/products/bloomberg-terminal/access/bloomberg-professional-app/) — watchlists, worksheets, events, research, charts, and alerts.

### Risk framework

The Federal Reserve's May 2026 financial-stability framework monitors four broad vulnerabilities: valuation pressures; borrowing by businesses and households; financial-sector leverage; and funding risks. ECONOMICS Radar should present these as observable axes rather than an unexplained single confidence percentage.

- [Federal Reserve Financial Stability Report framework](https://www.federalreserve.gov/publications/2026-may-financial-stability-report-purpose-and-framework.htm)

### Forecasting and validation evidence

- [New York Fed yield-curve model](https://www.newyorkfed.org/research/capital_markets/ycfaq.htm) uses the 10-year minus 3-month term spread for a clearly defined 12-month recession probability and explicitly states that the estimate is not an official Federal Reserve forecast. The transferable rule is to name the target, horizon, data, and limitation.
- [Chicago Fed NFCI](https://www.chicagofed.org/research/data/nfci/current-data) separates financial conditions from current economic conditions. It is context for a forecast, not a direct equity direction label.
- Moskowitz, Ooi, and Pedersen, [Time Series Momentum](https://fairmodel.econ.yale.edu/ec439/mosk.pdf), documents persistence in an instrument's own returns over intermediate horizons across liquid futures. This supports including transparent price-trend features, but not claiming certainty.
- Gneiting et al., [Probabilistic forecasts, calibration and sharpness](https://rss.onlinelibrary.wiley.com/doi/pdf/10.1111/j.1467-9868.2007.00587.x), establishes calibration and proper scoring rules as core probability-forecast evaluation tools.
- Bailey and López de Prado, [Deflated Sharpe Ratio](https://doi.org/10.2139/ssrn.2460551), and Bailey et al., [Probability of Backtest Overfitting](https://escholarship.org/uc/item/4w1110bb), show why repeated model selection inflates backtest performance. The application must preserve out-of-time evaluation and report when a model fails to beat a naive baseline.

### New official/free sources

- [OFR Short-term Funding Monitor API](https://www.financialresearch.gov/short-term-funding-monitor/api/) — public JSON API with daily SOFR, EFFR, volume, and repo series; no separate user credential.
- [CFTC Public Reporting User Guide](https://publicreporting.cftc.gov/stories/s/User-s-Guide/p2fg-u73y/) and [Traders in Financial Futures](https://www.cftc.gov/dea/futures/financial_lf.htm) — weekly open interest and trader-category positions.
- [Deribit public book summary](https://docs.deribit.com/api-reference/market-data/public-get_book_summary_by_currency) and [DVOL history](https://docs.deribit.com/api-reference/market-data/public-get_volatility_index_data) — public BTC option open interest, volume, put/call balance, and implied-volatility index.
- [Coin Metrics API v4](https://docs.coinmetrics.io/api/v4/) — community asset metrics for BTC network activity, MVRV, fees, hash rate, and stablecoin market capitalization.
- [Bank of Korea ECOS](https://ecos.bok.or.kr/) — official Korean CPI, M2, base rate, USD/KRW, and government-bond yield statistics.

## Gap-to-implementation matrix

| Gap | Evidence | v0.9.0 implementation | Interpretation boundary |
|---|---|---|---|
| Slow navigation | Bloomberg Launchpad/worksheets/watchlists | Global search, `/` shortcut, click-to-jump results, favorites monitor | Navigation only; no predictive claim |
| Excessive page length | Local F1-F5 visual audit | Collapsible sections, core/all view, readable density | Does not hide stale/error rows |
| Weak exception handling | Bloomberg alert/data-validation workflow | Overview priority queue for missing, stale, and extreme changes | Priority is rule-based and transparent |
| Funding blind spot | Fed funding-risk axis; OFR API | SOFR, EFFR, tail spread, rate spread, volume, repo monitor | Stress proxy, not a direct price forecast |
| Positioning blind spot | CFTC TFF | Asset-manager and leveraged-fund net position as % of OI | Weekly positioning; sign is not a trade signal by itself |
| Crypto derivatives blind spot | Deribit public API | BTC DVOL, option OI, put/call OI, 24h option volume | Venue-specific and not the whole options market |
| On-chain/liquidity blind spot | Coin Metrics community API | MVRV, active addresses, transactions, fees, hash rate, USDT+USDC cap | Network/liquidity context, not guaranteed direction |
| Korean macro/rates gaps | ECOS official catalog | CPI, M2, 3Y and 10Y government yields, 10Y-3Y curve | Publication cadence shown explicitly |
| Incomplete keyboard UX | Local semantic audit | Arrow/Home/End tabs, F6 workspace, focus state, reduced-motion support | Accessibility/workflow enhancement |
| Ambiguous prediction claims | Forecast-calibration and backtest-overfit literature | Named horizons; rise/sideways/fall probabilities; explicit flat bands; minimum 700 sessions; rolling out-of-time Brier score and hit rate against the prior class-frequency baseline; no forecast when history is insufficient | Conditional empirical frequency, not guaranteed return probability |

## Product rules

1. Never replace a missing value with an estimate; display `NO DATA` and the expected cadence.
2. Every metric must retain source, original value, observation time, change, and freshness.
3. Price direction remains only rising, falling, or sideways. Risk/funding/positioning measures must not be labeled as certain market direction.
4. Derived values must name their formula in the UI or methodology copy.
5. The default view prioritizes abnormal, stale, and pinned metrics; the complete view remains available in one action.
6. Every forecast must state the asset, target horizon, sideways threshold, sample count, validation window, and prior class-frequency baseline.
7. A forecast without sufficient history or without out-of-time baseline advantage is labeled observation-only; it must not be promoted as a directional edge.

## Acceptance criteria

- Search locates an indicator across every tab and jumps to the exact row.
- Favorites persist locally and appear in the overview monitor.
- F6 contains funding, CFTC positioning, crypto options, and on-chain liquidity groups.
- New official collectors fail independently and never prevent other sources from updating.
- Monthly and weekly cadence is reflected in freshness instead of being mislabeled stale.
- Tab keyboard navigation and visible focus work without a mouse.
- Rust tests, dashboard smoke tests, and a live installed-build browser check pass.
- Experimental forecasts report 1/5/21-session rise/sideways/fall probabilities, explicit bands, analog counts, rolling validation hit rate, and multiclass Brier error against the prior class-frequency baseline.
