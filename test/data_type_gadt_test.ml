open Core
open Polars

let%expect_test "unit tests" =
  let series = Series.createo Int64 "series_name" [ Some 1; None; Some 2 ] in
  Series.to_option_list Int64 series |> [%sexp_of: int option list] |> print_s;
  [%expect {|
    ((1) () (2)) |}];
  (* Trying to convert to non-null list when there are nulls should raise *)
  Expect_test_helpers_core.require_does_raise [%here] (fun () ->
    Series.to_list Int64 series);
  [%expect
    {|
    (Failure
     "Polars panicked: Series contains 1 null values, expected none\nbacktrace not captured") |}];
  (* Trying to convert to list of different type should raise *)
  Expect_test_helpers_core.require_does_raise [%here] (fun () ->
    Series.to_option_list Float64 series);
  [%expect
    {|
    (Failure
     "Polars panicked: data types don't match: invalid series dtype: expected `Float64`, got `i64`\nbacktrace not captured") |}]
;;

(* TODO: perhaps these things should be bundled in a module and there should be a single function like:
   {[
     val data_type_value
       :  'a Data_type.Typed.t
       -> (module Data_type_value with type t = 'a)
   ]}
*)
let rec value_generator : type a. a Data_type.Typed.t -> a Quickcheck.Generator.t =
  fun (type a) (t : a Data_type.Typed.t) : a Quickcheck.Generator.t ->
  let open Base_quickcheck in
  let uint bits = Generator.int_inclusive 0 (Int.(2 ** bits) - 1) in
  let int bits =
    Generator.int_inclusive (-Int.(2 ** (bits - 1))) (Int.(2 ** (bits - 1)) - 1)
  in
  match t with
  | Boolean -> Generator.bool
  | UInt8 -> uint 8
  | UInt16 -> uint 16
  | UInt32 -> uint 32
  | UInt64 -> Generator.int |> Generator.filter ~f:Int.is_non_negative
  | Int8 -> int 8
  | Int16 -> int 16
  | Int32 -> int 32
  | Int64 -> Generator.int
  | Float32 ->
    Generator.float
    |> Generator.map ~f:(fun x ->
      (* By converting to a 32bit float and back, we generate doubles which do
         not change values even when converted to a 32bit float. *)
      Int32.bits_of_float x |> Int32.float_of_bits)
  | Float64 -> Generator.float
  | Utf8 ->
    Generator.string
    |> (* Core.String doesn't have a [is_valid_utf_8] function :( *)
    Generator.filter ~f:Stdlib.String.is_valid_utf_8
  | Binary -> Generator.string
  | Date ->
    let open Generator.Let_syntax in
    let%map year =
      (* Date.t doesn't support dates outside of years 0-9999 *)
      Generator.int_inclusive 0 9999
    and month = Generator.int_inclusive 1 12 >>| Month.of_int_exn
    and day = Generator.int_inclusive 1 28 in
    Date.create_exn ~y:year ~m:month ~d:day |> Common.Naive_date.of_date
  | List t -> value_generator t |> Generator.list
  | Custom { data_type; f; f_inverse = _ } ->
    value_generator data_type |> Generator.map ~f
;;

let rec value_shrinker : type a. a Data_type.Typed.t -> a Quickcheck.Shrinker.t =
  fun (type a) (t : a Data_type.Typed.t) : a Quickcheck.Shrinker.t ->
  let open Base_quickcheck in
  match t with
  | Boolean -> Shrinker.bool
  | UInt8 -> Shrinker.int |> Shrinker.filter ~f:Int.is_non_negative
  | UInt16 -> Shrinker.int |> Shrinker.filter ~f:Int.is_non_negative
  | UInt32 -> Shrinker.int |> Shrinker.filter ~f:Int.is_non_negative
  | UInt64 -> Shrinker.int |> Shrinker.filter ~f:Int.is_non_negative
  | Int8 -> Shrinker.int
  | Int16 -> Shrinker.int
  | Int32 -> Shrinker.int
  | Int64 -> Shrinker.int
  | Float32 -> Shrinker.float
  | Float64 -> Shrinker.float
  | Utf8 -> Shrinker.string
  | Binary -> Shrinker.string
  | Date -> Shrinker.atomic
  | List t ->
    value_shrinker t |> Shrinker.list |> Shrinker.filter ~f:(Fn.non List.is_empty)
  | Custom { data_type; f; f_inverse } ->
    value_shrinker data_type |> Shrinker.map ~f ~f_inverse
;;

let rec value_to_sexp : type a. a Data_type.Typed.t -> a -> Sexp.t =
  fun (type a) (t : a Data_type.Typed.t) (a : a) ->
  match t with
  | Boolean -> [%sexp_of: bool] a
  | UInt8 -> [%sexp_of: int] a
  | UInt16 -> [%sexp_of: int] a
  | UInt32 -> [%sexp_of: int] a
  | UInt64 -> [%sexp_of: int] a
  | Int8 -> [%sexp_of: int] a
  | Int16 -> [%sexp_of: int] a
  | Int32 -> [%sexp_of: int] a
  | Int64 -> [%sexp_of: int] a
  | Float32 -> [%sexp_of: float] a
  | Float64 -> [%sexp_of: float] a
  | Utf8 -> [%sexp_of: string] a
  | Binary -> [%sexp_of: string] a
  | Date -> [%sexp_of: Date.t] (Common.Naive_date.to_date_exn a)
  | List t ->
    let sexp_of_value = value_to_sexp t in
    [%sexp_of: value list] a
  | Custom { data_type; f = _; f_inverse } -> value_to_sexp data_type (f_inverse a)
;;

let rec value_compare : type a. a Data_type.Typed.t -> a -> a -> int =
  fun (type a) (t : a Data_type.Typed.t) (a : a) (b : a) ->
  match t with
  | Boolean -> [%compare: bool] a b
  | UInt8 -> [%compare: int] a b
  | UInt16 -> [%compare: int] a b
  | UInt32 -> [%compare: int] a b
  | UInt64 -> [%compare: int] a b
  | Int8 -> [%compare: int] a b
  | Int16 -> [%compare: int] a b
  | Int32 -> [%compare: int] a b
  | Int64 -> [%compare: int] a b
  | Float32 -> [%compare: float] a b
  | Float64 -> [%compare: float] a b
  | Utf8 -> [%compare: string] a b
  | Binary -> [%compare: string] a b
  | Date -> Comparable.lift [%compare: Date.t] ~f:Common.Naive_date.to_date_exn a b
  | List t -> List.compare (value_compare t) a b
  | Custom { data_type; f = _; f_inverse } ->
    Comparable.lift (value_compare data_type) ~f:f_inverse a b
;;

module Series_create = struct
  type t = Args : 'a Data_type.Typed.t * 'a list -> t

  let compare (Args (data_type1, values1)) (Args (data_type2, values2)) =
    match Data_type.Typed.strict_type_equal data_type1 data_type2 with
    | None ->
      [%compare: Data_type.t]
        (Data_type.Typed.to_untyped data_type1)
        (Data_type.Typed.to_untyped data_type1)
    | Some T -> List.compare (value_compare data_type1) values1 values2
  ;;

  let sexp_of_t (Args (data_type, values)) =
    let sexp_of_value = value_to_sexp data_type in
    [%sexp_of: Data_type.Typed.packed * value list] (Data_type.Typed.T data_type, values)
  ;;

  let quickcheck_generator =
    let open Quickcheck.Generator.Let_syntax in
    let%bind (T data_type) = Data_type.Typed.quickcheck_generator_packed in
    let%map values = Quickcheck.Generator.list (value_generator data_type) in
    Args (data_type, values)
  ;;

  let quickcheck_shrinker =
    Quickcheck.Shrinker.create (fun (Args (data_type, values)) ->
      let value_shrinker = Base_quickcheck.Shrinker.list (value_shrinker data_type) in
      Quickcheck.Shrinker.shrink value_shrinker values
      |> Sequence.map ~f:(fun values -> Args (data_type, values)))
  ;;

  let quickcheck_observer =
    Quickcheck.Observer.unmap
      Data_type.Typed.quickcheck_observer_packed
      ~f:(fun (Args (data_type, _values)) -> T data_type)
  ;;
end

(* TODO: below code causes panic *)
(* {[
     let%expect_test "demonstrate panic" =
       ignore (Series.create (List (List Date)) "test" [ [] ] : Series.t);
       [%expect.unreachable]
     ;;
   ]} *)

let%expect_test "Series.create and Series.create' doesn't raise" =
  Base_quickcheck.Test.run_exn
    (module Series_create)
    ~f:(fun (Series_create.Args (data_type, values) as args) ->
      (* TODO: polars has a bug where it panics when creating a series where
         the dtype is a [(List (List Date))] with an empty list in it
         i.e. [Series.create (List (List Date)) "test" [ [] ] ]. It's very
         difficult to work around this, so we just skip this test for now.
         See lib.rs for a rust reproduction of this bug. *)
      match Data_type.Typed.flatten_custom data_type with
      | List (List _) | Custom { data_type = List (List _); f = _; f_inverse = _ } -> ()
      | _ ->
        (* Test Series.create *)
        let series = Series.create data_type "series_name" values in
        let values' = Series.to_list data_type series in
        let args' = Series_create.Args (data_type, values') in
        [%test_result: Series_create.t] ~expect:args' args;
        let args' =
          Series_create.Args
            (data_type, Series.to_option_list data_type series |> List.filter_opt)
        in
        [%test_result: Series_create.t] ~expect:args' args;
        List.iteri values' ~f:(fun i value ->
          let value_equal = Comparable.equal (value_compare data_type) in
          assert (value_equal value (Series.get_exn data_type series i)));
        (* Test Series.create' *)
        let series =
          Series.create' data_type "series_name" (Uniform_array.of_list values)
        in
        let values' = Series.to_list data_type series in
        let args' = Series_create.Args (data_type, values') in
        [%test_result: Series_create.t] ~expect:args' args;
        let args' =
          Series_create.Args
            (data_type, Series.to_option_list data_type series |> List.filter_opt)
        in
        [%test_result: Series_create.t] ~expect:args' args;
        List.iteri values' ~f:(fun i value ->
          let value_equal = Comparable.equal (value_compare data_type) in
          assert (value_equal value (Series.get_exn data_type series i))))
