(lang dune 3.7)

(name polars)

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
 (name polars)
 (synopsis "OCaml bindings to the Polars dataframe library")
 (description "OCaml bindings to the Polars dataframe library")
 (depends
  (expect_test_helpers_core :with-test)
  (ocamlformat :dev)
  (odoc :with-doc)
  (shell :with-test)
  core
  core_kernel
  dune
  ocaml
  ppx_jane)
 (tags
  (data-science polars rust)))


(package
 (name polars_async)
 (synopsis "Async OCaml bindings to the Polars dataframe library")
 (description "Async OCaml bindings to the Polars dataframe library")
 (depends
  polars
  async
  core)
 (tags
  (data-science polars rust)))
