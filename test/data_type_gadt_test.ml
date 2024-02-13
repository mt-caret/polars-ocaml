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

module Series_create = struct
  type t = Args : 'a Data_type.Typed.t * 'a list -> t

  let compare (Args (data_type1, values1)) (Args (data_type2, values2)) =
    let (module Value) = Data_type_quickcheck.data_type_value data_type1 in
    match Data_type.Typed.strict_type_equal data_type1 data_type2 with
    | None ->
      [%compare: Data_type.t]
        (Data_type.Typed.to_untyped data_type1)
        (Data_type.Typed.to_untyped data_type1)
    | Some T -> List.compare Value.compare values1 values2
  ;;

  let sexp_of_t (Args (data_type, values)) =
    let (module Value) = Data_type_quickcheck.data_type_value data_type in
    let sexp_of_value = Value.sexp_of_t in
    [%sexp_of: Data_type.Typed.packed * value list] (Data_type.Typed.T data_type, values)
  ;;

  let quickcheck_generator =
    let open Quickcheck.Generator.Let_syntax in
    let%bind (T data_type) = Data_type.Typed.quickcheck_generator_packed in
    let (module Value) = Data_type_quickcheck.data_type_value data_type in
    let%map values = Quickcheck.Generator.list Value.quickcheck_generator in
    Args (data_type, values)
  ;;

  let quickcheck_shrinker =
    Quickcheck.Shrinker.create (fun (Args (data_type, values)) ->
      let (module Value) = Data_type_quickcheck.data_type_value data_type in
      let value_shrinker = Base_quickcheck.Shrinker.list Value.quickcheck_shrinker in
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
      let (module Value) = Data_type_quickcheck.data_type_value data_type in
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
          let value_equal = Comparable.equal Value.compare in
          assert (value_equal value (Series.get_exn data_type series i)));
        (* Test Series.create' *)
        let series = Series.create' data_type "series_name" (Array.of_list values) in
        let values' = Series.to_list data_type series in
        let args' = Series_create.Args (data_type, values') in
        [%test_result: Series_create.t] ~expect:args' args;
        let args' =
          Series_create.Args
            (data_type, Series.to_option_list data_type series |> List.filter_opt)
        in
        [%test_result: Series_create.t] ~expect:args' args;
        List.iteri values' ~f:(fun i value ->
          let value_equal = Comparable.equal Value.compare in
          assert (value_equal value (Series.get_exn data_type series i))))
;;

(* TODO: there's a *lot* of duplication with the Series_create module; perhaps
   functorizing this would clean things up... *)
module Series_createo = struct
  type t = Args : 'a Data_type.Typed.t * 'a option list -> t

  let compare (Args (data_type1, values1)) (Args (data_type2, values2)) =
    let (module Value1) = Data_type_quickcheck.data_type_value data_type1 in
    match Data_type.Typed.strict_type_equal data_type1 data_type2 with
    | None ->
      [%compare: Data_type.t]
        (Data_type.Typed.to_untyped data_type1)
        (Data_type.Typed.to_untyped data_type1)
    | Some T -> List.compare (Option.compare Value1.compare) values1 values2
  ;;

  let sexp_of_t (Args (data_type, values)) =
    let (module Value) = Data_type_quickcheck.data_type_value data_type in
    let sexp_of_value = Value.sexp_of_t in
    [%sexp_of: Data_type.Typed.packed * value option list]
      (Data_type.Typed.T data_type, values)
  ;;

  let quickcheck_generator =
    let open Quickcheck.Generator.Let_syntax in
    let%bind (T data_type) = Data_type.Typed.quickcheck_generator_packed in
    let (module Value) = Data_type_quickcheck.data_type_value data_type in
    let%map values =
      Quickcheck.Generator.list
        (Base_quickcheck.Generator.option Value.quickcheck_generator)
    in
    Args (data_type, values)
  ;;

  let quickcheck_shrinker =
    Quickcheck.Shrinker.create (fun (Args (data_type, values)) ->
      let (module Value) = Data_type_quickcheck.data_type_value data_type in
      let value_shrinker =
        Base_quickcheck.Shrinker.list
          (Base_quickcheck.Shrinker.option Value.quickcheck_shrinker)
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
      let (module Value) = Data_type_quickcheck.data_type_value data_type in
      match Data_type.Typed.flatten_custom data_type with
      | List (List _) | Custom { data_type = List (List _); f = _; f_inverse = _ } -> ()
      | _ ->
        (* Test Series.createo *)
        let series = Series.createo data_type "series_name" values in
        let values' = Series.to_option_list data_type series in
        let args' = Series_createo.Args (data_type, values') in
        [%test_result: Series_createo.t] ~expect:args' args;
        List.iteri values' ~f:(fun i value ->
          let value_equal = Option.equal (Comparable.equal Value.compare) in
          assert (value_equal value (Series.get data_type series i)));
        (* Test Series.createo' *)
        let series = Series.createo' data_type "series_name" (Array.of_list values) in
        let values' = Series.to_option_list data_type series in
        let args' = Series_createo.Args (data_type, values') in
        [%test_result: Series_createo.t] ~expect:args' args;
        List.iteri values' ~f:(fun i value ->
          let value_equal = Option.equal (Comparable.equal Value.compare) in
          assert (value_equal value (Series.get data_type series i))))
;;

module Expr_lit = struct
  type t = Args : 'a Data_type.Typed.t * 'a -> t

  let compare (Args (data_type1, value1)) (Args (data_type2, value2)) =
    let (module Value1) = Data_type_quickcheck.data_type_value data_type1 in
    match Data_type.Typed.strict_type_equal data_type1 data_type2 with
    | None ->
      [%compare: Data_type.t]
        (Data_type.Typed.to_untyped data_type1)
        (Data_type.Typed.to_untyped data_type1)
    | Some T -> Value1.compare value1 value2
  ;;

  let sexp_of_t (Args (data_type, value)) =
    let (module Value) = Data_type_quickcheck.data_type_value data_type in
    let sexp_of_value = Value.sexp_of_t in
    [%sexp_of: Data_type.Typed.packed * value] (Data_type.Typed.T data_type, value)
  ;;

  let quickcheck_generator =
    let open Quickcheck.Generator.Let_syntax in
    let%bind (T data_type) = Data_type.Typed.quickcheck_generator_packed in
    let (module Value) = Data_type_quickcheck.data_type_value data_type in
    let%map value = Value.quickcheck_generator in
    Args (data_type, value)
  ;;

  let quickcheck_shrinker =
    Quickcheck.Shrinker.create (fun (Args (data_type, value)) ->
      let (module Value) = Data_type_quickcheck.data_type_value data_type in
      let value_shrinker = Value.quickcheck_shrinker in
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
