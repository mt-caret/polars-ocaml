open! Core

type t

val create : 'a Data_type.Typed.t -> string -> 'a list -> t
val createo : 'a Data_type.Typed.t -> string -> 'a option list -> t
val int : string -> int list -> t
val into : string -> int option list -> t
val float : string -> float list -> t
val floato : string -> float option list -> t
val bool : string -> bool list -> t
val boolo : string -> bool option list -> t
val string : string -> string list -> t
val stringo : string -> string option list -> t
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
val to_list : 'a Data_type.Typed.t -> t -> 'a list
val to_option_list : 'a Data_type.Typed.t -> t -> 'a option list
val get : 'a Data_type.Typed.t -> t -> int -> 'a option
val get_exn : 'a Data_type.Typed.t -> t -> int -> 'a
val name : t -> string
val rename : t -> name:string -> t
val dtype : t -> Data_type.t
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
val fill_null : t -> strategy:Fill_null_strategy.t -> (t, string) result
val fill_null_exn : t -> strategy:Fill_null_strategy.t -> t
val interpolate : t -> method_:[ `Linear | `Nearest ] -> (t, string) result
val interpolate_exn : t -> method_:[ `Linear | `Nearest ] -> t
val to_string_hum : t -> string
val print : t -> unit
val pp : Format.formatter -> t -> unit [@@ocaml.toplevel_printer]

include Common.Compare with type t := t
include Common.Numeric with type t := t
