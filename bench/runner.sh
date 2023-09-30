#!/usr/bin/env sh
export BENCHMARKS_RUNNER=TRUE
case "$1" in
  "dataframe_builders_bench" ) test="dataframe_builders_bench"; main="dataframe_builders_bench_main";;
esac
shift;
export BENCH_LIB="$test"
exec dune exec --release -- "./bench/$main.exe" -fork -run-without-cross-library-inlining "$@"