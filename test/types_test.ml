open! Core
open Polars

module type Testable = sig
  type t [@@deriving quickcheck, compare, sexp_of]

  module For_testing : sig
    val to_rust_and_back : t -> t
  end
end

let run_roundtrip_test (type t) ?examples (module T : Testable with type t = t) =
  Base_quickcheck.Test.run_exn
    ?examples
    (module T)
    ~f:(fun t -> [%test_result: T.t] ~expect:t (T.For_testing.to_rust_and_back t))
;;

let%expect_test "Fill_null_strategy.t" = run_roundtrip_test (module Fill_null_strategy)

let%expect_test "Fill_null_strategy.t (invalid values)" =
  let test_raise t =
    Expect_test_helpers_core.require_does_raise [%here] (fun () ->
      ignore (Fill_null_strategy.For_testing.to_rust_and_back t : Fill_null_strategy.t))
  in
  test_raise (Forward (Some (-1)));
  [%expect {| (Failure "Failed conversion to u32 Some(-1)") |}];
  test_raise (Backward (Some (Int.pow 2 32)));
  [%expect {| (Failure "Failed conversion to u32 Some(4294967296)") |}]
;;
