open! Core
open Polars

let%expect_test "Basic modify -- float" =
  let series = Series.float "float" [ 0.; 1.; 2. ] in
  let df = Data_frame.create_exn [ series ] in
  Series.clear series;
  Data_frame.Expert.modify_series_at_chunk_index
    df
    ~dtype:Float64
    ~series_index:0
    ~chunk_index:0
    ~indices_and_values:[ 0, 10.0; 2, 20.0 ]
  |> Result.ok_or_failwith;
  Data_frame.to_string_hum df |> print_endline;
  [%expect
    {|
    shape: (3, 1)
    ┌───────┐
    │ float │
    │ ---   │
    │ f64   │
    ╞═══════╡
    │ 10.0  │
    │ 1.0   │
    │ 20.0  │
    └───────┘ |}]
;;

let%expect_test "Basic modify -- float option" =
  let series = Series.float "float" [ 0.; 1.; 2. ] in
  let series_option = Series.floato "float_option" [ Some 0.; Some 1.; None ] in
  let series2 = Series.float "float" [ 0.; 1.; 2. ] in
  let series_option2 = Series.floato "float_option" [ Some 0.; Some 1.; None ] in
  let df = Data_frame.create_exn [ series; series_option ] in
  let chunk2 = Data_frame.create_exn [ series2; series_option2 ] in
  Data_frame.vstack_exn df ~other:chunk2;
  Series.clear series;
  Series.clear series_option;
  Series.clear series2;
  Series.clear series_option2;
  Data_frame.Expert.clear_mut chunk2;
  let series = Data_frame.column_exn df ~name:"float_option" in
  Series.Expert.compute_null_count series |> Int.to_string |> print_endline;
  Series.clear series;
  [%expect {| 2 |}];
  Data_frame.Expert.modify_optional_series_at_chunk_index
    df
    ~dtype:Float64
    ~series_index:1
    ~chunk_index:0
    ~indices_and_values:[ 0, None; 1, None; 2, Some 20.0 ]
  |> Result.ok_or_failwith;
  Data_frame.to_string_hum df |> print_endline;
  [%expect
    {|
    shape: (6, 2)
    ┌───────┬──────────────┐
    │ float ┆ float_option │
    │ ---   ┆ ---          │
    │ f64   ┆ f64          │
    ╞═══════╪══════════════╡
    │ 0.0   ┆ null         │
    │ 1.0   ┆ null         │
    │ 2.0   ┆ 20.0         │
    │ 0.0   ┆ 0.0          │
    │ 1.0   ┆ 1.0          │
    │ 2.0   ┆ null         │
    └───────┴──────────────┘ |}];
  Data_frame.column_exn df ~name:"float_option"
  |> Series.Expert.compute_null_count
  |> Int.to_string
  |> print_endline;
  [%expect {| 3 |}]
;;
