open! Core
open Polars

let%expect_test "register multiple dataframes" =
  let df = Data_frame.create_exn Series.[ int "integer" [ 1; 2; 3; 4; 5 ] ] in
  let df2 = Data_frame.create_exn Series.[ float "float" [ 4.; 5.; 6.; 7.; 8. ] ] in
  Sql_context.rust_sql_context_execute_with_data_frames_exn
    ~data_frames_with_names:[ df, "data"; df2, "data2" ]
    ~query:"select * from data limit 1"
  |> Data_frame.to_string_hum
  |> print_endline;
  Sql_context.rust_sql_context_execute_with_data_frames_exn
    ~data_frames_with_names:[ df, "data"; df2, "data2" ]
    ~query:"select * from data2 limit 1"
  |> Data_frame.to_string_hum
  |> print_endline;
  [%expect
    {|
    shape: (1, 1)
    ┌─────────┐
    │ integer │
    │ ---     │
    │ i64     │
    ╞═════════╡
    │ 1       │
    └─────────┘
    shape: (1, 1)
    ┌───────┐
    │ float │
    │ ---   │
    │ f64   │
    ╞═══════╡
    │ 4.0   │
    └───────┘ |}]
;;
