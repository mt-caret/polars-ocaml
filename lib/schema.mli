open! Core

type t [@@deriving sexp]

external create : (string * Data_type.t) list -> t = "rust_schema_create"
external to_fields : t -> (string * Data_type.t) list = "rust_schema_to_fields"
