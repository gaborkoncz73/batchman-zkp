#!/usr/bin/env python3
"""Generate the manuscript figure set as editable SVG plus PDF/PNG exports.

The artwork is deliberately generated from a small vector vocabulary so that
labels, geometry, palette, and implemented/proposed line semantics remain
consistent.  No production source is read or modified by this script.
"""

from __future__ import annotations

import html
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC, PDF, PNG = ROOT / "src", ROOT / "pdf", ROOT / "png"

C = {
    "blue": "#245A8D", "teal": "#168A83", "slate": "#536575",
    "violet": "#7256A8", "green": "#2C8B57", "amber": "#D58A16",
    "red": "#C84A3A", "gray": "#E9EDF0", "ink": "#263238",
    "pblue": "#EAF2F8", "pteal": "#E8F5F3", "pgray": "#F4F6F7",
    "pviolet": "#F1EDF8", "pamber": "#FFF5E3", "pred": "#FBEDEA",
    "white": "#FFFFFF", "mid": "#AAB5BD",
}


class Fig:
    def __init__(self, name: str, width_mm: int, height_mm: int):
        self.name, self.wmm, self.hmm = name, width_mm, height_mm
        self.w, self.h = width_mm * 10, height_mm * 10
        self.e = []

    def esc(self, s): return html.escape(str(s), quote=True)

    def rect(self, x, y, w, h, *, fill="white", stroke="ink", sw=2.2,
             r=12, dash=None, opacity=1, cls=None):
        ds = f' stroke-dasharray="{dash}"' if dash else ""
        cl = f' class="{cls}"' if cls else ""
        self.e.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}" '
                      f'fill="{C.get(fill, fill)}" stroke="{C.get(stroke, stroke)}" '
                      f'stroke-width="{sw}" opacity="{opacity}"{ds}{cl}/>' )

    def line(self, x1, y1, x2, y2, *, stroke="ink", sw=2.5, dash=None,
             arrow=False, both=False, opacity=1):
        ds = f' stroke-dasharray="{dash}"' if dash else ""
        end = ' marker-end="url(#arrow)"' if arrow else ""
        start = ' marker-start="url(#arrowStart)"' if both else ""
        self.e.append(f'<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" '
                      f'stroke="{C.get(stroke, stroke)}" stroke-width="{sw}" '
                      f'opacity="{opacity}"{ds}{start}{end}/>' )

    def path(self, d, *, stroke="ink", sw=2.5, fill="none", dash=None, arrow=False):
        ds = f' stroke-dasharray="{dash}"' if dash else ""
        end = ' marker-end="url(#arrow)"' if arrow else ""
        self.e.append(f'<path d="{d}" stroke="{C.get(stroke, stroke)}" '
                      f'stroke-width="{sw}" fill="{C.get(fill, fill)}"{ds}{end}/>' )

    def text(self, x, y, lines, *, size=23, weight=400, fill="ink", anchor="start",
             italic=False, leading=1.18):
        if isinstance(lines, str): lines = [lines]
        style = "font-style:italic;" if italic else ""
        spans = []
        for i, line in enumerate(lines):
            dy = 0 if i == 0 else size * leading
            spans.append(f'<tspan x="{x}" dy="{dy}">{self.esc(line)}</tspan>')
        self.e.append(f'<text x="{x}" y="{y}" text-anchor="{anchor}" '
                      f'font-size="{size}" font-weight="{weight}" fill="{C.get(fill, fill)}" '
                      f'style="{style}">' + ''.join(spans) + '</text>')

    def title(self, title, subtitle=None):
        self.text(35, 43, title, size=29, weight=700)
        if subtitle: self.text(self.w-35, 43, subtitle, size=18, fill="slate", anchor="end")
        self.line(35, 58, self.w-35, 58, stroke="mid", sw=1.4)

    def panel(self, x, y, label, heading):
        self.text(x, y, label, size=27, weight=700)
        self.text(x+32, y, heading, size=25, weight=700)

    def box(self, x, y, w, h, heading, body=(), *, fill="white", stroke="slate",
            dash=None, tag=None, center=False):
        self.rect(x, y, w, h, fill=fill, stroke=stroke, dash=dash)
        ax = x+w/2 if center else x+16
        anch = "middle" if center else "start"
        self.text(ax, y+28, heading, size=21, weight=700, fill=stroke, anchor=anch)
        if body:
            self.text(ax, y+55, body, size=18, anchor=anch, leading=1.25)
        if tag:
            self.pill(x+w-94, y+8, 82, 24, tag, stroke=stroke, size=14)

    def pill(self, x, y, w, h, label, *, stroke="slate", fill="white", size=16):
        self.rect(x, y, w, h, fill=fill, stroke=stroke, sw=1.7, r=h/2)
        self.text(x+w/2, y+h/2+size*.34, label, size=size, weight=700, fill=stroke, anchor="middle")

    def arrow_label(self, x1, y1, x2, y2, label="", *, stroke="ink", dash=None, above=10):
        self.line(x1, y1, x2, y2, stroke=stroke, dash=dash, arrow=True)
        if label:
            self.text((x1+x2)/2, (y1+y2)/2-above, label, size=16, fill=stroke, anchor="middle")

    def callout(self, x, y, w, label, *, stroke="green", fill="white"):
        self.rect(x, y, w, 34, fill=fill, stroke=stroke, sw=1.8, r=17)
        self.text(x+w/2, y+23, label, size=16, weight=700, fill=stroke, anchor="middle")

    def save(self):
        defs = f'''<defs>
          <marker id="arrow" markerWidth="9" markerHeight="9" refX="8" refY="3.5" orient="auto" markerUnits="strokeWidth"><path d="M0,0 L8,3.5 L0,7 Z" fill="{C['ink']}"/></marker>
          <marker id="arrowStart" markerWidth="9" markerHeight="9" refX="1" refY="3.5" orient="auto-start-reverse" markerUnits="strokeWidth"><path d="M8,0 L0,3.5 L8,7 Z" fill="{C['ink']}"/></marker>
          <style>text {{ font-family: Arial, Helvetica, 'Liberation Sans', sans-serif; }} line,path,rect {{ vector-effect: non-scaling-stroke; }}</style>
        </defs>'''
        svg = (f'<svg xmlns="http://www.w3.org/2000/svg" width="{self.wmm}mm" height="{self.hmm}mm" '
               f'viewBox="0 0 {self.w} {self.h}"><rect width="100%" height="100%" fill="#FFFFFF"/>{defs}'
               + ''.join(self.e) + '</svg>\n')
        out = SRC / self.name
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(svg, encoding="utf-8")


