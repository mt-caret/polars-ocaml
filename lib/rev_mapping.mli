open! Core

type t [@@deriving sexp_of]

val quickcheck_shrinker : t Quickcheck.Shrinker.t
val quickcheck_observer : t Quickcheck.Observer.t
val get_categories : t -> string list
