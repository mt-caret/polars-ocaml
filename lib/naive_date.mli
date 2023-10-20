open Core

(** [Naive_date.t] is an *unzoned* date type used by polars.

    The underlying Rust type is chrono::naive::NaiveDate
    (https://docs.rs/chrono/latest/chrono/naive/struct.NaiveDate.html). *)
type t

val create : year:int -> month:int -> day:int -> t option
val of_date : Date.t -> t
val to_date_exn : t -> Date.t
val of_string : string -> t