;;

(* TODO: there's a *lot* of duplication with the Series_create module; perhaps
   functorizing this would clean things up... *)
module Series_createo = struct
  type t = Args : 'a Data_type.Typed.t * 'a option list -> t

  let compare (Args (data_type1, values1)) (Args (data_type2, values2)) =
    match Data_type.Typed.strict_type_equal data_type1 data_type2 with
    | None ->
      [%compare: Data_type.t]
        (Data_type.Typed.to_untyped data_type1)
        (Data_type.Typed.to_untyped data_type1)
    | Some T -> List.compare (Option.compare (value_compare data_type1)) values1 values2
  ;;

  let sexp_of_t (Args (data_type, values)) =
    let sexp_of_value = value_to_sexp data_type in
    [%sexp_of: Data_type.Typed.packed * value option list]
      (Data_type.Typed.T data_type, values)
  ;;

  let quickcheck_generator =
    let open Quickcheck.Generator.Let_syntax in
    let%bind (T data_type) = Data_type.Typed.quickcheck_generator_packed in
    let%map values =
      Quickcheck.Generator.list
        (Base_quickcheck.Generator.option (value_generator data_type))
    in
    Args (data_type, values)
  ;;

  let quickcheck_shrinker =
    Quickcheck.Shrinker.create (fun (Args (data_type, values)) ->
      let value_shrinker =
        Base_quickcheck.Shrinker.list
          (Base_quickcheck.Shrinker.option (value_shrinker data_type))
      in
      Quickcheck.Shrinker.shrink value_shrinker values
      |> Sequence.map ~f:(fun values -> Args (data_type, values)))
  ;;

  let quickcheck_observer =
    Quickcheck.Observer.unmap
      Data_type.Typed.quickcheck_observer_packed
      ~f:(fun (Args (data_type, _values)) -> T data_type)
  ;;
