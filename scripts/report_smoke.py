#!/usr/bin/env python3
"""GitHub Actions end-to-end smoke report, strictly grounded in SQLite observations."""
import argparse
import html
import json
import os
import sqlite3
from collections import defaultdict
from datetime import datetime, timezone, timedelta
from pathlib import Path
from urllib.parse import quote

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib import font_manager
from reportlab.lib import colors
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.enums import TA_CENTER
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle, Image, PageBreak

KST = timezone(timedelta(hours=9))
FONT = "/usr/share/fonts/truetype/nanum/NanumGothic.ttf"
if os.path.isfile(FONT):
    pdfmetrics.registerFont(TTFont("KR", FONT))
    font_manager.fontManager.addfont(FONT)
    plt.rcParams["font.family"] = "NanumGothic"
    REPORT_FONT = "KR"
else:
    REPORT_FONT = "Helvetica"
plt.rcParams["axes.unicode_minus"] = False

# These are *illustrative observable series* for each of the 30 categories.
# The report does not claim a category is fully analyzed when a single series exists.
GROUPS = [
 ("01","미국 통화정책",["fred:IORB","fred:DGS2"]),
 ("02","물가·인플레이션",["fred:CPIAUCSL","fred:PCEPI","ecos:KR_CPI"]),
 ("03","미국 고용·경기",["fred:ICSA","fred:CFNAI","fred:WEI"]),
 ("04","환율·달러",["fred:DEXKOUS","fred:DEXJPUS","ecos:KR_USD_KRW"]),
 ("05","채권시장",["fred:DGS2","fred:DGS10","fred:DGS30"]),
 ("06","글로벌 유동성",["fred:WALCL","fred:WTREGEN","fred:RRPONTSYD","fred:ECBASSETSW"]),
 ("07","주식·기업실적",["fred:SP500","fred:NASDAQCOM","fred:DJIA"]),
 ("08","암호화폐",["binance:BTC_SPOT_PRICE_USD","binance:BTC_OI","binance:BTC_FUNDING_RATE","coinmetrics:BTC_PRICE_USD"]),
 ("09","원자재·에너지",["krx:KRX_GOLD","fred:DCOILWTICO"]),
 ("10","한국 경제",["ecos:KR_BASE_RATE","ecos:KR_CPI","krx:KRX_KOSPI_CLOSE"]),
 ("11","중국·일본·유럽",["fred:ECBASSETSW","fred:JPNASSETS","fred:CHNLOLITOAASTSAM"]),
 ("12","금융시장 위험",["fred:VIXCLS","fred:STLFSI4","ofr_fsi:OFR_FSI"]),
 ("13","국제정세·정책",[]),
 ("14","기관투자자 자금",["cftc:UST_NET_POSITION"]),
 ("15","경제 일정·기대치",[]),
 ("16","미국 재정·국가부채",["fred:WTREGEN"]),
 ("17","부동산·주택시장",["fred:MORTGAGE30US"]),
 ("18","은행·비은행 금융",["fred:BUSLOANS","fred:TOTLL"]),
 ("19","단기자금·달러 조달",["ofr_repo:SOFR","ofr_repo:SOFR_EFFR_SPREAD"]),
 ("20","기업 신용·부도 위험",["fred:BAMLH0A0HYM2","fred:BAMLC0A0CM"]),
 ("21","옵션·파생상품",["deribit:BTC_DVOL","deribit:BTC_OPTION_OI"]),
 ("22","주식시장 내부 체력",[]),
 ("23","기업 이익·산업 변화",[]),
 ("24","국제무역·공급망",[]),
 ("25","신흥국 금융시장",["fred:KORLOLITOAASTSAM","fred:CHNLOLITOAASTSAM"]),
 ("26","에너지 수급 구조",[]),
 ("27","정부 정책·규제",[]),
 ("28","장기 구조적 변화",["bis:GLOBAL_DOLLAR_CREDIT"]),
 ("29","시장 충격·위기 전염",["ofr_fsi:OFR_FSI","fred:STLFSI4"]),
 ("30","데이터 신뢰성·분석 검증",[]),
]

