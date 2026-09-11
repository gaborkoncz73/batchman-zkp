#!/usr/bin/env python3
"""Regenerate Figure 10 only after validating that evaluation CSVs have data.

The current checked-in output is intentionally a data-required draft.  This
script refuses to imply results while any required table is header-only.
"""
from pathlib import Path
import csv, sys

HERE=Path(__file__).resolve().parent
DATA=HERE.parent/"data"
files=["seeded_defects.csv","proof_tree_scaling.csv","parallel_speedup.csv","analyzer_cost.csv"]
missing=[]
for name in files:
    with (DATA/name).open(newline="",encoding="utf-8") as fh:
        if not list(csv.DictReader(fh)): missing.append(name)
if missing:
    print("Figure 10 remains an internal draft; no fabricated plot was generated.")
    print("Header-only datasets: "+", ".join(missing))
    sys.exit(2)

raise SystemExit("Data are present, but statistical plotting must be implemented with the preregistered aggregation method before publication.")
