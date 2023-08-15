open! Core

type t [@@deriving sexp]

val create : (string * Data_type.t) list -> t
val to_fields : t -> (string * Data_type.t) list
