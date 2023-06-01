open! Core

external rust_twice : int -> int = "rust_twice"

type series

external rust_series_new : string -> int list -> series = "rust_series_new"
external rust_series_to_string_hum : series -> string = "rust_series_to_string_hum"

type lazy_frame

external rust_lazy_frame_of_parquet
  :  string
  -> (lazy_frame, string) result
  = "rust_lazy_frame_of_parquet"

external rust_lazy_frame_to_dot
  :  lazy_frame
  -> (string, string) result
  = "rust_lazy_frame_to_dot"

type data_frame

external rust_data_frame_new
  :  series list
  -> (data_frame, string) result
  = "rust_data_frame_new"

external rust_lazy_frame_to_data_frame
  :  lazy_frame
  -> (data_frame, string) result
  = "rust_lazy_frame_to_data_frame"

external rust_data_frame_to_string_hum
  :  data_frame
  -> string
  = "rust_data_frame_to_string_hum"

let%expect_test "twice" =
  print_s [%sexp (rust_twice 2 : int)];
  [%expect {| 4 |}];
  print_s [%sexp (rust_twice Int.max_value : int)];
  [%expect {| -2 |}];
  (* TODO: what's an ergonomic way to create series and dataframes easily? *)
  let series = rust_series_new "col" [ 1; 2; 3 ] in
  rust_series_to_string_hum series |> print_endline;
  [%expect {|
    shape: (3,)
    Series: 'col' [i64]
    [
    	1
    	2
    	3
    ] |}];
  let df =
    rust_data_frame_new [ series ]
    |> Result.map_error ~f:(fun error_message -> Exn.create_s [%message error_message])
    |> Result.ok_exn
  in
  rust_data_frame_to_string_hum df |> print_endline;
  [%expect
    {|
    shape: (3, 1)
    ┌─────┐
    │ col │
    │ --- │
    │ i64 │
    ╞═════╡
    │ 1   │
    │ 2   │
    │ 3   │
    └─────┘ |}]
;;
