open! Core

type t

val int : string -> int list -> t
val int_option : string -> int option list -> t
val float : string -> float list -> t
val float_option : string -> float option list -> t
val bool : string -> bool list -> t
val bool_option : string -> bool option list -> t
val string : string -> string list -> t
val string_option : string -> string option list -> t
val date : string -> Date.t list -> t
val datetime : string -> Common.Naive_datetime.t list -> t
val datetime' : string -> Date.t list -> t

val date_range
  :  ?every:string
  -> string
  -> start:Date.t
  -> stop:Date.t
  -> (t, string) result

val date_range_exn : ?every:string -> string -> start:Date.t -> stop:Date.t -> t

val datetime_range
  :  ?every:string
  -> string
  -> start:Common.Naive_datetime.t
  -> stop:Common.Naive_datetime.t
  -> (t, string) result

val datetime_range_exn
  :  ?every:string
  -> string
  -> start:Common.Naive_datetime.t
  -> stop:Common.Naive_datetime.t
  -> t

val datetime_range'
  :  ?every:string
  -> string
  -> start:Date.t
  -> stop:Date.t
  -> (t, string) result

val datetime_range_exn' : ?every:string -> string -> start:Date.t -> stop:Date.t -> t
val rename : t -> name:string -> t
val to_data_frame : t -> Data_frame0.t
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
val to_string_hum : t -> string
val print : t -> unit

type typed_list =
  | Int of int option list
  | Int32 of Int32.t option list
  | Float of float option list
  | String of string option list
  | Bytes of bytes option list
[@@deriving sexp_of]

val to_typed_list : t -> (typed_list, string) result
val to_typed_list_exn : t -> typed_list

include Common.Compare with type t := t
include Common.Numeric with type t := t
