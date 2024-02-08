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
  |> Fn.flip List.take 3
     (* Don't bother printing the backtrace since it's super unstable across
        dev/release profiles, OSes, architectures, etc. *)
  |> String.concat_lines
  |> print_endline;
  [%expect
    {|
    (Failure
     "Polars panicked: test panic: This is a panic
    Backtrace: |}]
;;
