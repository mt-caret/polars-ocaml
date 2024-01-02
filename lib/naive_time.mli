open! Core

(** [Naive_time.t] is a type representing the time of day used by polars.

    The underlying Rust type is chrono::NaiveTime
    (https://docs.rs/chrono/latest/chrono/naive/struct.NaiveTime.html). *)
type t

(** Returns [None] if [Naive_date.t] does not support the [Time_ns.Ofday.t] value.
    One example is [Time_ns.Ofday.of_string "24:00:00"]. *)
val of_ofday : Time_ns.Ofday.t -> t option

val of_ofday_exn : Time_ns.Ofday.t -> t
val to_ofday : t -> Time_ns.Ofday.t
val to_string : t -> string
val pp : Format.formatter -> t -> unit [@@ocaml.toplevel_printer]