def get_rows(db):
    if not db.exists():
        return [], "database missing"
    try:
        with sqlite3.connect(str(db)) as conn:
            result = conn.execute("""SELECT source, series, entity, observed_at, value,
                                     released_at, source_asof, ingested_at
                                     FROM observations_v2 ORDER BY ingested_at DESC""").fetchall()
            live = conn.execute("""SELECT source, series, entity, observed_at, value,
                                   NULL, source_asof, ingested_at FROM live_quotes
                                   ORDER BY ingested_at DESC""").fetchall()
        return result + live, None
    except (sqlite3.Error, OSError) as exc:
        return [], str(exc)

def origin_url(source, series):
    if source == "fred":
        return "https://fred.stlouisfed.org/series/" + quote(series)
    return {
        "binance":"https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api",
        "krx":"https://openapi.krx.co.kr/",
        "ecos":"https://ecos.bok.or.kr/",
        "ofr_repo":"https://www.financialresearch.gov/short-term-funding-monitor/",
        "ofr_fsi":"https://www.financialresearch.gov/financial-stress-index/",
        "cftc":"https://www.cftc.gov/MarketReports/CommitmentsofTraders/",
        "deribit":"https://docs.deribit.com/",
        "coinmetrics":"https://docs.coinmetrics.io/api/v4/",
        "bis":"https://data.bis.org/"
    }.get(source,"https://github.com/cys123431-ship-it/ECONOMICS")

def series_matches(latest, identifier):
    if ":" not in identifier:
        return None
    return latest.get(tuple(identifier.split(":",1)))

def render_text_card(path, title, subtitle, items, footer):
    fig = plt.figure(figsize=(9, 12), facecolor="#f8fafc")
    fig.text(.07,.93,title, fontsize=23, weight="bold",color="#142a44")
    fig.text(.07,.89,subtitle, fontsize=11,color="#5a6980")
    n=max(1,len(items))
    y=.83
    step=min(.095, .67/n)
    for label, value, note in items[:12]:
        fig.text(.07,y,label,fontsize=12,weight="bold",color="#1d344e")
        fig.text(.93,y,value,fontsize=14,ha="right",weight="bold",color="#0d6881")
        fig.text(.07,y-.025,note[:110],fontsize=9,color="#58667d")
        y-=step
    fig.text(.07,.075,footer,fontsize=10,color="#657387",wrap=True)
    fig.text(.07,.04,"ECONOMICS RADAR | GitHub Actions SMOKE TEST",fontsize=9,color="#7b869b")
    fig.savefig(path,dpi=155,bbox_inches="tight",facecolor=fig.get_facecolor())
    plt.close(fig)

