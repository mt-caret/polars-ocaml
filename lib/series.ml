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

  (* [create'] actually lies about its signature; if a [float array] is passed
     and it happens to be unboxed, the Rust-side function does not know how to
     properly handle it and will crash. So, instead of exposing it directly we
     branch on the tag that indicates whether the array is boxed or not and
     switch implementations between it and [float'], which takes a [floatarray]
     (which is guaranteed to always be unboxed).

     Converting from an [float array] to an [floatarray] via [Obj.magic] is
     safe, since on the Rust side we immediately copy the values into a
     [Vec<f64>]. *)
  external create'
    :  'a Data_type.Typed.t
    -> string
    -> 'a array
    -> t
    = "rust_series_new_array"

  external float'
    :  string
    -> floatarray
    -> downcast_to_f32:bool
    -> t
    = "rust_series_new_float_array"

  let%expect_test "floatarrays have double array tags" =
    let floatarray = Stdlib.Float.Array.of_list [ 1.; 2.; 3. ] in
    assert (Obj.tag (Obj.repr floatarray) = Obj.double_array_tag)
  ;;

  let create' =
    let rec go : type a. a Data_type.Typed.t -> string -> a array -> t =
      fun data_type name values ->
      let has_double_array_tag = Obj.tag (Obj.repr values) = Obj.double_array_tag in
      match Data_type.Typed.flatten_custom data_type with
      | Custom { data_type; f = _; f_inverse } ->
        let values = Array.map values ~f:f_inverse in
        go data_type name values
      | Float32 ->
        if has_double_array_tag
        then float' name (Obj.magic values) ~downcast_to_f32:true
        else create' data_type name values
      | Float64 ->
        if has_double_array_tag
        then float' name (Obj.magic values) ~downcast_to_f32:false
        else create' data_type name values
      | data_type -> create' data_type name values
    in
    go
  ;;

  external createo'
    :  'a Data_type.Typed.t
    -> string
    -> 'a option array
    -> t
    = "rust_series_new_option_array"

  let createo' (type a) (data_type : a Data_type.Typed.t) name values =
    match Data_type.Typed.flatten_custom data_type with
    | Custom { data_type; f = _; f_inverse } ->
      createo' data_type name (Array.map values ~f:(Option.map ~f:f_inverse))
    | data_type -> createo' data_type name values
  ;;

  let int = create Int64
  let into = createo Int64
  let float = create Float64
  let floato = createo Float64
  let float' name values = float' name values ~downcast_to_f32:false
  let bool = create Boolean
  let boolo = createo Boolean

  (* TODO: perhaps this should actually be Bytes? *)

  let string = create Utf8
  let stringo = createo Utf8
  let datetime = create (Datetime (Nanoseconds, None))
  let datetimeo = createo (Datetime (Nanoseconds, None))
  let datetime' = create Data_type.Typed.Core.time
  let datetimeo' = createo Data_type.Typed.Core.time
  let date = create Date
  let dateo = createo Date
  let date' = create Data_type.Typed.Core.date
  let dateo' = createo Data_type.Typed.Core.date
  let duration = create (Duration Nanoseconds)
  let durationo = createo (Duration Nanoseconds)
  let duration' = create Data_type.Typed.Core.span
  let durationo' = createo Data_type.Typed.Core.span
  let time = create Time
  let timeo = createo Time
  let time' = create Data_type.Typed.Core.ofday
  let timeo' = createo Data_type.Typed.Core.ofday

  external date_range
    :  string
    -> Naive_datetime.t
    -> Naive_datetime.t
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
      ~start:(Naive_datetime.of_date start)
      ~stop:(Naive_datetime.of_date stop)
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
      ~start:(Naive_datetime.of_date start)
      ~stop:(Naive_datetime.of_date stop)
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

  external map
    :  'a Data_type.Typed.t
    -> 'b Data_type.Typed.t
    -> t
    -> f:('a option -> 'b option)
    -> (t, exn) result
    = "rust_series_map"

  let map input_data_type output_data_type t ~f =
    map input_data_type output_data_type t ~f |> Result.ok_exn
  ;;

  external cast
    :  t
    -> to_:Data_type.t
    -> strict:bool
    -> (t, string) result
    = "rust_series_cast"

  let cast ?(strict = true) t ~to_ = cast t ~to_ ~strict |> Utils.string_result_ok_exn

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

  external estimated_size : t -> int = "rust_series_estimated_size"

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

  external clear : t -> unit = "rust_series_clear"

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
end

include T

include Pretty_printer.Register (struct
    type nonrec t = t

    let module_name = "Polars.Series"
    let to_string = to_string_hum
  end)

include Common.Make_numeric (T)

module Expert = struct
  external modify_at_chunk_index_exn
    :  t
    -> dtype:'a Data_type.Typed.t
    -> chunk_index:int
    -> indices_and_values:(int * 'a) list
    -> (unit, string) result
    = "rust_series_modify_at_chunk_index"

  let modify_at_chunk_index t ~dtype ~chunk_index ~indices_and_values =
    match
      Or_error.try_with (fun () ->
        modify_at_chunk_index_exn t ~dtype ~chunk_index ~indices_and_values)
    with
    | Ok result -> result
    | Error error ->
      Error
        (sprintf
           "modify_at_chunk_index_exn raised an exception. Usually this happens when \
            accessing an index out of bounds of the chunk or passing in a value outside \
            of the domain of dtype: %s"
           (Error.to_string_hum error))
  ;;

  external modify_optional_at_chunk_index_exn
    :  t
    -> dtype:'a Data_type.Typed.t
    -> chunk_index:int
    -> indices_and_values:(int * 'a option) list
    -> (unit, string) result
    = "rust_series_modify_optional_at_chunk_index"

  let modify_optional_at_chunk_index t ~dtype ~chunk_index ~indices_and_values =
    match
      Or_error.try_with (fun () ->
        modify_optional_at_chunk_index_exn t ~dtype ~chunk_index ~indices_and_values)
    with
    | Ok result -> result
    | Error error ->
      Error
        (sprintf
           "modify_optional_at_chunk_index_exn raised an exception. Usually this happens \
            when accessing an index out of bounds of the chunk: %s"
           (Error.to_string_hum error))
  ;;

  external compute_null_count : t -> int = "rust_series_compute_null_count"
end
