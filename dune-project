(lang dune 3.8)

(name polars_ocaml)

(generate_opam_files true)

(source
 (github mt-caret/polars-ocaml))

(authors "mtakeda")

(maintainers "mtakeda")

(license LICENSE)

(documentation https://url/to/documentation)

(using mdx 0.4)

(package
 (name polars_ocaml)
 (synopsis "OCaml bindings to the Polars dataframe library")
 (description "OCaml bindings to the Polars dataframe library")
 (depends ocaml dune)
 (tags
  (topics "to describe" your project)))