def run():
    ap=argparse.ArgumentParser()
    ap.add_argument("--db",default="runtime/economics.db")
    ap.add_argument("--out",default="macro-report-smoke")
    args=ap.parse_args()
    out=Path(args.out)
    out.mkdir(parents=True,exist_ok=True)
    rows,err=get_rows(Path(args.db))
    latest={}
    for x in rows:
        source,series,entity,at,value,released,source_asof,ingested=x
        key=(source,series)
        # Observations may arrive in any ingestion order. Select newest actual observation.
        if key not in latest or str(at or "")>str(latest[key][3] or ""):
            latest[key]=x
    coverage=[]
    for num,name,ids in GROUPS:
        matched=[(sid,series_matches(latest,sid)) for sid in ids]
        present=[(sid,r) for sid,r in matched if r is not None]
        status="NO-MAP" if not ids else "NO-DATA" if not present else "PARTIAL" if len(present)<len(ids) else "REFERENCE-ONLY"
        # REFERENCE-ONLY explicitly means this is not a completeness certification.
        coverage.append({"id":num,"name":name,"status":status,"found":len(present),
                         "expected_reference":len(ids),"examples":[
                         {"id":sid,"value":r[4],"observed_at":r[3],
                          "source":origin_url(r[0],r[1])} for sid,r in present]})
    summary={s:sum(g["status"]==s for g in coverage) for s in ["REFERENCE-ONLY","PARTIAL","NO-DATA","NO-MAP"]}
    (out/"quality.json").write_text(json.dumps({
        "generated_at":datetime.now(KST).isoformat(), "test_only":True,
        "purpose":"GitHub Actions pipeline smoke test; not a complete 30-domain daily macro report",
        "rows_loaded":len(rows), "distinct_source_series":len(latest),
        "db_error":err,"coverage_summary":summary,"domains":coverage
        },ensure_ascii=False,indent=2),encoding="utf-8")
    latest_items=sorted(latest.items(),key=lambda kv:str(kv[1][3]),reverse=True)
    market_cards=[]
    for (source,series),r in latest_items[:10]:
        try:value=f"{float(r[4]):,.5g}"
        except (ValueError,TypeError):value="N/A"
        market_cards.append((series,value,f"{source} | 관측: {str(r[3])[:19]}"))
    if not market_cards:
        market_cards=[("NO DATA","—","수집 성공 여부와 API 키, 실행 로그를 확인해야 합니다.")]
    timestamp=datetime.now(KST).strftime("%Y-%m-%d %H:%M KST")
    render_text_card(out/"01_market.png","글로벌 시장 데이터 점검",timestamp, market_cards,
                     "시장별 단위가 다릅니다. 이 페이지는 실제 수집된 최근 관측치만 표시합니다.")
    completeness=[
       ("대표지표 전체 매핑 확인",str(summary["REFERENCE-ONLY"]),"완전한 경제분야 분석을 의미하지 않습니다."),
       ("대표지표 부분 확인",str(summary["PARTIAL"]),"참고할 일부 지표만 확인했습니다."),
       ("수집 실패 / 값 없음",str(summary["NO-DATA"]),"정의된 지표에서 관측값을 확보하지 못했습니다."),
       ("수집 매핑 미설계",str(summary["NO-MAP"]),"뉴스·재정 등 추가 모듈이 필요합니다."),
       ("DB 관측행",str(len(rows)),"실제 SQLite 관측값 수"),
       ("출처-시계열 조합",str(len(latest)),"실제 식별된 데이터 계열 수"),
    ]
    render_text_card(out/"02_macro.png","30개 분야 수집 품질",timestamp,completeness,
                     "불충분하거나 미연결인 영역은 빈 상태 그대로 보존합니다. 추정값을 생성하지 않습니다.")
    crypto_keys=["binance:BTC_SPOT_PRICE_USD","binance:BTC_FUNDING_RATE","binance:BTC_OI",
                 "deribit:BTC_DVOL","coinmetrics:BTC_PRICE_USD"]
    crypto=[]
    for sid in crypto_keys:
        r=series_matches(latest,sid)
        crypto.append((sid.split(":",1)[1], f"{float(r[4]):,.6g}" if r else "미확보",
                       "기준 "+str(r[3])[:19] if r else "실제 관측값이 없습니다."))
    render_text_card(out/"03_crypto.png","코인 전망 데이터 검증",timestamp,crypto,
                     "자동 매매 권고나 검증되지 않은 가격예측을 생성하지 않습니다.")
    styles=getSampleStyleSheet()
    styles.add(ParagraphStyle(name="KTitle",parent=styles["Title"],fontName=REPORT_FONT,fontSize=18,leading=24,textColor=colors.HexColor("#172943")))
    styles.add(ParagraphStyle(name="KBody",parent=styles["BodyText"],fontName=REPORT_FONT,fontSize=9,leading=14,spaceAfter=8))
    styles.add(ParagraphStyle(name="KSmall",parent=styles["BodyText"],fontName=REPORT_FONT,fontSize=7.8,leading=11))
    text=lambda s:Paragraph(html.escape(str(s)),styles["KSmall"])
    story=[
      Paragraph("ECONOMICS 일간 거시경제: GitHub Actions 실제 실행 검증",styles["KTitle"]),
      Spacer(1,12),Paragraph("작성: "+timestamp,styles["KBody"]),
      Paragraph("중요: 이것은 자동 수집과 PDF/이미지 생성이 가능한지 확인하는 기술적 테스트입니다. 30개 분야 심층 분석 완성본이 아닙니다.",styles["KBody"]),
      Paragraph("SQLite 관측행: "+str(len(rows))+" / 실제 데이터 계열: "+str(len(latest))+" / 오류: "+str(err or "없음"),styles["KBody"]),
      Paragraph("확인 분포: "+", ".join(k+"="+str(v) for k,v in summary.items()),styles["KBody"]),
      Spacer(1,8),
      Image(str(out/"01_market.png"),width=270,height=360),
      PageBreak(),
      Paragraph("30개 분야별 대표지표 조사 상태",styles["KTitle"]),
      Paragraph("REFERENCE-ONLY는 대표 예시 지표가 있는 상태입니다. 분야의 모든 지표·해설이 완성되었다는 뜻이 아닙니다.",styles["KBody"])
    ]
    cells=[[text("번호"),text("분야"),text("기준 지표"),text("확인"),text("상태")]]
    for v in coverage:
        cells.append([text(v["id"]),text(v["name"]),text(v["expected_reference"]),text(v["found"]),text(v["status"])])
    table=Table(cells,colWidths=[35,170,75,49,100],repeatRows=1,hAlign="LEFT")
    table.setStyle(TableStyle([("GRID",(0,0),(-1,-1),.35,colors.HexColor("#dce2ea")),
           ("BACKGROUND",(0,0),(-1,0),colors.HexColor("#edf2f8")),
           ("VALIGN",(0,0),(-1,-1),"TOP"),("LEFTPADDING",(0,0),(-1,-1),5),
           ("RIGHTPADDING",(0,0),(-1,-1),5),("TOPPADDING",(0,0),(-1,-1),5),
           ("BOTTOMPADDING",(0,0),(-1,-1),5)]))
    story += [table,PageBreak(),Paragraph("실제 수집 지표 및 검증 가능한 원본 링크",styles["KTitle"])]
    if latest_items:
        for (source,series),r in latest_items[:45]:
            url=origin_url(source,series)
            label=f"{source}:{series} = {r[4]} | {r[3]}"
            story.append(Paragraph(html.escape(label),styles["KSmall"]))
            story.append(Paragraph('<link href="'+html.escape(url,quote=True)+'" color="blue">'+html.escape(url)+'</link>',styles["KSmall"]))
    else:
        story.append(Paragraph("이번 실행에서 SQLite 실측 데이터를 확인하지 못했습니다. 수집 로그와 API 인증 상태를 확인해야 합니다.",styles["KBody"]))
    story += [PageBreak(), Paragraph("금융시장·코인 브리핑 파일",styles["KTitle"]),
        Image(str(out/"02_macro.png"),width=245,height=327),Spacer(1,12),
        Image(str(out/"03_crypto.png"),width=245,height=327)]
    doc=SimpleDocTemplate(str(out/"2026_Macro_Actions_Smoke.pdf"),pagesize=A4,
                          leftMargin=32,rightMargin=32,topMargin=35,bottomMargin=35)
    doc.build(story)
    for fn in ["2026_Macro_Actions_Smoke.pdf","01_market.png","02_macro.png","03_crypto.png","quality.json"]:
        p=out/fn
        if not p.is_file() or p.stat().st_size<400:
            raise RuntimeError("Artifact missing/too small: "+str(p))
    print("SMOKE_REPORT_OK","rows",len(rows),"distinct",len(latest),"quality",summary,flush=True)

if __name__=="__main__":
    run()
