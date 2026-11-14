"""Generate the static pages for basestationtool.com or an alternate HTTPS origin."""
import argparse, csv, html, shutil
from pathlib import Path
from urllib.parse import urljoin
from PIL import Image, ImageDraw, ImageFont

root = Path(__file__).resolve().parent
out = root / "dist"
ap = argparse.ArgumentParser()
ap.add_argument("--base-url", default="https://www.basestationtool.com/")
base = ap.parse_args().base_url.rstrip("/") + "/"
if not base.startswith("https://"):
    raise ValueError("Use the actual HTTPS publication URL")
langs = dict(de="Deutsch", en="English", fr="Français", es="Español", it="Italiano")
locales = dict(de="de_DE", en="en_US", fr="fr_FR", es="es_ES", it="it_IT")
pages = {l: ("index.html" if l == "en" else l+".html") for l in langs}
with (root/"copy.tsv").open(encoding="utf-8", newline="") as f:
    rows = list(csv.DictReader(f, delimiter="\t"))
files = ["Basestation-Tool-0.1.1-x64.msi","basestation-tool_0.1.1_amd64.deb","Basestation-Tool-0.1.1-x86_64.AppImage"]
repository = "https://github.com/He1p1ess/BasestationTool"
release_download = repository + "/releases/download/v0.1.1/"
exe = "Basestation-Tool-0.1.1-x64.exe"
def font(size,bold=False):
    for name in (("segoeuib.ttf" if bold else "segoeui.ttf"),("DejaVuSans-Bold.ttf" if bold else "DejaVuSans.ttf")):
        try: return ImageFont.truetype(name,size)
        except OSError: pass
    return ImageFont.load_default(size=size)
