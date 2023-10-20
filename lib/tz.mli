open! Core

(** [Tz.t] represents the time zone type used by polars. Since the underlyingx
    type is statically generated at compile-time, it is not guaranteed to always
    match tz values which are dynamically queries from the tz database at
    runtime.

    The underlying Rust type is chrono_tz::Tz
    (https://docs.rs/chrono-tz/latest/chrono_tz/) *)
type t [@@deriving compare, sexp, quickcheck]

val all : unit -> t list
val to_string : t -> string
val parse : string -> t option
val of_time_zone : Time_ns_unix.Zone.t -> t option
val to_time_zone : t -> Time_ns_unix.Zone.t option