def fig1():
    f=Fig("fig01_end_to_end_assurance.svg",180,78); f.title("End-to-end assurance chain", "Implemented solid · target architecture dotted")
    xs=[45,300,565,835,1110,1390]; heads=["Public Prolog policy","Canonical compilation","Local proof search","Parallel Halo2 proving","Authenticated tree package","Verifier decision"]
    subs=[("policy.pl","trusted input"),("typed policy IR","bounded plan"),("certificate tree","outside circuit"),("one proof / node","private witness"),("root · edges · manifest","linked, not aggregated"),("verify proofs + statement","accept / reject")]
    colors=["blue","slate","slate","teal","violet","green"]
    for i,x in enumerate(xs):
        dash="5 7" if i==4 else None
        f.box(x,165,210,112,heads[i],subs[i],fill="pgray" if i in (1,2) else ("pteal" if i==3 else "white"),stroke=colors[i],dash=dash,center=True)
        if i<5: f.arrow_label(x+210,221,xs[i+1]-14,221,stroke="slate")
    f.callout(347,300,160,"Semantics preserved",stroke="green")
    f.callout(856,300,185,"Local relations enforced",stroke="green")
    f.box(1460,292,210,44,"Target: complete statement",(),fill="white",stroke="green",dash="3 7",center=True)
    f.box(45,390,260,105,"Authenticated private facts",("typed fact + salt","prover only"),fill="pteal",stroke="teal",center=True)
    f.arrow_label(305,440,815,250,"private input",stroke="teal",dash="7 6")
    f.rect(395,370,920,150,fill="pgray",stroke="slate",sw=1.6,r=8)
    f.text(415,400,"Complementary circuit assurance",size=20,weight=700,fill="slate")
    f.box(435,425,250,70,"Extract constraints",("gates · selectors · rotations",),fill="white",stroke="slate",center=True)
    f.arrow_label(685,460,760,460,stroke="slate")
    f.box(775,425,250,70,"UCP + finite-field SMT",("targeted residual analysis",),fill="white",stroke="violet",center=True)
    f.arrow_label(1025,460,1100,460,stroke="slate")
    f.box(1115,425,165,70,"Evidence",("not acceptance",),fill="white",stroke="amber",center=True)
    f.path("M 930 370 C 930 330, 935 310, 935 278",stroke="slate",dash="6 5",arrow=True)
    f.text(35,590,"Assurance layers",size=18,weight=700,fill="slate")
    for x,w,label,col in [(185,380,"Semantic assurance","blue"),(575,480,"Circuit assurance","violet"),(1065,660,"Cryptographic verification","green")]:
        f.line(x,615,x+w,615,stroke=col,sw=7); f.text(x+w/2,646,label,size=17,fill=col,anchor="middle")
    f.text(900,735,"No single stage establishes the complete end-to-end claim.",size=21,weight=700,anchor="middle")
    f.save()


