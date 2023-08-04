(lang dune 3.7)

(name polars_ocaml)

(generate_opam_files true)

(source
 (github mt-caret/polars-ocaml))

(authors "mtakeda")

(maintainers "mtakeda")

(license MIT)

; TODO: update below link once we start generating documentation
; (https://github.com/mt-caret/polars-ocaml/issues/5)

(documentation https://github.com/mt-caret/polars-ocaml)

(package
 (name polars_ocaml)
 (synopsis "OCaml bindings to the Polars dataframe library")
 (description "OCaml bindings to the Polars dataframe library")
 (depends ocaml dune)
 (tags
  (data-science polars rust)))
