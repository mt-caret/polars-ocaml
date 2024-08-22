open! Core
open Polars

let%expect_test "register multiple dataframes" =
  let df = Data_frame.create_exn Series.[ int "integer" [ 1; 2; 3; 4; 5 ] ] in
  Sql_context.execute_with_data_frames_exn
    ~names_and_data_frames:[ "data", df ]
    ~query:"select * from data limit 1"
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
    └─────────┘ |}]
;;

let%expect_test "register multiple dataframes" =
  let df = Data_frame.create_exn Series.[ int "integer" [ 1; 2; 3; 4; 5 ] ] in
  let df2 = Data_frame.create_exn Series.[ float "float" [ 4.; 5.; 6.; 7.; 8. ] ] in
  Sql_context.execute_with_data_frames_exn
    ~names_and_data_frames:[ "data", df; "data2", df2 ]
    ~query:"select * from data limit 1"
  |> Data_frame.to_string_hum
  |> print_endline;
  Sql_context.execute_with_data_frames_exn
    ~names_and_data_frames:[ "data", df; "data2", df2 ]
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

let%expect_test "Vstack and collect" =
  let df =
    Data_frame.create_exn Series.[ int "integer" [ 1; 2 ]; float "float" [ 1.5; 2.5 ] ]
  in
  let df2 =
    Data_frame.create_exn Series.[ int "integer" [ 4; 5 ]; floato "float" [ None; None ] ]
  in
  Sql_context.vstack_and_collect
    ~names_and_data_frames:[ "data", [ df; df2 ] ]
    ~query:"select * from data"
  |> Result.ok_or_failwith
  |> Data_frame.to_string_hum
  |> print_endline;
  [%expect
    {|
    shape: (4, 2)
    ┌─────────┬───────┐
    │ integer ┆ float │
    │ ---     ┆ ---   │
    │ i64     ┆ f64   │
    ╞═════════╪═══════╡
    │ 1       ┆ 1.5   │
    │ 2       ┆ 2.5   │
    │ 4       ┆ null  │
    │ 5       ┆ null  │
    └─────────┴───────┘ |}]
;;

let%expect_test "Vstack and execute" =
  let df =
    Data_frame.create_exn Series.[ int "integer" [ 1; 2 ]; float "float" [ 1.5; 2.5 ] ]
  in
  let df2 =
    Data_frame.create_exn Series.[ int "integer" [ 4; 5 ]; floato "float" [ None; None ] ]
  in
  let lazy_frame = Sql_context.vstack_and_execute
    ~names_and_data_frames:[ "data", [ df; df2 ] ]
    ~query:"select * from data"
  |> Result.ok_or_failwith
  in
  Lazy_frame.collect lazy_frame
  |> Result.ok_or_failwith
  |> Data_frame.to_string_hum
  |> print_endline;
  [%expect {|
    shape: (4, 2)
    ┌─────────┬───────┐
    │ integer ┆ float │
    │ ---     ┆ ---   │
    │ i64     ┆ f64   │
    ╞═════════╪═══════╡
    │ 1       ┆ 1.5   │
    │ 2       ┆ 2.5   │
    │ 4       ┆ null  │
    │ 5       ┆ null  │
    └─────────┴───────┘
    |}];
  Lazy_frame.explain lazy_frame
  |> Result.ok_or_failwith
  |> print_endline;
  [%expect {|
    FAST_PROJECT: [integer, float]
      DF ["integer", "float"]; PROJECT 2/2 COLUMNS; SELECTION: "None"
    |}]
