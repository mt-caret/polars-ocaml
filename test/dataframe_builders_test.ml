open! Core
open Polars

let%expect_test "Basic clear" =
  let df =
    Data_frame.create_exn
      Series.[ int "integer" [ 1; 2; 3; 4; 5 ]; float "float" [ 4.; 5.; 6.; 7.; 8. ] ]
  in
  let df2 = Data_frame.clear df in
  Data_frame.to_string_hum df |> print_endline;
  Data_frame.to_string_hum df2 |> print_endline;
  [%expect
    {|
    shape: (5, 2)
    ┌─────────┬───────┐
    │ integer ┆ float │
    │ ---     ┆ ---   │
    │ i64     ┆ f64   │
    ╞═════════╪═══════╡
    │ 1       ┆ 4.0   │
    │ 2       ┆ 5.0   │
    │ 3       ┆ 6.0   │
    │ 4       ┆ 7.0   │
    │ 5       ┆ 8.0   │
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
  let df =
    Data_frame.create_exn
      Series.[ int "integer" [ 1; 2; 3; 4; 5 ]; float "float" [ 4.; 5.; 6.; 7.; 8. ] ]
  in
  let df2 =
    Data_frame.create_exn
      Series.[ int "integer" [ 1; 2; 3; 4; 5 ]; float "float" [ 4.; 5.; 6.; 7.; 8. ] ]
  in
  Data_frame.vstack_exn df ~other:df2;
  Data_frame.to_string_hum df |> print_endline;
  [%expect
    {|
    shape: (10, 2)
    ┌─────────┬───────┐
    │ integer ┆ float │
    │ ---     ┆ ---   │
    │ i64     ┆ f64   │
    ╞═════════╪═══════╡
    │ 1       ┆ 4.0   │
    │ 2       ┆ 5.0   │
    │ 3       ┆ 6.0   │
    │ 4       ┆ 7.0   │
    │ …       ┆ …     │
    │ 2       ┆ 5.0   │
    │ 3       ┆ 6.0   │
    │ 4       ┆ 7.0   │
    │ 5       ┆ 8.0   │
    └─────────┴───────┘ |}]
;;

let build_a_dataframe_usage n =
  let df = Data_frame.create_exn Series.[ int "integer" []; float "float" [] ] in
  let single_row =
    Data_frame.create_exn Series.[ int "integer" [ 1 ]; float "float" [ 4. ] ]
  in
  List.init n ~f:Fn.id
  |> List.iter ~f:(fun i ->
    Data_frame.vstack_exn df ~other:single_row;
    if i % 100 = 0
    then (
      let (_ : Data_frame.t) =
        Sql_context.rust_sql_context_execute_with_data_frames_exn
          ~data_frames_with_names:[ df, "data" ]
          ~query:"select * from data limit 1"
      in
      ()));
  Data_frame.as_single_chunk_par df;
  let count =
    Sql_context.rust_sql_context_execute_with_data_frames_exn
      ~data_frames_with_names:[ df, "data" ]
      ~query:"select count(*), max(integer) from data"
  in
  Data_frame.to_string_hum count |> print_string
;;

let%bench "build and periodically query a dataframe -- 100_000 iters" =
  build_a_dataframe_usage 100_000
;;

let%bench "build and periodically query a dataframe -- 1_000_000 iters" =
  build_a_dataframe_usage 1_000_000
;;

let%bench "build and periodically query a dataframe -- 10_000_000 iters" =
  build_a_dataframe_usage 10_000_000
;;

let one_chunk_at_a_time_usage n =
  let df = Data_frame.create_exn Series.[ int "integer" []; float "float" [] ] in
  let df2 = ref (Data_frame.create_exn Series.[ int "integer" []; float "float" [] ]) in
  let single_row =
    Data_frame.create_exn Series.[ int "integer" [ 1 ]; float "float" [ 4. ] ]
  in
  List.init n ~f:Fn.id
  |> List.iter ~f:(fun i ->
    Data_frame.vstack_exn !df2 ~other:single_row;
    if i % 1000 = 0
    then (
      Data_frame.as_single_chunk_par !df2;
      Data_frame.vstack_exn df ~other:!df2;
      df2 := Data_frame.clear !df2));
  let count =
    Sql_context.rust_sql_context_execute_with_data_frames_exn
      ~data_frames_with_names:[ df, "data" ]
      ~query:"select count(*), max(integer) from data"
  in
  Data_frame.to_string_hum count |> print_string
;;

let%bench "one chunk at a time -- 100_000 iters" = build_a_dataframe_usage 100_000
let%bench "one chunk at a time -- 1_000_000 iters" = build_a_dataframe_usage 1_000_000
let%bench "one chunk at a time -- 10_000_000 iters" = build_a_dataframe_usage 10_000_000