end

let%expect_test "Series.createo and Series.createo' doesn't raise" =
  Base_quickcheck.Test.run_exn
    (module Series_createo)
    ~f:(fun (Series_createo.Args (data_type, values) as args) ->
      match Data_type.Typed.flatten_custom data_type with
      | List (List _) | Custom { data_type = List (List _); f = _; f_inverse = _ } -> ()
      | _ ->
        (* Test Series.createo *)
        let series = Series.createo data_type "series_name" values in
        let values' = Series.to_option_list data_type series in
        let args' = Series_createo.Args (data_type, values') in
        [%test_result: Series_createo.t] ~expect:args' args;
        List.iteri values' ~f:(fun i value ->
          let value_equal = Option.equal (Comparable.equal (value_compare data_type)) in
          assert (value_equal value (Series.get data_type series i)));
        (* Test Series.createo' *)
        let series =
          Series.createo' data_type "series_name" (Uniform_array.of_list values)
        in
        let values' = Series.to_option_list data_type series in
        let args' = Series_createo.Args (data_type, values') in
        [%test_result: Series_createo.t] ~expect:args' args;
        List.iteri values' ~f:(fun i value ->
          let value_equal = Option.equal (Comparable.equal (value_compare data_type)) in
          assert (value_equal value (Series.get data_type series i))))