def fig2():
    f=Fig("fig02_roles_trust_dataflows.svg",180,75); f.title("Roles, trust anchors, and data flows")
    f.rect(35,90,520,445,fill="pblue",stroke="blue",dash="8 6",r=18); f.text(55,120,"Trusted publication domain",size=19,weight=700,fill="blue")
    f.box(70,160,190,95,"Policy author",("authenticates policy",),fill="white",stroke="blue",center=True)
    f.box(320,160,190,95,"Fact / VC issuer",("controls registry",),fill="white",stroke="blue",center=True)
    f.rect(620,90,480,445,fill="pteal",stroke="teal",dash="8 6",r=18); f.text(640,120,"Untrusted prover boundary",size=19,weight=700,fill="teal")
    f.box(715,175,290,120,"Prover",("may be malicious","holds fact + salt"),fill="white",stroke="teal",center=True)
    f.rect(1165,90,600,445,fill="pblue",stroke="blue",dash="8 6",r=18); f.text(1185,120,"Verifier-controlled boundary",size=19,weight=700,fill="blue")
    f.box(1325,175,280,120,"Verifier",("selects statement","may be privacy-curious"),fill="white",stroke="blue",center=True)
    f.pill(560,132,230,28,"Authenticated policy",stroke="blue",fill="white",size=14)
    f.path("M 165 160 L 165 146 L 560 146",stroke="blue",arrow=True)
    f.path("M 790 146 L 860 146 L 860 175",stroke="blue",arrow=True)
    f.path("M 790 146 L 1465 146 L 1465 175",stroke="blue",arrow=True)
    f.path("M 510 230 C 590 245, 650 260, 715 265",stroke="teal",arrow=True)
    f.text(620,240,"fact + salt",size=16,fill="teal",anchor="middle")
    f.box(285,355,250,78,"Public fact registry",("commitments only",),fill="white",stroke="violet",center=True)
    f.arrow_label(415,255,415,355,"register c_f",stroke="violet")
    f.path("M 535 394 C 790 350, 1110 350, 1325 280",stroke="violet",dash="7 5",arrow=True)
    f.text(940,354,"public registry",size=16,fill="violet",anchor="middle")
    f.arrow_label(1005,210,1325,210,"π + public inputs",stroke="green",above=16)
    f.arrow_label(1005,285,1325,285,"tree manifest",stroke="violet",dash="2 7",above=10)
    f.text(1165,320,"target final architecture",size=14,fill="violet",italic=True,anchor="middle")
    f.box(1270,355,390,78,"Policy-specific verification material",("same compiler + limits → VK_P",),fill="white",stroke="slate",center=True)
    f.arrow_label(1465,355,1465,295,stroke="slate")
    f.text(900,575,"Trust anchors",size=22,weight=700,anchor="middle")
    labels=["Authenticated policy source","Trusted registry / issuer","Compiler + limit profile","VK_P provenance","Public Halo2 parameters"]
    for i,l in enumerate(labels): f.callout(95+i*330,605,290,l,stroke="slate",fill="pgray")
    f.text(900,700,"Issuer authentication is outside the circuit; the verifier never receives the private fact or salt.",size=19,weight=700,anchor="middle")
    f.save()


def fig3():
    f=Fig("fig03_resolution_to_zk.svg",180,70); f.title("From a resolution step to a zero-knowledge relation")
    f.panel(40,105,"a","Resolution certificate")
    f.box(40,135,470,300,"Parent goal",("p(t)",),fill="pgray",stroke="slate")
    f.box(75,220,180,92,"Selected clause",("p(X) ← q₁,…,qₙ",),fill="white",stroke="slate",center=True)
    f.box(285,220,180,92,"Substitution",("θ: X ↦ t",),fill="white",stroke="teal",center=True)
    f.text(275,360,"ordered children  [q₁(t₁), …, qₙ(tₙ)]",size=19,weight=700,anchor="middle")
    f.text(275,410,"Proof search remains outside the circuit.",size=16,fill="slate",italic=True,anchor="middle")
    f.arrow_label(510,280,585,280,stroke="slate")
    f.panel(585,105,"b","Public / private partition")
    f.box(585,135,330,215,"Public statement x",("fact registry","parent commitment","ordered-child commitment","resolution tag · VK_P context"),fill="pblue",stroke="blue")
    f.box(585,370,330,215,"Private witness w",("typed goals · clause","substitution · term data","fact + salt, or","builtin intermediates"),fill="pteal",stroke="teal")
    f.arrow_label(915,280,1000,280,stroke="slate")
    f.panel(1000,105,"c","Halo2 relation")
    f.rect(1000,135,755,450,fill="pgray",stroke="slate",r=16)
    f.text(1377,205,"∃ w : R_step(x, w) = 1",size=33,weight=700,anchor="middle")
    for i,(lab,col) in enumerate([("valid rule","slate"),("valid fact","violet"),("valid builtin","amber")]):
        f.box(1050+i*225,260,190,80,lab,(),fill="white",stroke=col,center=True)
    f.text(1375,382,"exactly one branch",size=18,weight=700,anchor="middle")
    f.arrow_label(1375,405,1375,475,stroke="green")
    f.box(1275,475,200,70,"Local proof π",("w remains hidden",),fill="white",stroke="green",center=True)
    f.text(900,660,"The circuit validates a bounded certificate; it does not execute Prolog search.",size=20,weight=700,anchor="middle")
    f.save()


