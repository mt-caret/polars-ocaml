open! Core
open! Polars

let%expect_test "Series.map" =
  let series = Series.int "ints" [ 1; -2; 3; -4 ] in
  Series.map Int64 Int64 series ~f:(Option.map ~f:(fun x -> x + 1)) |> Series.print;
  [%expect
    {|
    shape: (4,)
    Series: 'ints' [i64]
    [
    	2
    	-1
    	4
    	-3
    ] |}];
  let series = Series.into "ints" [ Some 1; Some (-2); Some 3; None ] in
  (* Raising an exception from within a closure works. *)
  (* TODO: unfortunately, exceptions raised in the closure will cause
     backtraces to be lost. this is pretty unfortunate, but at the same time it
     doesn't seem clear that OCaml exposes a way for C/Rust to recover the
     backtrace associated with calling an OCaml function.

     We could wrap [f] in [Series.map] with a try-catch and explicitly load the
     backtrace via [Backtrace.Exn.most_recent_for_exn] and thread it through. *)
  Expect_test_helpers_core.require_does_raise [%here] (fun () ->
    Series.map Int64 Int64 series ~f:(fun _ -> failwith "Some exception"));
  [%expect {|
    (Failure "Some exception") |}]
;;
