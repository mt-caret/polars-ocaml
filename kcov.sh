#!/bin/bash
set -euxo pipefail

# We can't simply run `kcov dune runtest` since kcov seems to be unable to
# locate the source files when running tests via dune. Instad, we manually
# inspected the test commands via `dune runtest --force --verbose
# --always-show-command-line` and manually run them individually.
#
# We want to still run `dune runtest` separately to make sure the inline test
# runners properly exist.
dune runtest

KCOV_DIR="/tmp/kcov"
rm -rf "$KCOV_DIR"

KCOV="kcov --include-pattern=polars-ocaml $KCOV_DIR"

(cd _build/default/async && $KCOV .polars_async.inline-tests/inline_test_runner_polars_async.exe inline-test-runner polars_async -source-tree-root .. -diff-cmd -)
(cd _build/default/guide && $KCOV .polars_guide.inline-tests/inline_test_runner_polars_guide.exe inline-test-runner polars_guide -source-tree-root .. -diff-cmd -)
(cd _build/default/lib && $KCOV .polars.inline-tests/inline_test_runner_polars.exe inline-test-runner polars -source-tree-root .. -diff-cmd -)
(cd _build/default/test && $KCOV .polars_tests.inline-tests/inline_test_runner_polars_tests.exe inline-test-runner polars_tests -source-tree-root .. -diff-cmd -)