open! Core

module T = struct
  type t

  (* TODO: Consider using Bigarray here instead of OCaml lists to keep memory outside the
     OCaml heap and skip a copy. *)
  external create : 'a Data_type.Typed.t -> string -> 'a list -> t = "rust_series_new"

  external create'
    :  'a Data_type.Typed.t
    -> string
    -> 'a option list
    -> t
    = "rust_series_new_option"

  let int = create Int64
  let int' = create' Int64
  let float = create Float64
  let float' = create' Float64
  let bool = create Boolean
  let bool' = create' Boolean

  (* TODO: perhaps this should actually be Bytes? *)

  let string = create Utf8
  let string' = create' Utf8

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

  external date_range
    :  string
    -> Common.Naive_datetime.t
    -> Common.Naive_datetime.t
    -> every:string option
    -> cast_to_date:bool
    -> (t, string) result
    = "rust_series_date_range"

  let date_range_castable ?every name ~start ~stop ~cast_to_date =
    date_range name start stop ~every ~cast_to_date
  ;;

  let date_range ?every name ~start ~stop =
    date_range_castable
      ?every
      name
      ~start:(Common.Naive_datetime.of_date start)
      ~stop:(Common.Naive_datetime.of_date stop)
      ~cast_to_date:true
  ;;

  let date_range_exn ?every name ~start ~stop =
    date_range ?every name ~start ~stop
    |> Result.map_error ~f:Error.of_string
    |> Or_error.ok_exn
  ;;

  let datetime_range ?every name ~start ~stop =
    date_range_castable ?every name ~start ~stop ~cast_to_date:false
  ;;

  let datetime_range_exn ?every name ~start ~stop =
    datetime_range ?every name ~start ~stop
    |> Result.map_error ~f:Error.of_string
    |> Or_error.ok_exn
  ;;

  let datetime_range' ?every name ~start ~stop =
    date_range_castable
      ?every
      name
      ~start:(Common.Naive_datetime.of_date start)
      ~stop:(Common.Naive_datetime.of_date stop)
      ~cast_to_date:false
  ;;

  let datetime_range_exn' ?every name ~start ~stop =
    datetime_range' ?every name ~start ~stop
    |> Result.map_error ~f:Error.of_string
    |> Or_error.ok_exn
  ;;

  external to_list : 'a Data_type.Typed.t -> t -> 'a list = "rust_series_to_list"

  external to_option_list
    :  'a Data_type.Typed.t
    -> t
    -> 'a option list
    = "rust_series_to_option_list"

  external get : 'a Data_type.Typed.t -> t -> int -> 'a option = "rust_series_get"

  let get_exn data_type t i = get data_type t i |> Option.value_exn ~here:[%here]

  external name : t -> string = "rust_series_name"
  external rename : t -> name:string -> t = "rust_series_rename"
  external dtype : t -> Data_type.t = "rust_series_dtype"
  external to_data_frame : t -> Data_frame0.t = "rust_series_to_data_frame"
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

  external fill_null
    :  t
    -> strategy:Fill_null_strategy.t
    -> (t, string) result
    = "rust_series_fill_null_with_strategy"

  let fill_null_exn t ~strategy =
    fill_null t ~strategy |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
  ;;

  external interpolate
    :  t
    -> method_:[ `Linear | `Nearest ]
    -> (t, string) result
    = "rust_series_interpolate"

  let interpolate_exn t ~method_ =
    interpolate t ~method_ |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
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
  let pp formatter t = Stdlib.Format.pp_print_string formatter (to_string_hum t)
end

include T
include Common.Make_numeric (T)
