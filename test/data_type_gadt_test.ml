open Core
open Polars

let%expect_test "unit tests" =
  let series = Series.createo Int64 "series_name" [ Some 1; None; Some 2 ] in
  Series.to_option_list Int64 series |> [%sexp_of: int option list] |> print_s;
  [%expect {| ((1) () (2)) |}];
  (* Trying to convert to non-null list when there are nulls should raise *)
  Expect_test_helpers_core.require_does_raise [%here] (fun () ->
    ignore (Series.to_list Int64 series));
  [%expect
    {|
    (Failure
     "Polars panicked: Series contains 1 null values, expected none\nbacktrace not captured") |}];
  (* Trying to convert to list of different type should raise *)
  Expect_test_helpers_core.require_does_raise [%here] (fun () ->
    ignore (Series.to_option_list Float64 series));
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
  | List t ->
    value_generator t
    |> (* Polars currently doesn't support passing empty Vec<Series> to Series::new.
          See test in rust/polars-ocaml/src/lib.rs. *)
    Generator.list_non_empty
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
    let%map values = Quickcheck.Generator.list_non_empty (value_generator data_type) in
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

let%expect_test "Series.create doesn't raise" =
  Base_quickcheck.Test.run_exn
    (module Series_create)
    ~f:(fun (Series_create.Args (data_type, values) as args) ->
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
        assert (value_equal value (Series.get_exn data_type series i))));
  [%expect.unreachable]
[@@expect.uncaught_exn
  {|
  (* CR expect_test_collector: This test expectation appears to contain a backtrace.
     This is strongly discouraged as backtraces are fragile.
     Please change this test to not include a backtrace. *)

  ("Base_quickcheck.Test.run: test failed" (input ((List (List Date)) ()))
    (error
      ((Failure
          "Polars panicked: index out of bounds: the len is 0 but the index is 0\
         \nBacktrace:\
         \n   0: polars_ocaml::utils::rust_record_panic_backtraces::{{closure}}::{{closure}}\
         \n             at /home/ubuntu/dev/ocaml/polars/polars-ocaml/_build/default/rust/polars-ocaml/src/utils.rs:31:1\
         \n   1: <alloc::boxed::Box<F,A> as core::ops::function::Fn<Args>>::call\
         \n             at /rustc/498553fc04f6a3fdc53412320f4e913bc53bc267/library/alloc/src/boxed.rs:1999:9\
         \n   2: std::panicking::rust_panic_with_hook\
         \n             at /rustc/498553fc04f6a3fdc53412320f4e913bc53bc267/library/std/src/panicking.rs:709:13\
         \n   3: std::panicking::begin_panic_handler::{{closure}}\
         \n             at /rustc/498553fc04f6a3fdc53412320f4e913bc53bc267/library/std/src/panicking.rs:597:13\
         \n   4: std::sys_common::backtrace::__rust_end_short_backtrace\
         \n             at /rustc/498553fc04f6a3fdc53412320f4e913bc53bc267/library/std/src/sys_common/backtrace.rs:151:18\
         \n   5: rust_begin_unwind\
         \n             at /rustc/498553fc04f6a3fdc53412320f4e913bc53bc267/library/std/src/panicking.rs:593:5\
         \n   6: core::panicking::panic_fmt\
         \n             at /rustc/498553fc04f6a3fdc53412320f4e913bc53bc267/library/core/src/panicking.rs:67:14\
         \n   7: core::panicking::panic_bounds_check\
         \n             at /rustc/498553fc04f6a3fdc53412320f4e913bc53bc267/library/core/src/panicking.rs:162:5\
         \n   8: <polars_core::series::Series as polars_core::named_from::NamedFrom<T,polars_core::datatypes::ListType>>::new\
         \n             at /home/ubuntu/.cargo/registry/src/index.crates.io-6f17d22bba15001f/polars-core-0.31.1/src/named_from.rs:126:18\
         \n   9: polars_ocaml::series::series_new\
         \n             at /home/ubuntu/dev/ocaml/polars/polars-ocaml/_build/default/rust/polars-ocaml/src/series.rs:188:20\
         \n  10: polars_ocaml::series::rust_series_new::{{closure}}\
         \n             at /home/ubuntu/dev/ocaml/polars/polars-ocaml/_build/default/rust/polars-ocaml/src/series.rs:205:18\
         \n  11: std::panicking::try::do_call\
         \n             at /rustc/498553fc04f6a3fdc53412320f4e913bc53bc267/library/std/src/panicking.rs:500:40\
         \n  12: __rust_try\
         \n  13: std::panicking::try\
         \n             at /rustc/498553fc04f6a3fdc53412320f4e913bc53bc267/library/std/src/panicking.rs:464:19\
         \n  14: std::panic::catch_unwind\
         \n             at /rustc/498553fc04f6a3fdc53412320f4e913bc53bc267/library/std/src/panic.rs:142:14\
         \n  15: rust_series_new\
         \n             at /home/ubuntu/dev/ocaml/polars/polars-ocaml/_build/default/rust/polars-ocaml/src/series.rs:194:1\
         \n  16: camlPolars__Series__fun_4440\
         \n  17: camlPolars_tests__Data_type_gadt_test__fun_7627\
         \n             at /workspace_root/test/data_type_gadt_test.ml:181:19\
         \n  18: camlBase__Or_error__try_with_inner_2477\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/base.v0.16.2/_build/default/src/or_error.ml:99:9\
         \n  19: camlBase__Or_error__try_with_inner_2477\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/base.v0.16.2/_build/default/src/or_error.ml:99:9\
         \n  20: camlBase__Or_error__try_with_join_1933\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/base.v0.16.2/_build/default/src/or_error.ml:103:38\
         \n  21: camlBase_quickcheck__Test__loop_2499\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/base_quickcheck.v0.16.0/_build/default/src/test.ml:83:16\
         \n  22: camlBase_quickcheck__Test__fun_4279\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/base_quickcheck.v0.16.0/_build/default/src/test.ml:119:25\
         \n  23: camlBase_quickcheck__Test__run_3813\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/base_quickcheck.v0.16.0/_build/default/src/test.ml:127:8\
         \n  24: camlBase_quickcheck__Test__run_exn_3995\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/base_quickcheck.v0.16.0/_build/default/src/test.ml:143:2\
         \n  25: camlPolars_tests__Data_type_gadt_test__fun_7625\
         \n             at /workspace_root/test/data_type_gadt_test.ml:178:2\
         \n  26: camlExpect_test_collector__exec_1988\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/ppx_expect.v0.16.0/_build/default/collector/expect_test_collector.ml:234:12\
         \n  27: camlExpect_test_collector__fun_2607\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/ppx_expect.v0.16.0/_build/default/collector/expect_test_collector.ml:283:11\
         \n  28: camlPpx_inline_test_lib__time_without_resetting_random_seeds_2082\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/ppx_inline_test.v0.16.0/_build/default/runtime-lib/ppx_inline_test_lib.ml:405:11\
         \n  29: camlPpx_inline_test_lib__time_and_reset_random_seeds_2263\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/ppx_inline_test.v0.16.0/_build/default/runtime-lib/ppx_inline_test_lib.ml:420:15\
         \n  30: camlPpx_inline_test_lib__test_inner_2527\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/ppx_inline_test.v0.16.0/_build/default/runtime-lib/ppx_inline_test_lib.ml:546:35\
         \n  31: camlPolars_tests__Data_type_gadt_test__entry\
         \n             at /workspace_root/test/data_type_gadt_test.ml:177\
         \n  32: caml_program\
         \n  33: caml_start_program\
         \n  34: caml_startup_common\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/ocaml-base-compiler.4.14.1/runtime/startup_nat.c:160:9\
         \n  35: caml_startup_exn\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/ocaml-base-compiler.4.14.1/runtime/startup_nat.c:167:10\
         \n  36: caml_startup\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/ocaml-base-compiler.4.14.1/runtime/startup_nat.c:172:15\
         \n  37: caml_main\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/ocaml-base-compiler.4.14.1/runtime/startup_nat.c:179:3\
         \n  38: main\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/ocaml-base-compiler.4.14.1/runtime/main.c:37:3\
         \n  39: <unknown>\
         \n  40: __libc_start_main\
         \n  41: _start\
         \n")
        ("Raised by primitive operation at Polars_tests__Data_type_gadt_test.(fun) in file \"test/data_type_gadt_test.ml\", line 181, characters 19-63"
          "Called from Base__Or_error.try_with in file \"src/or_error.ml\", line 99, characters 9-15"))))
  Raised at Base__Error.raise in file "src/error.ml" (inlined), line 9, characters 14-30
  Called from Base__Or_error.ok_exn in file "src/or_error.ml", line 107, characters 17-32
  Called from Polars_tests__Data_type_gadt_test.(fun) in file "test/data_type_gadt_test.ml", line 178, characters 2-744
  Called from Expect_test_collector.Make.Instance_io.exec in file "collector/expect_test_collector.ml", line 234, characters 12-19 |}]
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
      Quickcheck.Generator.list_non_empty
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

