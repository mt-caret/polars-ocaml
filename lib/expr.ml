open! Core

type t

external col : string -> t = "rust_expr_col"
external all : unit -> t = "rust_expr_all"
external exclude : string -> t = "rust_expr_exclude"
external cast : t -> to_:Data_type.t -> strict:bool -> t = "rust_expr_cast"

let cast ?(strict = true) t ~to_ = cast t ~to_ ~strict

external int : int -> t = "rust_expr_int"
external float : float -> t = "rust_expr_float"
external bool : bool -> t = "rust_expr_bool"
external string : string -> t = "rust_expr_string"
external sort : t -> descending:bool -> t = "rust_expr_sort"
external head : t -> length:int option -> t option = "rust_expr_head"

let head ?length t = head t ~length |> Option.value_exn ~here:[%here]

external tail : t -> length:int option -> t option = "rust_expr_tail"

let tail ?length t = tail t ~length |> Option.value_exn ~here:[%here]

external sample_n
  :  t
  -> n:int
  -> with_replacement:bool
  -> shuffle:bool
  -> seed:int option
  -> t option
  = "rust_expr_sample_n"

let sample_n ?seed t ~n ~with_replacement ~shuffle =
  sample_n t ~n ~with_replacement ~shuffle ~seed |> Option.value_exn ~here:[%here]
;;

external filter : t -> predicate:t -> t = "rust_expr_filter"
external sum : t -> t = "rust_expr_sum"
external n_unique : t -> t = "rust_expr_n_unique"
external approx_unique : t -> t = "rust_expr_approx_unique"
external when_ : (t * t) list -> otherwise:t -> t = "rust_expr_when_then"
external alias : t -> name:string -> t = "rust_expr_alias"
external prefix : t -> prefix:string -> t = "rust_expr_prefix"
external suffix : t -> suffix:string -> t = "rust_expr_suffix"
external equal : t -> t -> t = "rust_expr_eq"

let ( = ) = equal

external ( <> ) : t -> t -> t = "rust_expr_neq"
external ( > ) : t -> t -> t = "rust_expr_gt"
external ( >= ) : t -> t -> t = "rust_expr_gt_eq"
external ( < ) : t -> t -> t = "rust_expr_lt"
external ( <= ) : t -> t -> t = "rust_expr_lt_eq"
external not : t -> t = "rust_expr_not"

let ( ! ) = not

external and_ : t -> t -> t = "rust_expr_and"
external or_ : t -> t -> t = "rust_expr_or"
external xor : t -> t -> t = "rust_expr_xor"

let ( && ) = and_
let ( || ) = or_
let ( lxor ) = xor

external add : t -> t -> t = "rust_expr_add"
external sub : t -> t -> t = "rust_expr_sub"
external mul : t -> t -> t = "rust_expr_mul"
external div : t -> t -> t = "rust_expr_div"

let ( + ) = add
let ( - ) = sub
let ( * ) = mul
let ( / ) = div

module Dt = struct
  external strftime : t -> format:string -> t = "rust_expr_dt_strftime"
end

module Str = struct
  external strptime
    :  t
    -> type_:Data_type.t
    -> format:string
    -> t
    = "rust_expr_str_strptime"
end
