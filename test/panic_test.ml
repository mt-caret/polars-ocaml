open! Core
open Polars

let () = Common.For_testing.clear_panic_hook ()

let%expect_test "test" =
  Expect_test_helpers_core.require_does_raise [%here] (fun () ->
    Common.For_testing.panic "This is a panic");
  [%expect
    {|
    (Failure
     "Polars panicked: test panic: This is a panic\nbacktrace not captured") |}];
  Common.record_panic_backtraces ();
  Expect_test_helpers_core.require_does_raise ~hide_positions:true [%here] (fun () ->
    Common.For_testing.panic "This is a panic");
  [%expect.output]
  |> String.substr_replace_all ~pattern:"\\n" ~with_:"\n" (* Expand out newlines *)
  |> String.split_lines
  |> List.filter ~f:(Fn.non (String.is_substring ~substring:" at "))
     (* Lines containing " at " correpond to file paths, which are noisy *)
  |> String.concat_lines
  |> String.filter ~f:(Fn.non Char.is_digit)
     (* Remvoe all digits which may correspond to line/column numbers *)
  |> print_endline;
  [%expect
    {|
    (Failure
     "Polars panicked: test panic: This is a panic
    Backtrace:
       : polars_ocaml::utils::rust_record_panic_backtraces::{{closure}}::{{closure}}
       : <alloc::boxed::Box<F,A> as core::ops::function::Fn<Args>>::call
       : std::panicking::rust_panic_with_hook
       : std::panicking::begin_panic_handler::{{closure}}
       : std::sys_common::backtrace::__rust_end_short_backtrace
       : rust_begin_unwind
       : core::panicking::panic_fmt
       : polars_ocaml::misc::rust_test_panic::{{closure}}
       : std::panicking::try::do_call
       : __rust_try
      : std::panicking::try
      : std::panic::catch_unwind
      : rust_test_panic
      : camlPolars__Common__fun_
      : camlExpect_test_helpers_base__fun_
      : camlBase__Exn__protectx_
      : camlExpect_test_helpers_base__require_does_raise_
      : camlPolars_tests__Panic_test__fun_
      : camlExpect_test_collector__exec_
      : camlExpect_test_collector__fun_
      : camlPpx_inline_test_lib__time_without_resetting_random_seeds_
      : camlPpx_inline_test_lib__time_and_reset_random_seeds_
      : camlPpx_inline_test_lib__test_inner_
      : camlPolars_tests__Panic_test__entry
      : caml_program
      : caml_start_program
      : caml_startup_common
      : caml_startup_exn
      : caml_startup
      : caml_main
      : main
      : <unknown>
      : __libc_start_main
      : _start
    ") |}]
;;
