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

val date : string -> Date.t list -> t
val datetime : string -> Common.Naive_datetime.t list -> t
val datetime' : string -> Date.t list -> t
val date_range : string -> start:Date.t -> stop:Date.t -> (t, string) result
val date_range_exn : string -> start:Date.t -> stop:Date.t -> t
val datetime_range : string -> start:Date.t -> stop:Date.t -> (t, string) result
val datetime_range_exn : string -> start:Date.t -> stop:Date.t -> t
val sort : ?descending:bool -> t -> t
val head : ?length:int -> t -> t
val tail : ?length:int -> t -> t

val sample_n
  :  ?seed:int
  -> t
  -> n:int
  -> with_replacement:bool
  -> shuffle:bool
  -> (t, string) result

val sample_n_exn : ?seed:int -> t -> n:int -> with_replacement:bool -> shuffle:bool -> t
external to_string_hum : t -> string = "rust_series_to_string_hum"
val print : t -> unit

type typed_list =
  | Int of int option list
  | Int32 of Int32.t option list
  | Float of float option list
  | String of string option list
  | Bytes of bytes option list
[@@deriving sexp_of]

external to_typed_list : t -> (typed_list, string) result = "rust_series_to_typed_list"
val to_typed_list_exn : t -> typed_list

include Common.Compare with type t := t
include Common.Numeric with type t := t
