#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../.."
python3 paper/figures/scripts/generate_figures.py
python3 paper/figures/scripts/validate_figures.py
