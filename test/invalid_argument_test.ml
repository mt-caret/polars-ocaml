open! Core
open! Polars

let%expect_test "type conversion" =
  let s = Series.int "a" [ 1; 2; 3; 4; 5 ] in
  Series.print s;
  [%expect
    {|
    shape: (5,)
    Series: 'a' [i64]
    [
    	1
    	2
    	3
    	4
    	5
    ] |}];
  Expect_test_helpers_core.require_does_raise [%here] (fun () ->
    Series.head s ~length:(-1) |> Series.print);
  [%expect
    {|
    (Failure
     "Polars panicked: Failed to convert OCaml<Option<i64>> (from Option<isize>) to Rust<Option<usize>>: TryFromIntError(())\nbacktrace not captured") |}]
;;