let%expect_test "Series.createo doesn't raise" =
  Base_quickcheck.Test.run_exn
    (module Series_createo)
    ~f:(fun (Series_createo.Args (data_type, values) as args) ->
      let series = Series.createo data_type "series_name" values in
      let values' = Series.to_option_list data_type series in
      let args' = Series_createo.Args (data_type, values') in
      [%test_result: Series_createo.t] ~expect:args' args;
      List.iteri values' ~f:(fun i value ->
        let value_equal = Option.equal (Comparable.equal (value_compare data_type)) in
        assert (value_equal value (Series.get data_type series i))));
  [%expect.unreachable]
[@@expect.uncaught_exn
  {|
  (* CR expect_test_collector: This test expectation appears to contain a backtrace.
     This is strongly discouraged as backtraces are fragile.
     Please change this test to not include a backtrace. *)

  ("Base_quickcheck.Test.run: test failed"
    (input ((List (List (List Date))) (((((5325-08-22)))))))
    (error
      ((Failure
          "Polars panicked: data types don't match: invalid series dtype: expected `Date`, got `i32`\
         \nbacktrace not captured")
        ("Raised by primitive operation at Polars_tests__Data_type_gadt_test.(fun) in file \"test/data_type_gadt_test.ml\", line 348, characters 34-65"
          "Called from Base__List.iteri.(fun) in file \"src/list.ml\", line 630, characters 7-12"
          "Called from Base__List0.fold in file \"src/list0.ml\", line 37, characters 27-37"
          "Called from Base__List.iteri in file \"src/list.ml\", line 629, characters 5-62"
          "Called from Base__Or_error.try_with in file \"src/or_error.ml\", line 99, characters 9-15"))))
  Raised at Base__Error.raise in file "src/error.ml" (inlined), line 9, characters 14-30
  Called from Base__Or_error.ok_exn in file "src/or_error.ml", line 107, characters 17-32
  Called from Polars_tests__Data_type_gadt_test.(fun) in file "test/data_type_gadt_test.ml", line 339, characters 2-574
  Called from Expect_test_collector.Make.Instance_io.exec in file "collector/expect_test_collector.ml", line 234, characters 12-19 |}]
;;
