# Recommended development setup

For OCaml:

```
$ opam switch create 4.14.1
$ opam install dune ocamlformat
$ opam install . --deps-only --with-doc --with-test
$ dune build @fmt @runtest @doc -w --auto-promote
```

For Rust:

```
$ cargo install cargo-watch
$ UPDATE_EXPECT=1 cargo watch -x check -x test -x doc -x clippy
```

The `UPDATE_EXPECT` environment variable is similar to dune's `--auto-promote`
flag in that it automatically updates expect tests on the Rust side.

Running benchmarks:

```
$ ./bench/runner.sh dataframe_builders_bench
```

It is strongly recommended that you set up the mold linker, since builds
often are bottlenecked on very long link times:

1. Follow https://github.com/rui314/mold#how-to-use to configure the linker for Rust
2. add `(library_flags -ccopt -fuse-ld=mold)` to the `library` stanza in various dune files