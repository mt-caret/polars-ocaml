open! Core
open Polars

let create_basic_df () =
  Data_frame.create_exn Series.[ int "integer" [ 1 ]; float "float" [ 4. ] ]
;;

let%expect_test "Basic clear" =
  let df = create_basic_df () in
  let df2 = Data_frame.clear df in
  Data_frame.to_string_hum df |> print_endline;
  Data_frame.to_string_hum df2 |> print_endline;
  [%expect
    {|
    shape: (1, 2)
    ┌─────────┬───────┐
    │ integer ┆ float │
    │ ---     ┆ ---   │
    │ i64     ┆ f64   │
    ╞═════════╪═══════╡
    │ 1       ┆ 4.0   │
    └─────────┴───────┘
    shape: (0, 2)
    ┌─────────┬───────┐
    │ integer ┆ float │
    │ ---     ┆ ---   │
    │ i64     ┆ f64   │
    ╞═════════╪═══════╡
    └─────────┴───────┘ |}]
;;

let%expect_test "Basic vstack" =
  let df = create_basic_df () in
  let df2 = create_basic_df () in
  Data_frame.vstack_exn df ~other:df2;
  Data_frame.to_string_hum df |> print_endline;
  [%expect
    {|
    shape: (2, 2)
    ┌─────────┬───────┐
    │ integer ┆ float │
    │ ---     ┆ ---   │
    │ i64     ┆ f64   │
    ╞═════════╪═══════╡
    │ 1       ┆ 4.0   │
    │ 1       ┆ 4.0   │
    └─────────┴───────┘ |}]
;;
