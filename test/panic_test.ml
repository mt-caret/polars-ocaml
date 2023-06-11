open! Core
open Polars

let () = Common.For_testing.install_panic_hook ~suppress_backtrace:true

(* Define a custom exception and raise that instead of the generic Failure exception *)
let%expect_test "test" =
  Expect_test_helpers_core.require_does_raise [%here] (fun () ->
    Common.For_testing.raise_exception "This is an exception");
  [%expect {|
    (Failure "This is an exception") |}]
;;

let%expect_test "test" =
  Expect_test_helpers_core.require_does_raise [%here] (fun () ->
    Common.For_testing.panic "This is a panic");
  [%expect {|
    (Failure "Rust panic: This is a panic") |}]
;;
