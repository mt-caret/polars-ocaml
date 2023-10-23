open Core

(** [Naive_datetime.t] is an *unzoned* date and time type used by polars.

    The underlying Rust type is chrono::naive::NaiveDateTime
    (https://docs.rs/chrono/latest/chrono/naive/struct.NaiveDateTime.html). *)
type t

val of_naive_date : ?hour:int -> ?min:int -> ?sec:int -> Naive_date.t -> t
val of_date : ?hour:int -> ?min:int -> ?sec:int -> Date.t -> t
val to_string : t -> string
val of_string : string -> t
val of_time_ns : Time_ns.t -> t option
val of_time_ns_exn : Time_ns.t -> t
val to_time_ns : t -> Time_ns.t

module For_testing : sig
  val round_to_time_unit : t -> time_unit:Time_unit.t -> t
end
