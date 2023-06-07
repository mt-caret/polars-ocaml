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
external n_unique : t -> t = "rust_expr_n_unique"
external approx_unique : t -> t = "rust_expr_approx_unique"
external is_null : t -> t = "rust_expr_is_null"
external is_not_null : t -> t = "rust_expr_is_not_null"
external when_ : (t * t) list -> otherwise:t -> t = "rust_expr_when_then"
external alias : t -> name:string -> t = "rust_expr_alias"
external prefix : t -> prefix:string -> t = "rust_expr_prefix"
external suffix : t -> suffix:string -> t = "rust_expr_suffix"
external equal : t -> t -> t = "rust_expr_eq"
val ( = ) : t -> t -> t
external ( <> ) : t -> t -> t = "rust_expr_neq"
external ( > ) : t -> t -> t = "rust_expr_gt"
external ( >= ) : t -> t -> t = "rust_expr_gt_eq"
external ( < ) : t -> t -> t = "rust_expr_lt"
external ( <= ) : t -> t -> t = "rust_expr_lt_eq"
external not : t -> t = "rust_expr_not"
val ( ! ) : t -> t
external and_ : t -> t -> t = "rust_expr_and"
external or_ : t -> t -> t = "rust_expr_or"
external xor : t -> t -> t = "rust_expr_xor"
val ( && ) : t -> t -> t
val ( || ) : t -> t -> t
val ( lxor ) : t -> t -> t
external add : t -> t -> t = "rust_expr_add"
external sub : t -> t -> t = "rust_expr_sub"
external mul : t -> t -> t = "rust_expr_mul"
external div : t -> t -> t = "rust_expr_div"
val ( + ) : t -> t -> t
val ( - ) : t -> t -> t
val ( * ) : t -> t -> t
val ( / ) : t -> t -> t

module Dt : sig
  external strftime : t -> format:string -> t = "rust_expr_dt_strftime"
end

module Str : sig
  external strptime
    :  t
    -> type_:Data_type.t
    -> format:string
    -> t
    = "rust_expr_str_strptime"
end
