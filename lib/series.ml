open! Core

type t

external int : string -> int list -> t = "rust_series_new_int"
external int_option : string -> int option list -> t = "rust_series_new_int_option"
external float : string -> float list -> t = "rust_series_new_float"
external float_option : string -> float option list -> t = "rust_series_new_float_option"
external string : string -> string list -> t = "rust_series_new_string"

external string_option
  :  string
  -> string option list
  -> t
  = "rust_series_new_string_option"

external to_string_hum : t -> string = "rust_series_to_string_hum"

let print t = print_endline (to_string_hum t)
