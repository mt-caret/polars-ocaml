open! Core

module T = struct
  type t

  external col : string -> t = "rust_expr_col"
  external cols : string list -> t = "rust_expr_cols"
  external all : unit -> t = "rust_expr_all"
  external exclude : string -> t = "rust_expr_exclude"

  let element () = col ""

  external cast : t -> to_:Data_type.t -> strict:bool -> t = "rust_expr_cast"

  let cast ?(strict = true) t ~to_ = cast t ~to_ ~strict

  external null : unit -> t = "rust_expr_null"
  external int : int -> t = "rust_expr_int"
  external float : float -> t = "rust_expr_float"
  external bool : bool -> t = "rust_expr_bool"
  external string : string -> t = "rust_expr_string"
  external naive_date : Common.Naive_date.t -> t = "rust_expr_naive_date"
  external naive_datetime : Common.Naive_datetime.t -> t = "rust_expr_naive_datetime"
  external sort : t -> descending:bool -> t = "rust_expr_sort"

  let sort ?(descending = false) t = sort t ~descending

  external sort_by : t -> descending:bool -> by:t list -> t = "rust_expr_sort_by"

  let sort_by ?(descending = false) t ~by = sort_by t ~descending ~by

  external first : t -> t = "rust_expr_first"
  external last : t -> t = "rust_expr_last"
  external reverse : t -> t = "rust_expr_reverse"
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
    -> fixed_seed:bool
    -> t
    = "rust_expr_sample_n_bytecode" "rust_expr_sample_n"

  let sample_n ?seed ?(fixed_seed = true) t ~n ~with_replacement ~shuffle =
    sample_n t ~n ~with_replacement ~shuffle ~seed ~fixed_seed
  ;;

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

  external over
    :  t
    -> partition_by:t list
    -> mapping_strategy:[ `Groups_to_rows | `Explode | `Join ]
    -> t
    = "rust_expr_over"

  let over ?(mapping_strategy = `Groups_to_rows) t ~partition_by =
    over t ~partition_by ~mapping_strategy
  ;;

  external concat_list : t list -> (t, string) result = "rust_expr_concat_list"

  let concat_list ts =
    Nonempty_list.to_list ts
    |> concat_list
    |> Result.map_error ~f:Error.of_string
    (* Currently the only way time that rust_expr_concat_list will return an
       Error is when the argument is an empty list, so this should never raise *)
    |> Or_error.ok_exn
  ;;

  external null_count : t -> t = "rust_expr_null_count"
  external is_null : t -> t = "rust_expr_is_null"
  external is_not_null : t -> t = "rust_expr_is_not_null"
  external fill_null : t -> with_:t -> t = "rust_expr_fill_null"

  external fill_null'
    :  t
    -> strategy:Fill_null_strategy.t
    -> t
    = "rust_expr_fill_null_with_strategy"

  external interpolate
    :  t
    -> method_:[ `Linear | `Nearest ]
    -> t
    = "rust_expr_interpolate"

  let interpolate ?(method_ = `Linear) t = interpolate t ~method_

  external fill_nan : t -> with_:t -> t = "rust_expr_fill_nan"

  external rank
    :  t
    -> method_:[ `Average | `Min | `Max | `Dense | `Ordinal | `Random ]
    -> descending:bool
    -> seed:int option
    -> t
    = "rust_expr_rank"

  let rank ?(method_ = `Dense) ?(descending = false) ?seed t =
    rank t ~method_ ~descending ~seed
  ;;

  external when_ : (t * t) list -> otherwise:t -> t = "rust_expr_when_then"
  external shift : t -> periods:int -> fill_value:t option -> t = "rust_expr_shift"

  let shift ?fill_value t ~periods = shift t ~periods ~fill_value

  (* TODO: couldn't we do something like Haskell's Foldable typeclass and generalize? *)
  external cum_count : t -> reverse:bool -> t = "rust_expr_cum_count"

  let cum_count ?(reverse = false) t = cum_count t ~reverse

  external cum_sum : t -> reverse:bool -> t = "rust_expr_cum_sum"

  let cum_sum ?(reverse = false) t = cum_sum t ~reverse

  external cum_prod : t -> reverse:bool -> t = "rust_expr_cum_prod"

  let cum_prod ?(reverse = false) t = cum_prod t ~reverse

  external cum_min : t -> reverse:bool -> t = "rust_expr_cum_min"

  let cum_min ?(reverse = false) t = cum_min t ~reverse

  external cum_max : t -> reverse:bool -> t = "rust_expr_cum_max"

  let cum_max ?(reverse = false) t = cum_max t ~reverse

  external alias : t -> name:string -> t = "rust_expr_alias"
  external prefix : t -> prefix:string -> t = "rust_expr_prefix"
  external suffix : t -> suffix:string -> t = "rust_expr_suffix"
  external round : t -> decimals:int -> t = "rust_expr_round"
  external equal : t -> t -> t = "rust_expr_eq"

  let ( = ) = equal

  external ( <> ) : t -> t -> t = "rust_expr_neq"
  external ( > ) : t -> t -> t = "rust_expr_gt"
  external ( >= ) : t -> t -> t = "rust_expr_gt_eq"
  external ( < ) : t -> t -> t = "rust_expr_lt"
  external ( <= ) : t -> t -> t = "rust_expr_lt_eq"
  external not : t -> t = "rust_expr_not"
  external and_ : t -> t -> t = "rust_expr_and"
  external or_ : t -> t -> t = "rust_expr_or"
  external xor : t -> t -> t = "rust_expr_xor"
  external add : t -> t -> t = "rust_expr_add"
  external sub : t -> t -> t = "rust_expr_sub"
  external mul : t -> t -> t = "rust_expr_mul"
  external div : t -> t -> t = "rust_expr_div"
  external floor_div : t -> t -> t = "rust_expr_floor_div"

  let ( // ) = floor_div
end

include T
include Common.Make_logic (T)
include Common.Make_numeric (T)

module Dt = struct
  external strftime : t -> format:string -> t = "rust_expr_dt_strftime"

  (* TODO: consider supporting Time_ns.Zone.t *)
  external convert_time_zone : t -> to_:string -> t = "rust_expr_dt_convert_time_zone"
  external year : t -> t = "rust_expr_dt_year"
  external month : t -> t = "rust_expr_dt_month"
  external day : t -> t = "rust_expr_dt_day"
end

module Str = struct
  external split : t -> by:string -> inclusive:bool -> t = "rust_expr_str_split"

  let split ?(inclusive = false) t ~by = split t ~by ~inclusive

  external strptime
    :  t
    -> type_:Data_type.t
    -> format:string
    -> t
    = "rust_expr_str_strptime"

  external lengths : t -> t = "rust_expr_str_lengths"
  external n_chars : t -> t = "rust_expr_str_n_chars"
  external contains : t -> pat:string -> literal:bool -> t = "rust_expr_str_contains"

  let contains ?(literal = false) t ~pat = contains t ~pat ~literal

  external starts_with : t -> prefix:string -> t = "rust_expr_str_starts_with"
  external ends_with : t -> suffix:string -> t = "rust_expr_str_ends_with"
  external extract : t -> pat:string -> group:int -> t = "rust_expr_str_extract"
  external extract_all : t -> pat:string -> t = "rust_expr_str_extract_all"

  external replace
    :  t
    -> pat:string
    -> with_:string
    -> literal:bool
    -> t
    = "rust_expr_str_replace"

  let replace ?(literal = false) t ~pat ~with_ = replace t ~pat ~with_ ~literal

  external replace_all
    :  t
    -> pat:string
    -> with_:string
    -> literal:bool
    -> t
    = "rust_expr_str_replace_all"

  let replace_all ?(literal = false) t ~pat ~with_ = replace_all t ~pat ~with_ ~literal
end

module List = struct
  external lengths : t -> t = "rust_expr_list_lengths"
  external slice : t -> offset:t -> length:t -> t = "rust_expr_list_slice"
  external head : t -> n:t -> t = "rust_expr_list_head"
  external tail : t -> n:t -> t = "rust_expr_list_tail"
  external sum : t -> t = "rust_expr_list_sum"
  external eval : t -> expr:t -> parallel:bool -> t = "rust_expr_list_eval"

  let eval ?(parallel = false) t ~expr = eval t ~expr ~parallel
end
