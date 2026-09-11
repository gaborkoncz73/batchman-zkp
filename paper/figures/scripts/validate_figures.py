#!/usr/bin/env python3
"""Mechanical QA for the generated figure package."""
from pathlib import Path
import re, subprocess, sys
import xml.etree.ElementTree as ET

ROOT=Path(__file__).resolve().parents[1]
errors=[]
expected=[f"fig{i:02d}_" for i in range(1,11)]
svgs=sorted((ROOT/"src").glob("fig*.svg"))
if len(svgs)!=10: errors.append(f"expected 10 SVGs, found {len(svgs)}")
for prefix in expected:
    if not any(p.name.startswith(prefix) for p in svgs): errors.append(f"missing {prefix} SVG")
for svg in svgs:
    try: ET.parse(svg)
    except Exception as exc: errors.append(f"invalid XML {svg.name}: {exc}")
    raw=svg.read_text(encoding="utf-8")
    if "<foreignObject" in raw: errors.append(f"non-editable foreignObject in {svg.name}")
    for phrase in ("aggregated proof","formal verification","universal setup","toxic waste","in-circuit signature"):
        if phrase.lower() in raw.lower(): errors.append(f"misleading phrase '{phrase}' in {svg.name}")
    m=re.search(r'width="([0-9.]+)mm" height="([0-9.]+)mm"',raw[:300])
    if not m: errors.append(f"missing physical dimensions in {svg.name}")
    for ext,folder in ((".pdf","pdf"),(".png","png")):
        out=ROOT/folder/(svg.stem+ext)
        if not out.exists() or out.stat().st_size<1000: errors.append(f"missing/empty export {out}")
for png in sorted((ROOT/"png").glob("fig*.png")):
    from PIL import Image
    with Image.open(png) as im:
        if im.width<2000: errors.append(f"PNG appears below 600 dpi target: {png.name} {im.size}")
for doc in ("README.md","figure_manifest.md","captions.md","alt_text.md","style/figure_style.md","style/palette.svg","png/contact_sheet.png","png/contact_sheet_grayscale.png"):
    if not (ROOT/doc).exists(): errors.append(f"missing {doc}")
if errors:
    print("QA FAILED")
    print("\n".join("- "+e for e in errors)); sys.exit(1)
print("QA passed: 10 well-formed SVGs, matching PDF/PNG exports, required documentation, and forbidden-term scan.")
