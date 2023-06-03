open! Core

type t

external int : string -> int list -> t = "rust_series_new_int"
external int_option : string -> int option list -> t = "rust_series_new_int_option"
external float : string -> float list -> t = "rust_series_new_float"
external float_option : string -> float option list -> t = "rust_series_new_float_option"
external bool : string -> bool list -> t = "rust_series_new_bool"
external bool_option : string -> bool option list -> t = "rust_series_new_bool_option"
external string : string -> string list -> t = "rust_series_new_string"

external string_option
  :  string
  -> string option list
  -> t
  = "rust_series_new_string_option"

val date_range : string -> start:Date.t -> stop:Date.t -> (t, string) result
val date_range_exn : string -> start:Date.t -> stop:Date.t -> t
val datetime_range : string -> start:Date.t -> stop:Date.t -> (t, string) result
val datetime_range_exn : string -> start:Date.t -> stop:Date.t -> t
external to_string_hum : t -> string = "rust_series_to_string_hum"
val print : t -> unit
