open! Core

type t

external col : string -> t = "rust_expr_col"
external cols : string list -> t = "rust_expr_cols"
external all : unit -> t = "rust_expr_all"
external exclude : string -> t = "rust_expr_exclude"
val element : unit -> t
val cast : ?strict:bool -> t -> to_:Data_type.t -> t
external null : unit -> t = "rust_expr_null"
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
external median : t -> t = "rust_expr_median"
external max : t -> t = "rust_expr_max"
external min : t -> t = "rust_expr_min"
external count : t -> t = "rust_expr_count"
external count_ : unit -> t = "rust_expr_count_"
external n_unique : t -> t = "rust_expr_n_unique"
external approx_unique : t -> t = "rust_expr_approx_unique"
external explode : t -> t = "rust_expr_explode"

val over
  :  ?mapping_strategy:[ `Groups_to_rows | `Explode | `Join ]
  -> t
  -> partition_by:t list
  -> t

val concat_list : t Nonempty_list.t -> t
external null_count : t -> t = "rust_expr_null_count"
external is_null : t -> t = "rust_expr_is_null"
external is_not_null : t -> t = "rust_expr_is_not_null"
external fill_null : t -> with_:t -> t = "rust_expr_fill_null"

external fill_null'
  :  t
  -> strategy:Fill_null_strategy.t
  -> t
  = "rust_expr_fill_null_with_strategy"

val interpolate : ?method_:[ `Linear | `Nearest ] -> t -> t
external fill_nan : t -> with_:t -> t = "rust_expr_fill_nan"

val rank
  :  ?method_:[ `Average | `Dense | `Max | `Min | `Ordinal | `Random ]
  -> ?descending:bool
  -> ?seed:int
  -> t
  -> t

external when_ : (t * t) list -> otherwise:t -> t = "rust_expr_when_then"
val shift : ?fill_value:t -> t -> periods:int -> t
val cum_count : ?reverse:bool -> t -> t
val cum_sum : ?reverse:bool -> t -> t
val cum_prod : ?reverse:bool -> t -> t
val cum_min : ?reverse:bool -> t -> t
val cum_max : ?reverse:bool -> t -> t
external alias : t -> name:string -> t = "rust_expr_alias"
external prefix : t -> prefix:string -> t = "rust_expr_prefix"
external suffix : t -> suffix:string -> t = "rust_expr_suffix"
val round : t -> decimals:int -> t
external equal : t -> t -> t = "rust_expr_eq"

include Common.Compare with type t := t
include Common.Logic with type t := t
include Common.Numeric with type t := t

(* TODO: apparently this doesn't exist for series, which is surprising! *)
val floor_div : t -> t -> t
val ( // ) : t -> t -> t

module Dt : sig
  external strftime : t -> format:string -> t = "rust_expr_dt_strftime"
  external convert_time_zone : t -> to_:string -> t = "rust_expr_dt_convert_time_zone"
  external year : t -> t = "rust_expr_dt_year"
  external month : t -> t = "rust_expr_dt_month"
  external day : t -> t = "rust_expr_dt_day"
end

module Str : sig
  val split : ?inclusive:bool -> t -> by:string -> t

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

module List : sig
  external lengths : t -> t = "rust_expr_list_lengths"
  external slice : t -> offset:t -> length:t -> t = "rust_expr_list_slice"
  external head : t -> n:t -> t = "rust_expr_list_head"
  external tail : t -> n:t -> t = "rust_expr_list_tail"
  external sum : t -> t = "rust_expr_list_sum"
  val eval : ?parallel:bool -> t -> expr:t -> t
end
