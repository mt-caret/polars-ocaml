# polars-ocaml-macros

A crate for housing the `ocaml_interop_export` macro used in polars-ocaml.
Previously we used `export_ocaml!` provided by `ocaml_interop`, but we moved
away from it since rustfmt cannot format the code passed to it.