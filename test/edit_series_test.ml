open! Core
open Polars

module Edit_series = struct
  type ('data_type, 'value_type) value_kind =
    | Optional : ('data_type, 'data_type option) value_kind
    | Non_null : ('data_type, 'data_type) value_kind

  type t =
    | Args :
        { data_type : 'data_type Data_type.Typed.t
        ; initial_values : 'value_type list
        ; indices_and_values : (int * 'value_type) list
        ; value_kind : ('data_type, 'value_type) value_kind
        }
        -> t

  let sexp_of_t (Args { data_type; initial_values; indices_and_values; value_kind; _ }) =
    let (module Value) = Data_type_quickcheck.data_type_value data_type in
    match value_kind with
    | Optional ->
      [%message
        (T data_type : Data_type.Typed.packed)
          (initial_values : Value.t option list)
          (indices_and_values : (int * Value.t option) list)]
    | Non_null ->
      [%message
        (T data_type : Data_type.Typed.packed)
          (initial_values : Value.t list)
          (indices_and_values : (int * Value.t) list)]
  ;;

  let lists_generator value_generator =
    let open Quickcheck.Generator.Let_syntax in
    let%bind initial_values = Quickcheck.Generator.list value_generator in
    let index_generator =
      Quickcheck.Generator.map Int.quickcheck_generator ~f:(fun i ->
        i % List.length initial_values)
    in
    let index_and_value_generator =
      Quickcheck.Generator.tuple2 index_generator value_generator
    in
    let%map indices_and_values =
      match List.length initial_values with
      | 0 -> Quickcheck.Generator.return []
      | _ -> Quickcheck.Generator.list index_and_value_generator
    in
    initial_values, indices_and_values
  ;;

  let quickcheck_generator_optional =
    let open Quickcheck.Generator.Let_syntax in
    let%bind (T data_type) = Data_type.Typed.quickcheck_generator_packed in
    let (module Value) = Data_type_quickcheck.data_type_value data_type in
    let%map initial_values, indices_and_values =
      lists_generator [%quickcheck.generator: Value.t option]
    in
    Args { data_type; initial_values; indices_and_values; value_kind = Optional }
  ;;

  let quickcheck_generator_not_null =
    let open Quickcheck.Generator.Let_syntax in
    let%bind (T data_type) = Data_type.Typed.quickcheck_generator_packed in
    let (module Value) = Data_type_quickcheck.data_type_value data_type in
    let%map initial_values, indices_and_values =
      lists_generator [%quickcheck.generator: Value.t]
    in
    Args { data_type; initial_values; indices_and_values; value_kind = Non_null }
  ;;

  let quickcheck_generator =
    let open Quickcheck.Generator.Let_syntax in
    let%bind required_or_optional = Quickcheck.Generator.bool in
    match required_or_optional with
    | true -> quickcheck_generator_not_null
    | false -> quickcheck_generator_optional
  ;;

  let quickcheck_shrinker =
    Quickcheck.Shrinker.create
      (fun (Args { data_type; initial_values; indices_and_values; value_kind }) ->
         let (module Value) = Data_type_quickcheck.data_type_value data_type in
         match value_kind with
         | Optional ->
           let module T = struct
             type t =
               { initial_values : Value.t option list
               ; indices_and_values : (int * Value.t option) list
               }
             [@@deriving quickcheck]
           end
           in
           let%map.Sequence { initial_values; indices_and_values } =
             Quickcheck.Shrinker.shrink
               T.quickcheck_shrinker
               { T.initial_values; indices_and_values }
           in
           Args { data_type; initial_values; indices_and_values; value_kind }
         | Non_null ->
           let module T = struct
             type t =
               { initial_values : Value.t list
               ; indices_and_values : (int * Value.t) list
               }
             [@@deriving quickcheck]
           end
           in
           let%map.Sequence { initial_values; indices_and_values } =
             Quickcheck.Shrinker.shrink
               T.quickcheck_shrinker
               { T.initial_values; indices_and_values }
           in
           Args { data_type; initial_values; indices_and_values; value_kind })
  ;;

  let quickcheck_observer =
    Quickcheck.Observer.unmap
      Data_type.Typed.quickcheck_observer_packed
      ~f:(fun (Args { data_type; _ }) -> T data_type)
  ;;
end

let%expect_test "" =
  Common.For_testing.clear_panic_hook ();
  Base_quickcheck.Test.run_exn
    (module Edit_series)
    ~f:
      (fun
        (Edit_series.Args { data_type; initial_values; indices_and_values; value_kind })
      ->
      let try_modify () =
        match value_kind with
        | Optional ->
          let series = Series.createo data_type "test" initial_values in
          Series.Expert.modify_optional_at_chunk_index
            series
            ~dtype:data_type
            ~chunk_index:0
            ~indices_and_values
          |> Result.ok_or_failwith
        | Non_null ->
          let series = Series.create data_type "test" initial_values in
          Series.Expert.modify_at_chunk_index
            series
            ~dtype:data_type
            ~chunk_index:0
            ~indices_and_values
          |> Result.ok_or_failwith
      in
      let expect_fail thunk =
        match Or_error.try_with thunk with
        | Ok () ->
          raise_s
            [%message
              "Expected unsupported datatype to fail: "
                (T data_type : Data_type.Typed.packed)]
        | Error _ -> ()
      in
      let try_modify_non_option () =
        match value_kind with
        | Optional -> expect_fail try_modify
        | Non_null -> try_modify ()
      in
      match data_type with
      | Time | Datetime _ | Duration _ | List _ | Utf8 | Binary | Date | Custom _ ->
        expect_fail try_modify
      | Boolean -> try_modify_non_option ()
      | UInt8
      | UInt16
      | UInt32
      | UInt64
      | Int8
      | Int16
      | Int32
      | Int64
      | Float32
      | Float64 -> try_modify ())
;;

let%expect_test "Error handling" =
  Common.For_testing.clear_panic_hook ();
  let print_error = function
    | Ok _ -> raise_s [%message "Expected error"]
    | Error error -> print_endline error
  in
  let series = Series.float "float" [ 0.; 1.; 2. ] in
  Series.Expert.modify_at_chunk_index
    series
    ~dtype:Int64
    ~chunk_index:0
    ~indices_and_values:[ 0, 0; 2, 20 ]
  |> print_error;
  [%expect {|
    modify_chunk_at_index: unable to downcast to type T |}];
  let series = Series.int "int" [ 0; 1; 2 ] in
  Series.Expert.modify_at_chunk_index
    series
    ~dtype:Float64
    ~chunk_index:0
    ~indices_and_values:[ 0, 1.0; 2, 20.0 ]
  |> print_error;
  [%expect {|
      modify_chunk_at_index: unable to downcast to type T |}];
  let series = Series.float "float" [ 0.0; 1.0; 2.0 ] in
  Series.Expert.modify_at_chunk_index
    series
    ~dtype:Float64
    ~chunk_index:0
    ~indices_and_values:[ 3, 20.0 ]
  |> print_error;
  [%expect
    {|
      modify_at_chunk_index_exn raised an exception. Usually this happens when accessing an index out of bounds of the chunk or passing in a value outside of the domain of dtype: (Failure
        "Polars panicked: index out of bounds: the len is 3 but the index is 3\
       \nbacktrace not captured") |}];
  Series.Expert.modify_at_chunk_index
    series
    ~dtype:Float64
    ~chunk_index:0
    ~indices_and_values:[ -1, 20.0 ]
  |> print_error;
  [%expect
    {|
        modify_at_chunk_index_exn raised an exception. Usually this happens when accessing an index out of bounds of the chunk or passing in a value outside of the domain of dtype: (Failure
          "Polars panicked: index out of bounds: the len is 3 but the index is 18446744073709551615\
         \nbacktrace not captured") |}];
  let series = Series.create Data_type.Typed.UInt8 "negative uint" [ 0 ] in
  Series.Expert.modify_at_chunk_index
    series
    ~dtype:UInt8
    ~chunk_index:0
    ~indices_and_values:[ 0, -1 ]
  |> print_error;
  [%expect
    {|
    modify_at_chunk_index_exn raised an exception. Usually this happens when accessing an index out of bounds of the chunk or passing in a value outside of the domain of dtype: (Failure
      "Polars panicked: called `Result::unwrap()` on an `Err` value: TryFromIntError(())\
     \nbacktrace not captured") |}]
;;

let%expect_test "modify bool" =
  let series = Series.create Boolean "test" [ true; true; true ] in
  Series.Expert.modify_at_chunk_index
    series
    ~dtype:Boolean
    ~chunk_index:0
    ~indices_and_values:[ 1, false ]
  |> Result.ok_or_failwith;
  let as_list = Series.to_list Boolean series in
  print_s [%message (as_list : bool list)];
  [%expect {| (as_list (true false true)) |}]
;;
