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
  |> Fn.flip List.take 7
     (* Only take first few lines which are stable across dev and release builds *)
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
       : std::panicking::begin_panic_handler::{{closure}} |}]
;;