def fig4():
    f=Fig("fig04_semantics_preserving_compilation.svg",180,72); f.title("Semantics-preserving policy compilation", "Shared compiler and limit profile")
    xs=[35,225,415,610,820,1040,1280,1510]
    heads=["policy.pl","ANTLR parser","Parse tree","Semantic frontend","rules.json v2","Bound validation","Circuit plan","PK_P / VK_P"]
    colors=["blue","slate","slate","slate","blue","amber","violet","green"]
    widths=[150,155,155,175,180,200,190,235]
    for i,(x,h,w,col) in enumerate(zip(xs,heads,widths,colors)):
        f.box(x,135,w,82,h,(),fill="white",stroke=col,center=True)
        if i<len(xs)-1: f.arrow_label(x+w,176,xs[i+1]-10,176,stroke="slate")
    f.callout(1070,245,140,"within bounds",stroke="green")
    f.arrow_label(1140,217,1140,245,stroke="green")
    f.box(1045,315,190,62,"Reject",("unsupported / oversized",),fill="pred",stroke="red",center=True)
    f.arrow_label(1090,217,1090,315,"outside bounds",stroke="red")
    f.rect(35,420,1195,165,fill="pgray",stroke="slate",r=10)
    f.text(55,452,"Preserved meaning",size=20,weight=700,fill="slate")
    items=["name + arity","clause order","local variables","typed literals","compound / lists","operator tree","child order","equality paths"]
    for i,it in enumerate(items):
        x=55+(i%4)*285; y=480+(i//4)*48
        f.callout(x,y,250,"✓  "+it,stroke="slate",fill="white")
    f.rect(1270,400,495,205,fill="pviolet",stroke="violet",r=12)
    f.text(1517,435,"Consistent policy-specific derivation",size=20,weight=700,fill="violet",anchor="middle")
    f.box(1310,470,180,82,"Prover",("plan → PK_P",),fill="white",stroke="teal",center=True)
    f.box(1540,470,180,82,"Verifier",("plan → VK_P",),fill="white",stroke="blue",center=True)
    f.line(1515,448,1515,575,stroke="violet",dash="6 5")
    f.text(900,675,"Current artifacts also expose an explicit policy commitment; policy-specific keys bind the circuit shape.",size=18,fill="slate",anchor="middle")
    f.save()


def fig5():
    f=Fig("fig05_typed_prolog_terms.svg",89,72); f.title("Canonical typed Prolog terms")
    f.panel(30,95,"a","Distinct term kinds")
    kinds=[("Atom","\"1\"","blue"),("Integer","1","amber"),("Nil","[]","slate"),("Cons","head · tail","teal"),("Compound","functor · args","violet")]
    for i,(k,v,col) in enumerate(kinds):
        x=30+(i%3)*280; y=125+(i//3)*85
        f.box(x,y,245,66,k,(v,),fill="white",stroke=col,center=True)
    f.text(430,315,"Atom(\"1\")  ≠  Integer(1)",size=24,weight=700,fill="red",anchor="middle")
    f.panel(30,375,"b","Nested typed tree: record(user, [active, 3])")
    f.box(310,405,270,55,"Compound · record/2",(),fill="pviolet",stroke="violet",center=True)
    f.line(380,460,205,500,stroke="slate"); f.line(510,460,650,500,stroke="slate")
    f.box(105,500,200,52,"Atom · user",(),fill="pblue",stroke="blue",center=True)
    f.box(545,500,210,52,"Cons",(),fill="pteal",stroke="teal",center=True)
    f.line(600,552,500,588,stroke="slate"); f.line(700,552,760,588,stroke="slate")
    f.box(410,588,180,48,"Atom · active",(),fill="pblue",stroke="blue",center=True)
    f.box(680,588,155,48,"Cons",(),fill="pteal",stroke="teal",center=True)
    f.line(730,636,650,665,stroke="slate"); f.line(790,636,835,665,stroke="slate")
    f.box(575,665,145,42,"Integer · 3",(),fill="pamber",stroke="amber",center=True)
    f.box(775,665,90,42,"Nil",(),fill="pgray",stroke="slate",center=True)
    f.save()


def fig6():
    f=Fig("fig06_fact_issuance_commitment_zk.svg",180,82); f.title("Authenticated fact issuance, commitment, and zero-knowledge use")
    f.panel(35,105,"a","Issuance")
    f.rect(35,130,810,560,fill="pgray",stroke="slate",r=14)
    f.box(70,170,210,85,"Fact / VC issuer",("trusted registry writer",),fill="white",stroke="blue",center=True)
    f.arrow_label(280,212,345,212,stroke="slate")
    f.box(345,170,220,85,"Typed goal encoding",("canonical fact digest",),fill="white",stroke="slate",center=True)
    f.box(345,300,220,75,"Random salt s",("private · high entropy",),fill="pteal",stroke="teal",center=True)
    f.line(455,255,455,300,stroke="teal",arrow=True)
    f.arrow_label(565,212,640,275,stroke="violet")
    f.arrow_label(565,337,640,300,stroke="violet")
    f.box(640,230,165,118,"Poseidon",("c_f = H(domain,","version, digest, s)"),fill="pviolet",stroke="violet",center=True)
    f.arrow_label(722,348,722,440,"public c_f",stroke="violet")
    f.box(600,440,245,82,"Authenticated registry",("issuer-controlled",),fill="pblue",stroke="blue",center=True)
    f.path("M 650 348 C 550 430, 360 455, 235 455",stroke="teal",arrow=True)
    f.box(70,415,270,82,"Holder / prover",("private fact + salt",),fill="pteal",stroke="teal",center=True)
    f.text(440,630,"Issuer authentication / PKI is outside the circuit.",size=18,fill="slate",italic=True,anchor="middle")
    f.panel(900,105,"b","Zero-knowledge use")
    f.rect(900,130,865,560,fill="white",stroke="slate",r=14)
    f.box(930,180,220,100,"Private witness w",("typed fact","salt s"),fill="pteal",stroke="teal",center=True)
    f.box(930,430,220,100,"Public instance x",("33 registry slots",),fill="pblue",stroke="blue",center=True)
    f.rect(1210,170,330,370,fill="pgray",stroke="violet",r=16)
    f.text(1375,205,"Inside Halo2",size=22,weight=700,fill="violet",anchor="middle")
    steps=["1  Hash typed goal","2  Recompute c_f","3  Registry membership","4  Fact-resolution semantics"]
    for i,s in enumerate(steps): f.box(1240,230+i*68,270,48,s,(),fill="white",stroke="slate",center=True)
    f.arrow_label(1150,230,1210,260,stroke="teal")
    f.arrow_label(1150,480,1210,445,stroke="blue")
    f.arrow_label(1540,355,1610,355,stroke="green")
    f.box(1610,310,125,90,"Local π",("fact hidden",),fill="white",stroke="green",center=True)
    f.line(930,590,1735,590,stroke="red",dash="8 6")
    f.text(1332,625,"Verifier receives neither the fact nor the salt",size=19,weight=700,fill="red",anchor="middle")
    f.text(900,770,"The commitment authorizes a typed fact; it is not encryption.",size=18,fill="slate",anchor="middle")
    f.save()


def fig7():
    f=Fig("fig07_local_resolution_check.svg",180,68); f.title("Checking one proof-tree step")
    f.box(35,160,270,260,"Local node",("public: parent + children","public: resolution tag","private: certificate data"),fill="pgray",stroke="slate",center=True)
    f.arrow_label(305,290,385,290,stroke="slate")
    branches=[("Valid policy rule?",("real clause candidate","pattern / signature","variable consistency"),"slate","r"),("Authorized fact?",("typed digest","registry membership"),"violet","f"),("Correct builtin?",("operator semantics","arithmetic / comparison","range constraints"),"amber","b")]
    for i,(h,b,col,var) in enumerate(branches):
        y=100+i*155; f.box(385,y,440,120,h,b,fill="white",stroke=col)
        f.pill(765,y+78,42,30,var,stroke=col,size=18)
        f.arrow_label(825,y+60,925,290,stroke=col)
    f.rect(925,200,350,180,fill="pgray",stroke="slate",r=12)
    f.text(1100,235,"Exclusive resolution",size=22,weight=700,anchor="middle")
    f.text(1100,285,"r, f, b ∈ {0, 1}",size=25,weight=700,anchor="middle")
    f.text(1100,330,"r + f + b = 1",size=29,weight=700,fill="green",anchor="middle")
    f.box(955,420,290,72,"Bind public tag",("tag = r + 2f + 3b",),fill="pblue",stroke="blue",center=True)
    f.arrow_label(1100,380,1100,420,stroke="blue")
    f.arrow_label(1275,290,1375,290,stroke="green")
    f.box(1375,225,350,130,"Local step accepted",("output: independent proof π",),fill="white",stroke="green",center=True)
    f.text(900,625,"Exactly one implemented semantic branch must validate the same committed local statement.",size=20,weight=700,anchor="middle")
    f.save()


def fig8():
    f=Fig("fig08_proof_tree_composition.svg",180,88); f.title("Authenticated composition of independent local proofs", "Not recursive aggregation")
    f.panel(35,100,"a","Parallel local proving")
    nodes=[(300,150,"π₀"),(130,330,"π₁"),(470,330,"π₂"),(330,515,"π₃"),(650,515,"π₄")]
    edges=[(300,230,190,330),(300,230,530,330),(530,410,390,515),(530,410,710,515)]
    for x1,y1,x2,y2 in edges:
        f.line(x1,y1,x2,y2,stroke="violet",sw=3)
        f.text((x1+x2)/2,(y1+y2)/2-10,"C_child = C_parent",size=12,fill="violet",anchor="middle")
    for x,y,label in nodes:
        f.box(x-90,y,180,80,label,("C_parent · C_children","resolution tag"),fill="white",stroke="green",center=True)
    f.text(70,700,"Sibling lanes",size=16,fill="teal")
    for y in (730,755,780): f.line(180,y,690,y,stroke="teal",dash="8 6",arrow=True)
    f.panel(790,100,"b","Tree manifest and verification")
    f.box(790,140,390,295,"Tree manifest",("trusted root commitment","exact proof count","ordered topology","leaf positions","deterministic proof order"),fill="pviolet",stroke="violet",dash="3 7")
    f.text(985,465,"Target final architecture",size=16,fill="violet",italic=True,anchor="middle")
    f.box(1250,140,500,295,"Verifier",("1  Verify every local proof","2  Check every parent–child edge","3  Check root, count, order, topology"),fill="pblue",stroke="blue",dash="3 7")
    f.arrow_label(1180,285,1250,285,stroke="violet",dash="3 7")
    f.text(1500,465,"Local proof verification is implemented; complete orchestration is pending.",size=15,fill="slate",anchor="middle")
    attacks=[("Omitted child","missing edge"),("Duplicated proof","reused node"),("Reordered child","wrong position")]
    for i,(h,b) in enumerate(attacks):
        x=830+i*300; f.box(x,555,250,88,"×  "+h,(b,),fill="pred",stroke="red",dash="8 5",center=True)
    f.rect(790,700,960,70,fill="pgray",stroke="slate",r=8)
    f.text(1270,744,"Linked independent proofs — not recursive proof aggregation",size=22,weight=700,anchor="middle")
    f.save()


def fig9():
    f=Fig("fig09_static_assurance_pipeline.svg",180,82); f.title("Static assurance pipeline: structural analysis, UCP, and finite-field SMT")
    f.box(35,165,220,115,"Halo2 circuit",("semantic targets","public/private labels"),fill="pgray",stroke="slate",center=True)
    f.arrow_label(255,222,330,222,stroke="slate",dash="7 5")
    f.box(330,125,300,195,"Constraint extraction",("gates · selectors · rotations","fixed / advice / instance","copy constraints","lookups when supported"),fill="pgray",stroke="slate")
    f.arrow_label(630,175,715,175,stroke="slate")
    f.arrow_label(630,270,715,410,stroke="slate")
    f.box(715,105,330,150,"Structural checks",("unused gate / column","unconstrained cell","inactive selector"),fill="white",stroke="amber")
    f.box(715,340,330,150,"UCP determinacy",("seed public instance","propagate constraints","learn determined cells"),fill="white",stroke="violet")
    f.path("M 1010 340 C 1090 300, 1090 520, 875 520 C 735 520, 735 490, 760 485",stroke="violet",dash="7 5",arrow=True)
    f.text(910,550,"learned cells feed back",size=16,fill="violet",italic=True,anchor="middle")
    f.arrow_label(1045,415,1140,415,"residual targets only",stroke="violet")
    f.box(1140,340,300,150,"cvc5 finite-field SMT",("solve over F_p","targeted queries"),fill="pviolet",stroke="violet",center=True)
    f.arrow_label(1045,180,1510,180,stroke="amber")
    f.arrow_label(1440,415,1510,415,stroke="violet")
    f.rect(1510,105,255,485,fill="pgray",stroke="slate",r=12)
    f.text(1637,140,"Evidence classes",size=21,weight=700,anchor="middle")
    outs=[("Determined target","green"),("Alternative witness","red"),("Algebraic recovery","red"),("Inconclusive / timeout","amber")]
    for i,(lab,col) in enumerate(outs):
        f.rect(1535,175+i*90,205,62,fill="white",stroke=col,r=12)
        f.text(1637,213+i*90,lab,size=17 if i==3 else 19,weight=700,fill=col,anchor="middle")
    f.box(1140,125,300,85,"Opaque boundary",("hash / commitment if modeled",),fill="white",stroke="slate",dash="3 7",center=True)
    f.text(900,655,"Batchman adapter and study",size=17,fill="violet",italic=True,anchor="middle")
    f.line(630,630,1170,630,stroke="violet",dash="2 7",sw=3)
    f.rect(35,705,1730,62,fill="pamber",stroke="amber",r=8)
    f.text(900,744,"Constraint-level evidence; not a proof of whole-system correctness or general computational recoverability.",size=19,weight=700,anchor="middle")
    f.save()


def fig10():
    f=Fig("fig10_results.svg",180,116); f.title("Security, analyzer effectiveness, and scalability results", "INTERNAL DRAFT — measurements required")
    panels=[(35,95,845,465,"a","Seeded-defect detection"),(920,95,845,465,"b","Proof-tree scaling"),(35,600,845,465,"c","Parallel speedup"),(920,600,845,465,"d","Analyzer cost")]
    for x,y,w,h,l,t in panels:
        f.rect(x,y,w,h,fill="pgray",stroke="slate",dash="8 6",r=12); f.panel(x+15,y+35,l,t)
    rows=["Rule selector","Variable equality","Pattern binding","Fact membership","Builtin relation","Public binding","Tree edge"]
    cols=["Tests","Struct.","UCP","SMT"]
    for j,c in enumerate(cols): f.text(510+j*80,165,c,size=14,weight=700,anchor="middle")
    for i,r in enumerate(rows):
        f.text(80,205+i*38,r,size=15)
        for j in range(4): f.rect(485+j*80,185+i*38,50,27,fill="white",stroke="mid",sw=1,r=2)
    f.text(457,505,"Populate from seeded-defect campaign",size=17,fill="amber",italic=True,anchor="middle")
    def axes(x,y,w,h,xlab,ylab):
        f.line(x,y,x,y-h,stroke="ink"); f.line(x,y,x+w,y,stroke="ink")
        f.text(x+w/2,y+38,xlab,size=16,anchor="middle"); f.text(x-25,y-h/2,ylab,size=16,anchor="middle")
    axes(1010,485,650,270,"Tree size / shape","Proving time")
    f.text(1335,340,"No reproducible measurements found",size=19,fill="amber",weight=700,anchor="middle")
    axes(125,995,650,270,"Workers","Speedup")
    f.line(125,995,775,725,stroke="mid",dash="7 5")
    f.text(450,845,"Ideal reference only — data absent",size=19,fill="amber",weight=700,anchor="middle")
    axes(1010,995,650,270,"Circuit / target size","Runtime")
    f.text(1335,835,("Structural · UCP · SMT","No benchmark series available"),size=19,fill="amber",weight=700,anchor="middle")
    f.rect(35,1090,1730,45,fill="pamber",stroke="amber",r=6)
    f.text(900,1119,"This preview contains schemas and layout only; it must not be used as a manuscript results figure.",size=17,weight=700,anchor="middle")
    f.save()


def palette():
    f=Fig("../style/palette.svg",180,32); f.title("Figure palette and line semantics")
    labels=[("Public / trusted","blue"),("Private / witness","teal"),("Static structure","slate"),("Commitment","violet"),("Valid","green"),("Warning","amber"),("Rejected","red")]
    for i,(lab,col) in enumerate(labels):
        x=35+i*245; f.rect(x,90,36,36,fill=col,stroke=col,r=4); f.text(x+48,115,lab,size=17)
    f.line(35,205,210,205,stroke="ink"); f.text(220,211,"implemented",size=17)
    f.line(430,205,605,205,stroke="ink",dash="8 6"); f.text(615,211,"metadata / offline",size=17)
    f.line(900,205,1075,205,stroke="ink",dash="2 7"); f.text(1085,211,"optional / proposed",size=17)
    f.line(1360,205,1535,205,stroke="red",dash="8 6"); f.text(1545,211,"rejected flow",size=17)
    f.save()


def render():
    # Use the installed C libraries directly.  This avoids depending on the
    # optional gi._gi_cairo Python bridge while retaining native vector output.
    import ctypes
    from ctypes import c_char_p, c_double, c_int, c_void_p, POINTER, Structure
    class Dim(Structure):
        _fields_=[("width",c_int),("height",c_int),("em",c_double),("ex",c_double)]
    rsvg=ctypes.CDLL("librsvg-2.so.2"); cairo=ctypes.CDLL("libcairo.so.2"); gobj=ctypes.CDLL("libgobject-2.0.so.0")
    rsvg.rsvg_handle_new_from_file.argtypes=[c_char_p,POINTER(c_void_p)]; rsvg.rsvg_handle_new_from_file.restype=c_void_p
    rsvg.rsvg_handle_get_dimensions.argtypes=[c_void_p,POINTER(Dim)]
    rsvg.rsvg_handle_render_cairo.argtypes=[c_void_p,c_void_p]; rsvg.rsvg_handle_render_cairo.restype=c_int
    cairo.cairo_pdf_surface_create.argtypes=[c_char_p,c_double,c_double]; cairo.cairo_pdf_surface_create.restype=c_void_p
    cairo.cairo_image_surface_create.argtypes=[c_int,c_int,c_int]; cairo.cairo_image_surface_create.restype=c_void_p
    cairo.cairo_create.argtypes=[c_void_p]; cairo.cairo_create.restype=c_void_p
    cairo.cairo_scale.argtypes=[c_void_p,c_double,c_double]
    cairo.cairo_surface_write_to_png.argtypes=[c_void_p,c_char_p]; cairo.cairo_surface_write_to_png.restype=c_int
    cairo.cairo_surface_finish.argtypes=[c_void_p]; cairo.cairo_surface_destroy.argtypes=[c_void_p]; cairo.cairo_destroy.argtypes=[c_void_p]
    gobj.g_object_unref.argtypes=[c_void_p]
    PDF.mkdir(parents=True,exist_ok=True); PNG.mkdir(parents=True,exist_ok=True)
    for svg in sorted(SRC.glob("fig*.svg")):
        import re
        root_text=svg.read_text(encoding="utf-8")[:300]
        m=re.search(r'width="([0-9.]+)mm" height="([0-9.]+)mm"',root_text)
        if not m: raise RuntimeError(f"missing physical SVG dimensions: {svg}")
        width_mm,height_mm=map(float,m.groups())
        err=c_void_p(); handle=rsvg.rsvg_handle_new_from_file(str(svg).encode(),ctypes.byref(err))
        if not handle: raise RuntimeError(f"librsvg could not open {svg}")
        dim=Dim(); rsvg.rsvg_handle_get_dimensions(handle,ctypes.byref(dim))
        pdf=PDF/(svg.stem+".pdf")
        pdf_w,pdf_h=width_mm/25.4*72,height_mm/25.4*72
        pdf_scale=pdf_w/dim.width
        surf=cairo.cairo_pdf_surface_create(str(pdf).encode(),pdf_w,pdf_h); ctx=cairo.cairo_create(surf)
        cairo.cairo_scale(ctx,pdf_scale,pdf_scale); rsvg.rsvg_handle_render_cairo(handle,ctx)
        cairo.cairo_destroy(ctx); cairo.cairo_surface_finish(surf); cairo.cairo_surface_destroy(surf)
        # 600 dpi relative to SVG's nominal 96 dpi.
        png_w,png_h=round(width_mm/25.4*600),round(height_mm/25.4*600)
        scale=png_w/dim.width
        png=PNG/(svg.stem+".png")
        surf=cairo.cairo_image_surface_create(0,png_w,png_h); ctx=cairo.cairo_create(surf)
        cairo.cairo_scale(ctx,scale,scale); rsvg.rsvg_handle_render_cairo(handle,ctx)
        cairo.cairo_surface_write_to_png(surf,str(png).encode()); cairo.cairo_destroy(ctx); cairo.cairo_surface_destroy(surf)
        gobj.g_object_unref(handle)


def contact_sheet():
    from PIL import Image, ImageDraw, ImageFont
    imgs=[]
    for p in sorted(PNG.glob("fig*.png")):
        im=Image.open(p).convert("RGB"); im.thumbnail((1000,650)); imgs.append((p.stem,im.copy()))
    if not imgs:return
    sheet=Image.new("RGB",(2140, math.ceil(len(imgs)/2)*760),(255,255,255)); d=ImageDraw.Draw(sheet)
    for i,(name,im) in enumerate(imgs):
        x=40+(i%2)*1060; y=40+(i//2)*760; d.text((x,y),name,fill=C["ink"]); sheet.paste(im,(x,y+35))
    sheet.save(PNG/"contact_sheet.png",dpi=(300,300))
    sheet.convert("L").save(PNG/"contact_sheet_grayscale.png",dpi=(300,300))


if __name__ == "__main__":
    for fn in (fig1,fig2,fig3,fig4,fig5,fig6,fig7,fig8,fig9,fig10,palette): fn()
    render(); contact_sheet()
    print(f"Generated figures under {ROOT}")
