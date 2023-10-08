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
  Expect_test_helpers_core.require_does_raise [%here] (fun () ->
    Series.map Int64 Int64 series ~f:(fun _ -> raise_s [%message "Some exception"]));
  [%expect.output]
  |> String.substr_replace_all ~pattern:"\\n" ~with_:"\n" (* Expand out newlines *)
  |> String.split_lines
  (* Don't bother printing the backtrace since it's super unstable across
     dev/release profiles, OSes, architectures, etc. *)
  |> Fn.flip List.take 3
  |> String.concat_lines
  |> print_endline;
  [%expect {|
    (Failure
     "Polars panicked: Empty exception
    Backtrace: |}]
;;