logo='<span class="logo"><svg viewBox="0 0 32 32" aria-hidden="true"><path fill="none" stroke="currentColor" stroke-width="2" d="m5 10 11-5 11 5v13l-11 5-11-5z M5 10l11 5 11-5 M16 15v13"/><circle cx="22" cy="18" r="2" fill="currentColor"/></svg></span>'
history_pages = {lang: ("changelog.html" if lang == "en" else f"changelog-{lang}.html") for lang in langs}
for lang in langs:
    t={r["key"]:html.escape(r[lang]) for r in rows}
    assert all(t.values())
    og=Image.new("RGB",(1200,630),"#22172a"); draw=ImageDraw.Draw(og)
    for x in range(760,1200,24):draw.line((x,0,x,630),fill="#32273b")
    for y in range(0,630,24):draw.line((760,y,1200,y),fill="#32273b")
    draw.text((56,48),"BASESTATION TOOL    /    v0.1.1",font=font(30,True),fill="#dacce4")
    size=72
    while max(draw.textlength(html.unescape(t[k]),font=font(size,True)) for k in ("headline","accent"))>750:size-=2
    for k,y,color in [("headline",195,"#f1ebf6"),("accent",284,"#d1b3e8")]:draw.text((52,y),html.unescape(t[k]),font=font(size,True),fill=color)
    draw.text((56,519),"LIGHTHOUSE   /   BLUETOOTH LE   /   WINDOWS + LINUX",font=font(23),fill="#dad2e0")
    for poly,color in [([(849,188),(1003,138),(1113,183),(957,241)],"#83758d"),([(849,188),(957,241),(957,432),(849,381)],"#413749"),([(957,241),(1113,183),(1113,373),(957,432)],"#685a73"),([(970,250),(1100,202),(1100,355),(970,404)],"#1c1224")]:draw.polygon(poly,fill=color)
    draw.ellipse((1024,278,1047,304),fill="#d1b3e8")
    share="share-"+lang+".png";og.save(out/share)
    link=lambda name:urljoin(base,name)
    canonical=link(pages[lang])
    title="Basestation Tool · "+t["downloads"]
    alts=''.join(f'<link rel="alternate" hreflang="{l}" href="{link(v)}">' for l,v in pages.items())
    options=''.join(f'<option value="{v}" {"selected" if l==lang else ""}>{langs[l]}</option>' for l,v in pages.items())
    steps = ''.join(f'<article><span class="num">0{i} /</span><h3>{t[h]}</h3><p>{t[p]}</p></article>' for i,(h,p) in enumerate([("t1","p1"),("t3","p3"),("auto","autotext")],1))
    faq = ''.join(f'<details><summary>{t["q"+str(i)]}</summary><p>{t["a"+str(i)]}</p></details>' for i in range(1,4))
    packages = [
        ("Windows", "X64 · MSI", "msi", "MSI", files[0]),
        ("Windows", "X64 · EXE", "portable", "EXE", exe),
        ("Debian / Ubuntu", "AMD64 · LINUX", "deb", "DEB", files[1]),
        ("Linux AppImage", "X86_64 · LINUX", "app", "AppImage", files[2]),
    ]
    cards = ''.join(f'<article><span class="tag">{tag}</span><h3>{name}</h3><p>{t[key]}</p><a href="{release_download}{f}">{t["download"]} · {ext} ↓</a></article>' for name,tag,key,ext,f in packages)
    other=' · '.join(f'<a lang="{l}" hreflang="{l}" href="{p}">{langs[l]}</a>' for l,p in pages.items())
    (out/pages[lang]).write_text(f'''<!doctype html><html lang="{lang}"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{title}</title><meta name="description" content="{t["intro"]}"><link rel="canonical" href="{canonical}">{alts}<link rel="alternate" hreflang="x-default" href="{link("index.html")}"><meta property="og:type" content="website"><meta property="og:site_name" content="Basestation Tool"><meta property="og:locale" content="{locales[lang]}"><meta property="og:title" content="{title}"><meta property="og:description" content="{t["intro"]}"><meta property="og:url" content="{canonical}"><meta property="og:image" content="{link(share)}"><meta property="og:image:type" content="image/png"><meta property="og:image:width" content="1200"><meta property="og:image:height" content="630"><meta property="og:image:alt" content="Basestation Tool — {t["headline"]} {t["accent"]}"><meta name="twitter:card" content="summary_large_image"><meta name="twitter:title" content="{title}"><meta name="twitter:description" content="{t["intro"]}"><meta name="twitter:image" content="{link(share)}"><script src="theme.js"></script><link rel="stylesheet" href="style.css"></head><body>
<header><div class="wrap"><a class="brand" href="#top">{logo}Basestation Tool</a><nav><a href="#guide">{t["nav"]}</a><a href="#downloads">{t["downloads"]}</a><a href="#install">{t["install"]}</a><a href="#faq">FAQ</a><a href="{history_pages[lang]}">{t["history"]}</a></nav><div class="tools"><label class="sr" for="lang">{t["lang"]}</label><select id="lang">{options}</select><button type="button" id="theme" title="{t["theme"]}" aria-label="{t["theme"]}">◐</button></div></div></header>
<main class="wrap" id="top"><section class="hero"><div><div class="k">{t["eyebrow"]}</div><h1>{t["headline"]}<span>{t["accent"]}</span></h1><p class="intro">{t["intro"]}</p><div class="actions"><a class="btn" href="#downloads">{t["get"]} ↓</a><small class="meta">v0.1.1 · x86-64</small></div></div><figure><div class="paneltop"><span class="tag"><i class="light"></i>LIGHTHOUSE</span><span class="tag">FIG. 01</span></div><img class="station" src="station.svg" width="480" height="275" style="height:auto" alt=""><figcaption><span>{t["diagram"]}</span><b class="readout">BLE</b></figcaption></figure></section>
<section class="section" id="guide"><div class="k">01 / BLE</div><h2>{t["how"]}</h2><div class="steps">{steps}</div><aside class="channels"><h3>{t["channeltitle"]}</h3><p>{t["p2"]}</p></aside></section>
<section class="section" id="downloads"><div class="k">02 / {t["downloads"].upper()}</div><h2>{t["choose"]}</h2><div class="packs">{cards}</div><a class="checksum" href="{release_download}SHA256SUMS.txt">{t["checks"]}</a> · <a href="{history_pages[lang]}">{t["history"]}</a></section>
<section class="section" id="install"><div class="k">03 / {t["install"].upper()}</div><h2>{t["setup"]}</h2><details open><summary>Windows · MSI</summary><p>{t["winhelp"]}</p></details><details><summary>Windows · EXE</summary><p>{t["exehelp"]}</p></details><details><summary>Debian / Ubuntu · DEB</summary><p>{t["debhelp"]}</p><pre><code>sudo apt install ./{files[1]}\nbasestation-tool</code></pre></details><details><summary>Linux · AppImage</summary><p>{t["apphelp"]}</p><pre><code>chmod +x {files[2]}\n./{files[2]}</code></pre><p>{t["fuse"]}</p><pre><code>./{files[2]} --appimage-extract-and-run</code></pre></details><details><summary>{t["compat"]}</summary><p class="note">{t["notes"]}</p></details></section><section class="section" id="faq"><h2>FAQ</h2>{faq}</section></main>
<footer><div class="wrap"><small>Basestation Tool · v0.1.1<br>{t["ai"]} <a href="{repository}">{t["source"]}</a></small><small>{other}</small></div></footer></body></html>''',encoding="utf-8")

    history_options = ''.join(f'<option value="{path}" {"selected" if l == lang else ""}>{langs[l]}</option>' for l, path in history_pages.items())
    history_alts = ''.join(f'<link rel="alternate" hreflang="{l}" href="{link(path)}">' for l, path in history_pages.items())
    entries = []
    for version, date, keys in [
        ("0.1.1", "2026-11-14", ["wake011", "blink011", "restore011"]),
        ("0.1.0", "2026-11-10", ["initial010"]),
    ]:
        items = "".join(f"<li>{t[key]}</li>" for key in keys)
        entries.append(f'<section class="section"><time datetime="{date}">{date}</time><h2><a href="{repository}/releases/tag/v{version}">v{version}</a></h2><ul>{items}</ul></section>')
    (out/history_pages[lang]).write_text(f'''<!doctype html><html lang="{lang}"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{t["history"]} · Basestation Tool</title><link rel="canonical" href="{link(history_pages[lang])}">{history_alts}<link rel="alternate" hreflang="x-default" href="{link(history_pages["en"])}"><meta property="og:title" content="{t["history"]} · Basestation Tool"><meta property="og:image" content="{link(share)}"><meta property="og:url" content="{link(history_pages[lang])}"><script src="theme.js"></script><link rel="stylesheet" href="style.css"></head><body>
<header><div class="wrap"><a class="brand" href="{pages[lang]}">{logo}Basestation Tool</a><div class="tools"><label class="sr" for="lang">{t["lang"]}</label><select id="lang">{history_options}</select><button id="theme" aria-label="{t["theme"]}">◐</button></div></div></header>
<main class="wrap"><h1>{t["history"]}</h1>{''.join(entries)}<p><a href="{pages[lang]}#downloads">{t["downloads"]}</a></p></main></body></html>''', encoding="utf-8")

# Keep existing links to the English translation working.
shutil.copyfile(out/"index.html", out/"en.html")
