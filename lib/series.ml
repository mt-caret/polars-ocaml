open! Core

module T = struct
  type t

  (* TODO: Consider using Bigarray here instead of OCaml lists to keep memory outside the
     OCaml heap and skip a copy. *)
  external create : 'a Data_type.Typed.t -> string -> 'a list -> t = "rust_series_new"

  let create (type a) (data_type : a Data_type.Typed.t) name values =
    match Data_type.Typed.flatten_custom data_type with
    | Custom { data_type; f = _; f_inverse } ->
      create data_type name (List.map values ~f:f_inverse)
    | data_type -> create data_type name values
  ;;

  external createo
    :  'a Data_type.Typed.t
    -> string
    -> 'a option list
    -> t
    = "rust_series_new_option"

  let createo (type a) (data_type : a Data_type.Typed.t) name values =
    match Data_type.Typed.flatten_custom data_type with
    | Custom { data_type; f = _; f_inverse } ->
      createo data_type name (List.map values ~f:(Option.map ~f:f_inverse))
    | data_type -> createo data_type name values
  ;;

  (* TODO: the astute reader will realize that this is quite terrible when
     trying to passing float arrays! Unfortunately it's not clear to me how
     to pass regular arrays safely to Rust, whereas this is definitely safe
     since [Uniform_array.t] guarantees elements are boxed.

     See https://github.com/mt-caret/polars-ocaml/pull/67 for how naively trying to
     transmute on the Rust side doesn't work. *)
  external create'
    :  'a Data_type.Typed.t
    -> string
    -> 'a Uniform_array.t
    -> t
    = "rust_series_new_array"

  let create' (type a) (data_type : a Data_type.Typed.t) name values =
    match Data_type.Typed.flatten_custom data_type with
    | Custom { data_type; f = _; f_inverse } ->
      create' data_type name (Uniform_array.map values ~f:f_inverse)
    | data_type -> create' data_type name values
  ;;

  external createo'
    :  'a Data_type.Typed.t
    -> string
    -> 'a option Uniform_array.t
    -> t
    = "rust_series_new_option_array"

  let createo' (type a) (data_type : a Data_type.Typed.t) name values =
    match Data_type.Typed.flatten_custom data_type with
    | Custom { data_type; f = _; f_inverse } ->
      createo' data_type name (Uniform_array.map values ~f:(Option.map ~f:f_inverse))
    | data_type -> createo' data_type name values
  ;;

  let int = create Int64
  let into = createo Int64
  let float = create Float64
  let floato = createo Float64
  let bool = create Boolean
  let boolo = createo Boolean

  (* TODO: perhaps this should actually be Bytes? *)

  let string = create Utf8
  let stringo = createo Utf8

  external datetime
    :  string
    -> Common.Naive_datetime.t list
    -> t
    = "rust_series_new_datetime"

  external datetime_option
    :  string
    -> Common.Naive_datetime.t option list
    -> t
    = "rust_series_new_datetime_option"

  let datetime' name dates =
    datetime name (List.map dates ~f:Common.Naive_datetime.of_date)
  ;;

  let datetime_option' name dates =
    datetime_option name (List.map dates ~f:(Option.map ~f:Common.Naive_datetime.of_date))
  ;;

  let time string times =
    datetime string (List.map times ~f:Common.Naive_datetime.of_time_ns_exn)
  ;;

  let time_option string times =
    datetime_option
      string
      (List.map times ~f:(Option.map ~f:Common.Naive_datetime.of_time_ns_exn))
  ;;

  let date = create Data_type.Typed.date
  let dateo = createo Data_type.Typed.date

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
    date_range ?every name ~start ~stop |> Utils.string_result_ok_exn
  ;;

  let datetime_range ?every name ~start ~stop =
    date_range_castable ?every name ~start ~stop ~cast_to_date:false
  ;;

  let datetime_range_exn ?every name ~start ~stop =
    datetime_range ?every name ~start ~stop |> Utils.string_result_ok_exn
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
    datetime_range' ?every name ~start ~stop |> Utils.string_result_ok_exn
  ;;

  external to_list : 'a Data_type.Typed.t -> t -> 'a list = "rust_series_to_list"

  let to_list (type a) (data_type : a Data_type.Typed.t) t =
    match Data_type.Typed.flatten_custom data_type with
    | Custom { data_type; f; f_inverse = _ } -> to_list data_type t |> List.map ~f
    | data_type -> to_list data_type t
  ;;

  external to_option_list
    :  'a Data_type.Typed.t
    -> t
    -> 'a option list
    = "rust_series_to_option_list"

  let to_option_list (type a) (data_type : a Data_type.Typed.t) t =
    match Data_type.Typed.flatten_custom data_type with
    | Custom { data_type; f; f_inverse = _ } ->
      to_option_list data_type t |> List.map ~f:(Option.map ~f)
    | data_type -> to_option_list data_type t
  ;;

  external get : 'a Data_type.Typed.t -> t -> int -> 'a option = "rust_series_get"

  let get (type a) (data_type : a Data_type.Typed.t) t i =
    match Data_type.Typed.flatten_custom data_type with
    | Custom { data_type; f; f_inverse = _ } -> get data_type t i |> Option.map ~f
    | data_type -> get data_type t i
  ;;

  let get_exn data_type t i = get data_type t i |> Option.value_exn ~here:[%here]

  external name : t -> string = "rust_series_name"
  external rename : t -> name:string -> unit = "rust_series_rename"
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
    sample_n ?seed t ~n ~with_replacement ~shuffle |> Utils.string_result_ok_exn
  ;;

  external fill_null
    :  t
    -> strategy:Fill_null_strategy.t
    -> (t, string) result
    = "rust_series_fill_null_with_strategy"

  let fill_null_exn t ~strategy = fill_null t ~strategy |> Utils.string_result_ok_exn

  external interpolate
    :  t
    -> method_:[ `Linear | `Nearest ]
    -> (t, string) result
    = "rust_series_interpolate"

  let interpolate_exn t ~method_ = interpolate t ~method_ |> Utils.string_result_ok_exn
  let binary_op op t1 t2 = op t1 t2 |> Utils.string_result_ok_exn

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
