# Recommended development setup

```
$ opam switch create 4.14.1
$ opam install . --deps-only --with-doc --with-test
$ dune build @fmt @runtest @doc -w --auto-promote
```

# Building a 32bit binary

```
$ rustup target add i686-unknown-linux-gnu
$ opam switch create 4.14.1+32bit ocaml-variants.4.14.1+options ocaml-option-32bit
$ sed -i "s/cargo build/cargo build --target i686-unknown-linux-gnu/" lib/dune
$ sed -i 's/rust\/target/rust\/target\/i686-unknown-linux-gnu/' lib/dune
$ dune build @fmt @runtest @doc -w --auto-promote
```