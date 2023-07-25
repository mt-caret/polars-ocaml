open! Core

module T = struct
  type t

  external int : string -> int list -> t = "rust_series_new_int"
  external int_option : string -> int option list -> t = "rust_series_new_int_option"
  external float : string -> float list -> t = "rust_series_new_float"

  external float_option
    :  string
    -> float option list
    -> t
    = "rust_series_new_float_option"

  external bool : string -> bool list -> t = "rust_series_new_bool"
  external bool_option : string -> bool option list -> t = "rust_series_new_bool_option"
  external string : string -> string list -> t = "rust_series_new_string"

  external datetime
    :  string
    -> Common.Naive_datetime.t list
    -> t
    = "rust_series_new_datetime"

  let datetime' name dates =
    datetime name (List.map dates ~f:Common.Naive_datetime.of_date)
  ;;

  external date : string -> Common.Naive_date.t list -> t = "rust_series_new_date"

  let date name dates = date name (List.map dates ~f:Common.Naive_date.of_date)

  external string_option
    :  string
    -> string option list
    -> t
    = "rust_series_new_string_option"

  external date_range
    :  string
    -> Common.Naive_datetime.t
    -> Common.Naive_datetime.t
    -> cast_to_date:bool
    -> (t, string) result
    = "rust_series_date_range"

  let date_range_castable name ~start ~stop ~cast_to_date =
    date_range
      name
      (Common.Naive_datetime.of_date start)
      (Common.Naive_datetime.of_date stop)
      ~cast_to_date
  ;;

  let date_range = date_range_castable ~cast_to_date:true

  let date_range_exn name ~start ~stop =
    date_range name ~start ~stop |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
  ;;

  let datetime_range = date_range_castable ~cast_to_date:false

  let datetime_range_exn name ~start ~stop =
    datetime_range name ~start ~stop
    |> Result.map_error ~f:Error.of_string
    |> Or_error.ok_exn
  ;;

  external sort : t -> descending:bool -> t = "rust_series_sort"

  let sort ?(descending = false) t = sort t ~descending

  external head : t -> length:int option -> t = "rust_series_head"

  let head ?length t = head t ~length

  external tail : t -> length:int option -> t = "rust_series_tail"

  let tail ?length t = tail t ~length

  external sample_n
    :  t
    -> n:int
    -> with_replacement:bool
    -> shuffle:bool
    -> seed:int option
    -> (t, string) result
    = "rust_series_sample_n"

  let sample_n ?seed t ~n ~with_replacement ~shuffle =
    sample_n t ~n ~with_replacement ~shuffle ~seed
  ;;

  let sample_n_exn ?seed t ~n ~with_replacement ~shuffle =
    sample_n ?seed t ~n ~with_replacement ~shuffle
    |> Result.map_error ~f:Error.of_string
    |> Or_error.ok_exn
  ;;

  let binary_op op t1 t2 =
    op t1 t2 |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
  ;;

  external equal : t -> t -> (t, string) result = "rust_series_eq"

  let ( = ) = binary_op equal

  external ( <> ) : t -> t -> (t, string) result = "rust_series_neq"

  let ( <> ) = binary_op ( <> )

  external ( > ) : t -> t -> (t, string) result = "rust_series_gt"

  let ( > ) = binary_op ( > )

  external ( >= ) : t -> t -> (t, string) result = "rust_series_gt_eq"

  let ( >= ) = binary_op ( >= )

  external ( < ) : t -> t -> (t, string) result = "rust_series_lt"

  let ( < ) = binary_op ( < )

  external ( <= ) : t -> t -> (t, string) result = "rust_series_lt_eq"

  let ( <= ) = binary_op ( <= )

  external add : t -> t -> t = "rust_series_add"
  external sub : t -> t -> t = "rust_series_sub"
  external mul : t -> t -> t = "rust_series_mul"
  external div : t -> t -> t = "rust_series_div"
  external to_string_hum : t -> string = "rust_series_to_string_hum"

  let print t = print_endline (to_string_hum t)

  type typed_list =
    | Int of int option list
    | Int32 of Int32.t option list
    | Float of float option list
    | String of string option list
    | Bytes of bytes option list
  [@@deriving sexp_of]

  external to_typed_list : t -> (typed_list, string) result = "rust_series_to_typed_list"

  let to_typed_list_exn t =
    t |> to_typed_list |> Result.map_error ~f:Error.of_string |> ok_exn
  ;;
end

include T
include Common.Make_numeric (T)
