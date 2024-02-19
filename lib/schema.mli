open! Core

type t [@@deriving sexp_of]

val create : (string * Data_type.t) list -> t
val to_fields : t -> (string * Data_type.t) list
