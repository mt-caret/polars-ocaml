open! Core
open Polars

let%expect_test "read values from dataframe" =
  let init_num_list f = List.init 20 ~f in
  let df =
    Data_frame.create_exn
      Series.
        [ int "integers" (init_num_list Fn.id)
        ; float
            "floats"
            (init_num_list (fun i -> if Int.( = ) i 0 then Float.nan else Float.of_int i))
        ; string "strings" (init_num_list Int.to_string)
        ]
  in
  Data_frame.print df;
  [%expect
    {|
    shape: (20, 3)
    ┌──────────┬────────┬─────────┐
    │ integers ┆ floats ┆ strings │
    │ ---      ┆ ---    ┆ ---     │
    │ i64      ┆ f64    ┆ str     │
    ╞══════════╪════════╪═════════╡
    │ 0        ┆ NaN    ┆ 0       │
    │ 1        ┆ 1.0    ┆ 1       │
    │ 2        ┆ 2.0    ┆ 2       │
    │ 3        ┆ 3.0    ┆ 3       │
    │ …        ┆ …      ┆ …       │
    │ 16       ┆ 16.0   ┆ 16      │
    │ 17       ┆ 17.0   ┆ 17      │
    │ 18       ┆ 18.0   ┆ 18      │
    │ 19       ┆ 19.0   ┆ 19      │
    └──────────┴────────┴─────────┘ |}];
  let series_as_list name =
    df |> Data_frame.column_exn ~name |> Series.to_typed_list_exn
  in
  print_s [%message (series_as_list "integers" : Series.typed_list)];
  print_s [%message (series_as_list "floats" : Series.typed_list)];
  print_s [%message (series_as_list "strings" : Series.typed_list)];
  [%expect
    {|
    ("series_as_list \"integers\""
     (Int
      ((0) (1) (2) (3) (4) (5) (6) (7) (8) (9) (10) (11) (12) (13) (14) (15)
       (16) (17) (18) (19))))
    ("series_as_list \"floats\""
     (Float
      ((NAN) (1) (2) (3) (4) (5) (6) (7) (8) (9) (10) (11) (12) (13) (14)
       (15) (16) (17) (18) (19))))
    ("series_as_list \"strings\""
     (String
      ((0) (1) (2) (3) (4) (5) (6) (7) (8) (9) (10) (11) (12) (13) (14) (15)
       (16) (17) (18) (19)))) |}]
;;
