open Core
open Polars

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
  | Float32 -> Generator.float
  | Float64 -> Generator.float
  | Utf8 -> Generator.string
  | Binary -> Generator.string
  | List t ->
    value_generator t
    |> Generator.list
    |> (* TODO: nested empty lists seems to cause a crash in Polars! *)
    Generator.filter ~f:(Fn.non List.is_empty)
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
  | List t -> value_shrinker t |> Shrinker.list
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
  | List t ->
    let sexp_of_value = value_to_sexp t in
    [%sexp_of: value list] a
;;

module Series_create = struct
  type t = Args : 'a Data_type.Typed.t * 'a list -> t

  let sexp_of_t (Args (data_type, values)) =
    let sexp_of_value = value_to_sexp data_type in
    [%sexp_of: Data_type.Typed.packed * value list] (Data_type.Typed.T data_type, values)
  ;;

  let quickcheck_generator =
    let open Quickcheck.Generator.Let_syntax in
    let%bind (T data_type) = Data_type.Typed.quickcheck_generator_packed in
    let%map values =
      Quickcheck.Generator.list (value_generator data_type)
      (* |> (* TODO: nested empty lists seems to cause a crash in Polars! *)
         Quickcheck.Generator.filter ~f:(Fn.non List.is_empty) *)
    in
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

let%expect_test "test" =
  Base_quickcheck.Test.run_exn
    ~examples:[ Args (List Int16, []) ]
    (module Series_create)
    ~f:(fun (Series_create.Args (data_type, values)) ->
      ignore (Series.create data_type "series_name" values))
[@@expect.uncaught_exn
  {|
  (* CR expect_test_collector: This test expectation appears to contain a backtrace.
     This is strongly discouraged as backtraces are fragile.
     Please change this test to not include a backtrace. *)

  ("Base_quickcheck.Test.run: test failed" (input ((List Int16) ()))
    (error
      ((Failure
          "Polars panicked: index out of bounds: the len is 0 but the index is 0\
         \nBacktrace:\
         \n   0: polars_ocaml::utils::rust_record_panic_backtraces::{{closure}}::{{closure}}\
         \n             at /home/ubuntu/dev/ocaml/polars/polars-ocaml/_build/default/rust/polars-ocaml/src/utils.rs:30:1\
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
         \n             at /home/ubuntu/dev/ocaml/polars/polars-ocaml/_build/default/rust/polars-ocaml/src/series.rs:229:20\
         \n  10: polars_ocaml::series::rust_series_new::{{closure}}\
         \n             at /home/ubuntu/dev/ocaml/polars/polars-ocaml/_build/default/rust/polars-ocaml/src/series.rs:246:18\
         \n  11: std::panicking::try::do_call\
         \n             at /rustc/498553fc04f6a3fdc53412320f4e913bc53bc267/library/std/src/panicking.rs:500:40\
         \n  12: __rust_try\
         \n  13: std::panicking::try\
         \n             at /rustc/498553fc04f6a3fdc53412320f4e913bc53bc267/library/std/src/panicking.rs:464:19\
         \n  14: std::panic::catch_unwind\
         \n             at /rustc/498553fc04f6a3fdc53412320f4e913bc53bc267/library/std/src/panic.rs:142:14\
         \n  15: rust_series_new\
         \n             at /home/ubuntu/dev/ocaml/polars/polars-ocaml/_build/default/rust/polars-ocaml/src/series.rs:235:1\
         \n  16: camlPolars__Series__fun_5818\
         \n  17: camlPolars_tests__Data_type_gadt_test__fun_5332\
         \n             at /workspace_root/test/data_type_gadt_test.ml:111:13\
         \n  18: camlBase__Or_error__try_with_inner_2477\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/base.v0.16.2/_build/default/src/or_error.ml:99:9\
         \n  19: camlBase__Or_error__try_with_inner_2477\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/base.v0.16.2/_build/default/src/or_error.ml:99:9\
         \n  20: camlBase__Or_error__try_with_join_1933\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/base.v0.16.2/_build/default/src/or_error.ml:103:38\
         \n  21: camlBase_quickcheck__Test__fun_4285\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/base_quickcheck.v0.16.0/_build/default/src/test.ml:112:14\
         \n  22: camlBase__Sequence__loop_3103\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/base.v0.16.2/_build/default/src/sequence.ml:1193:14\
         \n  23: camlBase_quickcheck__Test__fun_4279\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/base_quickcheck.v0.16.0/_build/default/src/test.ml:111:6\
         \n  24: camlBase_quickcheck__Test__run_3813\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/base_quickcheck.v0.16.0/_build/default/src/test.ml:127:8\
         \n  25: camlBase_quickcheck__Test__run_exn_3995\
         \n             at /home/ubuntu/.opam/4.14.1/.opam-switch/build/base_quickcheck.v0.16.0/_build/default/src/test.ml:143:2\
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
         \n             at /workspace_root/test/data_type_gadt_test.ml:106\
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
        ("Raised by primitive operation at Polars_tests__Data_type_gadt_test.(fun) in file \"test/data_type_gadt_test.ml\", line 111, characters 13-59"
          "Called from Base__Or_error.try_with in file \"src/or_error.ml\", line 99, characters 9-15"))))
  Raised at Base__Error.raise in file "src/error.ml" (inlined), line 9, characters 14-30
  Called from Base__Or_error.ok_exn in file "src/or_error.ml", line 107, characters 17-32
  Called from Expect_test_collector.Make.Instance_io.exec in file "collector/expect_test_collector.ml", line 234, characters 12-19

  Trailing output
  ---------------
  debug: series_name, Int16, [] |}]
;;
