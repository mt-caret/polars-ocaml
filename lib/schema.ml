open! Core

type t

external create : (string * Data_type.t) list -> t = "rust_schema_create"
external to_fields : t -> (string * Data_type.t) list = "rust_schema_to_fields"

let sexp_of_t t = to_fields t |> [%sexp_of: (string * Data_type.t) list]
