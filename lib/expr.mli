open! Core

type t

external col : string -> t = "rust_expr_col"
external all : unit -> t = "rust_expr_all"
external exclude : string -> t = "rust_expr_exclude"
val cast : ?strict:bool -> t -> to_:Data_type.t -> t
external int : int -> t = "rust_expr_int"
external float : float -> t = "rust_expr_float"
external bool : bool -> t = "rust_expr_bool"
external string : string -> t = "rust_expr_string"
val sort : ?descending:bool -> t -> t
val sort_by : ?descending:bool -> t -> by:t list -> t
external first : t -> t = "rust_expr_first"
external last : t -> t = "rust_expr_last"
external reverse : t -> t = "rust_expr_reverse"
val head : ?length:int -> t -> t
val tail : ?length:int -> t -> t
val sample_n : ?seed:int -> t -> n:int -> with_replacement:bool -> shuffle:bool -> t
external filter : t -> predicate:t -> t = "rust_expr_filter"
external sum : t -> t = "rust_expr_sum"
external mean : t -> t = "rust_expr_mean"
external count : t -> t = "rust_expr_count"
external count_ : unit -> t = "rust_expr_count_"
external n_unique : t -> t = "rust_expr_n_unique"
external approx_unique : t -> t = "rust_expr_approx_unique"
external is_null : t -> t = "rust_expr_is_null"
external is_not_null : t -> t = "rust_expr_is_not_null"
external when_ : (t * t) list -> otherwise:t -> t = "rust_expr_when_then"
external alias : t -> name:string -> t = "rust_expr_alias"
external prefix : t -> prefix:string -> t = "rust_expr_prefix"
external suffix : t -> suffix:string -> t = "rust_expr_suffix"
external equal : t -> t -> t = "rust_expr_eq"

include Common.Compare with type t := t
include Common.Logic with type t := t
include Common.Numeric with type t := t

module Dt : sig
  external strftime : t -> format:string -> t = "rust_expr_dt_strftime"
  external year : t -> t = "rust_expr_dt_year"
  external month : t -> t = "rust_expr_dt_month"
  external day : t -> t = "rust_expr_dt_day"
end

module Str : sig
  external strptime
    :  t
    -> type_:Data_type.t
    -> format:string
    -> t
    = "rust_expr_str_strptime"

  external lengths : t -> t = "rust_expr_str_lengths"
  external n_chars : t -> t = "rust_expr_str_n_chars"
  val contains : ?literal:bool -> t -> pat:string -> t
  external starts_with : t -> prefix:string -> t = "rust_expr_str_starts_with"
  external ends_with : t -> suffix:string -> t = "rust_expr_str_ends_with"
  val extract : t -> pat:string -> group:int -> t
  external extract_all : t -> pat:string -> t = "rust_expr_str_extract_all"
  val replace : ?literal:bool -> t -> pat:string -> with_:string -> t
  val replace_all : ?literal:bool -> t -> pat:string -> with_:string -> t
end
