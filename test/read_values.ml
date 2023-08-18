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
        ; create
            (List Boolean)
            "boolean_lists"
            Int.(init_num_list (fun i -> List.init (i / 2) ~f:(fun j -> (i + j) % 2 = 0)))
        ]
  in
  Data_frame.print df;
  [%expect
    {|
      shape: (20, 4)
      ┌──────────┬────────┬─────────┬────────────────────────┐
      │ integers ┆ floats ┆ strings ┆ boolean_lists          │
      │ ---      ┆ ---    ┆ ---     ┆ ---                    │
      │ i64      ┆ f64    ┆ str     ┆ list[bool]             │
      ╞══════════╪════════╪═════════╪════════════════════════╡
      │ 0        ┆ NaN    ┆ 0       ┆ []                     │
      │ 1        ┆ 1.0    ┆ 1       ┆ []                     │
      │ 2        ┆ 2.0    ┆ 2       ┆ [true]                 │
      │ 3        ┆ 3.0    ┆ 3       ┆ [false]                │
      │ …        ┆ …      ┆ …       ┆ …                      │
      │ 16       ┆ 16.0   ┆ 16      ┆ [true, false, … false] │
      │ 17       ┆ 17.0   ┆ 17      ┆ [false, true, … true]  │
      │ 18       ┆ 18.0   ┆ 18      ┆ [true, false, … true]  │
      │ 19       ┆ 19.0   ┆ 19      ┆ [false, true, … false] │
      └──────────┴────────┴─────────┴────────────────────────┘ |}];
  let series_as_list data_type name =
    df |> Data_frame.column_exn ~name |> Series.to_list data_type
  in
  print_s [%message "" ~integers:(series_as_list Int64 "integers" : int list)];
  print_s [%message "" ~floats:(series_as_list Float64 "floats" : float list)];
  print_s [%message "" ~strings:(series_as_list Utf8 "strings" : string list)];
  print_s
    [%message
      "" ~boolean_lists:(series_as_list (List Boolean) "boolean_lists" : bool list list)];
  [%expect
    {|
      (integers (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19))
      (floats (NAN 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19))
      (strings (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19))
      (boolean_lists
       (() () (true) (false) (true false) (false true) (true false true)
        (false true false) (true false true false) (false true false true)
        (true false true false true) (false true false true false)
        (true false true false true false) (false true false true false true)
        (true false true false true false true)
        (false true false true false true false)
        (true false true false true false true false)
        (false true false true false true false true)
        (true false true false true false true false true)
        (false true false true false true false true false))) |}]
;;
