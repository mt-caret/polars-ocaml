(lang dune 3.8)

(name polars)

(generate_opam_files true)

(source
 (github mt-caret/polars-ocaml))

(authors mtakeda)

(maintainers mtakeda)

(license MIT)

(documentation https://github.com/mt-caret/polars-ocaml)

(using mdx 0.4)

(package
 (name polars)
 (synopsis "OCaml bindings to the Polars dataframe library")
 (description "OCaml bindings to the Polars dataframe library")
 (depends
  (core_bench :with-test)
  (expect_test_helpers_core :with-test)
  (mdx :with-test)
  (ocaml
   (< 5.0.0))
  (ocamlformat :dev)
  (odoc :with-doc)
  (re2 :with-test)
  (shell :with-test)
  conf-rust-2021
  core
  core_kernel
  core_unix
  dune
  ; ocaml-interop does not yet properly support OCaml 5
  ppx_jane
  ppx_typed_fields)
 (tags
  (data-science polars rust)))

(package
 (name polars_async)
 (synopsis "Async OCaml bindings to the Polars dataframe library")
 (description "Async OCaml bindings to the Polars dataframe library")
 (depends
  (core_bench :with-test)
  (expect_test_helpers_core :with-test)
  (mdx :with-test)
  (ocamlformat :dev)
  (odoc :with-doc)
  (re2 :with-test)
  (shell :with-test)
  async
  core
  polars
  ppx_jane)
 (tags
  (data-science polars rust)))
