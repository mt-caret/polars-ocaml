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
