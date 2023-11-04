open! Core

(** [Duration.t] is a timespan type used by polars.

    The underlying Rust type is chrono::Duration
    (https://docs.rs/chrono/latest/chrono/struct.Duration.html). *)
type t

val of_span : Time_ns.Span.t -> t
val to_span : t -> Time_ns.Span.t
val to_string : t -> string
val pp : Format.formatter -> t -> unit [@@ocaml.toplevel_printer]

module For_testing : sig
  val round_to_time_unit : t -> time_unit:Time_unit.t -> t
end
