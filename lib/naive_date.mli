open Core

(** [Naive_date.t] is polars' native representation of a date. *)

type t

val create : year:int -> month:int -> day:int -> t option
val of_date : Date.t -> t
val to_date_exn : t -> Date.t
val of_string : string -> t
