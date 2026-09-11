#!/usr/bin/env sh
set -eu

cd "$(dirname "$0")"

java -jar antlr4-4.13.3-SNAPSHOT-complete.jar \
    -Dlanguage=Rust \
    -visitor \
    -listener \
    -Xexact-output-dir \
    -o src/parser \
    prolog.g4

# ANTLR's interpreter/token metadata is not consumed by the Rust frontend.
rm -f src/parser/prolog.interp src/parser/prolog.tokens \
    src/parser/prologLexer.interp src/parser/prologLexer.tokens
