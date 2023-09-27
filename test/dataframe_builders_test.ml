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

(* Building a dataframe using ocaml-interop can be tricky, and there are ways to do it
   that result in O(n^2) runtime due to the way that dataframe refcounting works.
   The following function is intended to be close to linear runtime: vstack
   dataframes one by one, and then call as_single_chunk_par once at the end.
*)
let build_a_dataframe_usage n =
  let df = Data_frame.create_exn Series.[ int "integer" []; float "float" [] ] in
  let single_row = create_basic_df () in
  List.init n ~f:Fn.id
  |> List.iter ~f:(fun i ->
    Data_frame.vstack_exn df ~other:single_row;
    if i % 100 = 0
    then (
      let (_ : Data_frame.t) =
        Sql_context.execute_with_data_frames_exn
          ~names_and_data_frames:[ "data", df ]
          ~query:"select * from data limit 1"
      in
      ()));
  Data_frame.as_single_chunk_par df;
  let count =
    Sql_context.execute_with_data_frames_exn
      ~names_and_data_frames:[ "data", df ]
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

(* The following function is also intended to be close to linear runtime: use a helper
   dataframe to batch rows into chunks of size 1000, and vstack each chunk one at a
   time to [df]
*)
let one_chunk_at_a_time_usage n =
  let df = Data_frame.create_exn Series.[ int "integer" []; float "float" [] ] in
  let df2 = ref (Data_frame.create_exn Series.[ int "integer" []; float "float" [] ]) in
  let single_row = create_basic_df () in
  List.init n ~f:Fn.id
  |> List.iter ~f:(fun i ->
    Data_frame.vstack_exn !df2 ~other:single_row;
    if i % 1000 = 0
    then (
      Data_frame.as_single_chunk_par !df2;
      Data_frame.vstack_exn df ~other:!df2;
      df2 := Data_frame.clear !df2));
  let count =
    Sql_context.execute_with_data_frames_exn
      ~names_and_data_frames:[ "data", df ]
      ~query:"select count(*), max(integer) from data"
  in
  Data_frame.to_string_hum count |> print_string
;;

let%bench "one chunk at a time -- 100_000 iters" = build_a_dataframe_usage 100_000
let%bench "one chunk at a time -- 1_000_000 iters" = build_a_dataframe_usage 1_000_000
let%bench "one chunk at a time -- 10_000_000 iters" = build_a_dataframe_usage 10_000_000
