# Recommended development setup

```
$ opam switch create 4.14.1
$ opam install . --deps-only --with-doc --with-test
$ dune build @fmt @runtest @doc -w --auto-promote
```