;;

module Expr_lit = struct
  type t = Args : 'a Data_type.Typed.t * 'a -> t

  let compare (Args (data_type1, value1)) (Args (data_type2, value2)) =
    match Data_type.Typed.strict_type_equal data_type1 data_type2 with
    | None ->
      [%compare: Data_type.t]
        (Data_type.Typed.to_untyped data_type1)
        (Data_type.Typed.to_untyped data_type1)
    | Some T -> (value_compare data_type1) value1 value2
  ;;

  let sexp_of_t (Args (data_type, value)) =
    let sexp_of_value = value_to_sexp data_type in
    [%sexp_of: Data_type.Typed.packed * value] (Data_type.Typed.T data_type, value)
  ;;

  let quickcheck_generator =
    let open Quickcheck.Generator.Let_syntax in
    let%bind (T data_type) = Data_type.Typed.quickcheck_generator_packed in
    let%map value = value_generator data_type in
    Args (data_type, value)
  ;;

  let quickcheck_shrinker =
    Quickcheck.Shrinker.create (fun (Args (data_type, value)) ->
      let value_shrinker = value_shrinker data_type in
      Quickcheck.Shrinker.shrink value_shrinker value
      |> Sequence.map ~f:(fun value -> Args (data_type, value)))
  ;;

  let quickcheck_observer =
    Quickcheck.Observer.unmap
      Data_type.Typed.quickcheck_observer_packed
      ~f:(fun (Args (data_type, _value)) -> T data_type)
  ;;
end

let%expect_test "Expr.lit roundtrip" =
  Base_quickcheck.Test.run_exn
    (module Expr_lit)
    ~f:(fun (Expr_lit.Args (data_type, value) as args) ->
      match Data_type.Typed.flatten_custom data_type with
      | List (List _) | Custom { data_type = List (List _); f = _; f_inverse = _ } -> ()
      | _ ->
        let value' =
          Data_frame.create_exn []
          |> Data_frame.select_exn
               ~exprs:Expr.[ lit data_type value |> alias ~name:"col" ]
          |> Data_frame.column_exn ~name:"col"
          |> Series.to_list data_type
        in
        assert (List.length value' = 1);
        let args' = Expr_lit.Args (data_type, List.hd_exn value') in
        [%test_result: Expr_lit.t] ~expect:args' args)
;;
