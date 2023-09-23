#!/bin/bash
set -exo pipefail

# We can't simply run `kcov dune runtest` since kcov seems to be unable to
# properly track execution of the individual test executables. Instad, we
# manually inspected the test commands via `dune runtest --force --verbose
# --always-show-command-line` and run each executable outside of dune.

KCOV_DIR=/tmp/kcov
KCOV_DIRS=$(echo /tmp/{kcov_polars_async,kcov_polars_guide,kcov_polars,kcov_polars_tests})
rm -rf $KCOV_DIR $KCOV_DIRS

KCOV="kcov --include-pattern=polars-ocaml --exclude-pattern=ml-gen,test,guide --replace-src-path=/workspace_root:./"

if [ "$CI" = "true" ]; then
    # AFAICT there are some resource limits on GitHub Actions which prevent
    # parallelization of code coverage collection, so we run these sequentially.
    (cd _build/default && $KCOV /tmp/kcov_polars_async async/.polars_async.inline-tests/inline_test_runner_polars_async.exe inline-test-runner polars_async -source-tree-root . -diff-cmd -)
    (cd _build/default && $KCOV /tmp/kcov_polars_guide guide/.polars_guide.inline-tests/inline_test_runner_polars_guide.exe inline-test-runner polars_guide -source-tree-root . -diff-cmd -)
    (cd _build/default && $KCOV /tmp/kcov_polars lib/.polars.inline-tests/inline_test_runner_polars.exe inline-test-runner polars -source-tree-root . -diff-cmd -)
    (cd _build/default && $KCOV /tmp/kcov_polars_tests test/.polars_tests.inline-tests/inline_test_runner_polars_tests.exe inline-test-runner polars_tests -source-tree-root . -diff-cmd -)
else
    (cd _build/default && $KCOV /tmp/kcov_polars_async async/.polars_async.inline-tests/inline_test_runner_polars_async.exe inline-test-runner polars_async -source-tree-root . -diff-cmd -) &
    (cd _build/default && $KCOV /tmp/kcov_polars_guide guide/.polars_guide.inline-tests/inline_test_runner_polars_guide.exe inline-test-runner polars_guide -source-tree-root . -diff-cmd -) &
    (cd _build/default && $KCOV /tmp/kcov_polars lib/.polars.inline-tests/inline_test_runner_polars.exe inline-test-runner polars -source-tree-root . -diff-cmd -) &
    (cd _build/default && $KCOV /tmp/kcov_polars_tests test/.polars_tests.inline-tests/inline_test_runner_polars_tests.exe inline-test-runner polars_tests -source-tree-root . -diff-cmd -) &
    wait
fi

kcov --merge $KCOV_DIR $KCOV_DIRS
