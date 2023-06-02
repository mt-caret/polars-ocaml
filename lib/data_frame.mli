open! Core

type t

external create : Series.t list -> (t, string) result = "rust_data_frame_new"
val create_exn : Series.t list -> t
external to_string_hum : t -> string = "rust_data_frame_to_string_hum"
val print : t -> unit
