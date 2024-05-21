open Core
open Polars

let%expect_test "min_max" =
  let df = Data_frame.create_exn Series.[ int "ints" [ 1; -2; 3; -4 ] ] in
  let min_max =
    Data_frame.select_exn
      df
      ~exprs:
        Expr.
          [ col "ints" |> min |> alias ~name:"min"
          ; col "ints" |> max |> alias ~name:"max"
          ; col "ints" |> arg_min |> alias ~name:"arg_min"
          ; col "ints" |> arg_max |> alias ~name:"arg_max"
          ]
  in
  Data_frame.print min_max;
  [%expect
    {|
    shape: (1, 4)
    ┌─────┬─────┬─────────┬─────────┐
    │ min ┆ max ┆ arg_min ┆ arg_max │
    │ --- ┆ --- ┆ ---     ┆ ---     │
    │ i64 ┆ i64 ┆ u32     ┆ u32     │
    ╞═════╪═════╪═════════╪═════════╡
    │ -4  ┆ 3   ┆ 3       ┆ 2       │
    └─────┴─────┴─────────┴─────────┘ |}]
;;

let%expect_test "Rounding" =
  let df =
    Data_frame.create_exn [ Series.float "floats" [ -2.; -1.5; 0.; 1.5; 2. ] ]
    |> Data_frame.with_columns_exn
         ~exprs:
           Expr.
             [ col "floats" |> ceil |> alias ~name:"ceil"
             ; col "floats" |> floor |> alias ~name:"floor"
             ]
  in
  Data_frame.print df;
  [%expect
    {|
    shape: (5, 3)
    ┌────────┬──────┬───────┐
    │ floats ┆ ceil ┆ floor │
    │ ---    ┆ ---  ┆ ---   │
    │ f64    ┆ f64  ┆ f64   │
    ╞════════╪══════╪═══════╡
    │ -2.0   ┆ -2.0 ┆ -2.0  │
    │ -1.5   ┆ -1.0 ┆ -2.0  │
    │ 0.0    ┆ 0.0  ┆ 0.0   │
    │ 1.5    ┆ 2.0  ┆ 1.0   │
    │ 2.0    ┆ 2.0  ┆ 2.0   │
    └────────┴──────┴───────┘ |}]
;;

let%expect_test "Exponents" =
  let df =
    Data_frame.create_exn [ Series.float "floats" [ -2.; -1.5; 0.; 1.5; 2. ] ]
    |> Data_frame.with_columns_exn
         ~exprs:
           Expr.
             [ col "floats" |> Fn.flip pow (float 2.) |> alias ~name:"square"
             ; col "floats" |> Fn.flip pow (float 0.5) |> alias ~name:"sqrt"
             ]
  in
  Data_frame.print df;
  [%expect
    {|
    shape: (5, 3)
    ┌────────┬────────┬──────────┐
    │ floats ┆ square ┆ sqrt     │
    │ ---    ┆ ---    ┆ ---      │
    │ f64    ┆ f64    ┆ f64      │
    ╞════════╪════════╪══════════╡
    │ -2.0   ┆ 4.0    ┆ NaN      │
    │ -1.5   ┆ 2.25   ┆ NaN      │
    │ 0.0    ┆ 0.0    ┆ 0.0      │
    │ 1.5    ┆ 2.25   ┆ 1.224745 │
    │ 2.0    ┆ 4.0    ┆ 1.414214 │
    └────────┴────────┴──────────┘ |}]
;;

let%expect_test "Clamping" =
  let df =
    Data_frame.create_exn
      [ Series.float "floats" [ -3.; -1.; 0.; 1.; 3. ]
      ; Series.int "ints" [ -3; -1; 0; 1; 3 ]
      ]
    |> Data_frame.with_columns_exn
         ~exprs:
           Expr.
             [ col "ints" |> clip_min_int ~min:0 |> alias ~name:"int_min_0"
             ; col "ints" |> clip_max_int ~max:1 |> alias ~name:"int_max_1"
             ; col "floats" |> clip_min_float ~min:0. |> alias ~name:"float_min_0"
             ; col "floats" |> clip_max_float ~max:1. |> alias ~name:"float_max_1"
             ]
  in
  Data_frame.print df;
  [%expect
    {|
    shape: (5, 6)
    ┌────────┬──────┬───────────┬───────────┬─────────────┬─────────────┐
    │ floats ┆ ints ┆ int_min_0 ┆ int_max_1 ┆ float_min_0 ┆ float_max_1 │
    │ ---    ┆ ---  ┆ ---       ┆ ---       ┆ ---         ┆ ---         │
    │ f64    ┆ i64  ┆ i64       ┆ i64       ┆ f64         ┆ f64         │
    ╞════════╪══════╪═══════════╪═══════════╪═════════════╪═════════════╡
    │ -3.0   ┆ -3   ┆ 0         ┆ -3        ┆ 0.0         ┆ -3.0        │
    │ -1.0   ┆ -1   ┆ 0         ┆ -1        ┆ 0.0         ┆ -1.0        │
    │ 0.0    ┆ 0    ┆ 0         ┆ 0         ┆ 0.0         ┆ 0.0         │
    │ 1.0    ┆ 1    ┆ 1         ┆ 1         ┆ 1.0         ┆ 1.0         │
    │ 3.0    ┆ 3    ┆ 3         ┆ 1         ┆ 3.0         ┆ 1.0         │
    └────────┴──────┴───────────┴───────────┴─────────────┴─────────────┘ |}]
;;

let%expect_test "nan" =
  let nan_df =
    Data_frame.create_exn Series.[ float "value" [ 1.; Float.nan; Float.nan; 3. ] ]
    |> Data_frame.with_columns_exn
         ~exprs:
           [ Expr.col "value" |> Expr.is_nan |> Expr.alias ~name:"is_nan"
           ; Expr.col "value" |> Expr.is_not_nan |> Expr.alias ~name:"is_not_nan"
           ; Expr.col "value" |> Expr.is_finite |> Expr.alias ~name:"is_finite"
           ; Expr.col "value" |> Expr.is_infinite |> Expr.alias ~name:"is_infinite"
           ]
  in
  Data_frame.print nan_df;
  [%expect
    {|
    shape: (4, 5)
    ┌───────┬────────┬────────────┬───────────┬─────────────┐
    │ value ┆ is_nan ┆ is_not_nan ┆ is_finite ┆ is_infinite │
    │ ---   ┆ ---    ┆ ---        ┆ ---       ┆ ---         │
    │ f64   ┆ bool   ┆ bool       ┆ bool      ┆ bool        │
    ╞═══════╪════════╪════════════╪═══════════╪═════════════╡
    │ 1.0   ┆ false  ┆ true       ┆ true      ┆ false       │
    │ NaN   ┆ true   ┆ false      ┆ false     ┆ false       │
    │ NaN   ┆ true   ┆ false      ┆ false     ┆ false       │
    │ 3.0   ┆ false  ┆ true       ┆ true      ┆ false       │
    └───────┴────────┴────────────┴───────────┴─────────────┘ |}]
;;

let%expect_test "join_asof" =
  let thirds = Series.float "a" [ 33.3; 66.6; 99.9 ] in
  let tens = Series.float "b" [ 10.; 20.; 30.; 40.; 50.; 60.; 70.; 80.; 90.; 100. ] in
  let tens = Data_frame.create_exn [ tens ] |> Data_frame.lazy_ in
  let thirds = Data_frame.create_exn [ thirds ] |> Data_frame.lazy_ in
  List.iter [ `Backward; `Forward; `Nearest ] ~f:(fun strategy ->
    let joined =
      Lazy_frame.join'
        thirds
        ~other:tens
        ~left_on:Expr.[ col "a" |> set_sorted_flag ~sorted:`Ascending ]
        ~right_on:Expr.[ col "b" |> set_sorted_flag ~sorted:`Ascending ]
        ~how:(As_of { strategy; tolerance = None; left_by = None; right_by = None })
    in
    print_s ([%sexp_of: [ `Backward | `Forward | `Nearest ]] strategy);
    Lazy_frame.collect_exn joined |> Data_frame.print);
  [%expect
    {|
    Backward
    shape: (3, 2)
    ┌──────┬──────┐
    │ a    ┆ b    │
    │ ---  ┆ ---  │
    │ f64  ┆ f64  │
    ╞══════╪══════╡
    │ 33.3 ┆ 30.0 │
    │ 66.6 ┆ 60.0 │
    │ 99.9 ┆ 90.0 │
    └──────┴──────┘
    Forward
    shape: (3, 2)
    ┌──────┬───────┐
    │ a    ┆ b     │
    │ ---  ┆ ---   │
    │ f64  ┆ f64   │
    ╞══════╪═══════╡
    │ 33.3 ┆ 40.0  │
    │ 66.6 ┆ 70.0  │
    │ 99.9 ┆ 100.0 │
    └──────┴───────┘
    Nearest
    shape: (3, 2)
    ┌──────┬───────┐
    │ a    ┆ b     │
    │ ---  ┆ ---   │
    │ f64  ┆ f64   │
    ╞══════╪═══════╡
    │ 33.3 ┆ 30.0  │
    │ 66.6 ┆ 70.0  │
    │ 99.9 ┆ 100.0 │
    └──────┴───────┘ |}]
;;

let%expect_test "take" =
  let values = Series.float "val" [ 1.; 2.; 3.; 4.; 5.; 6. ] in
  let idxs = Series.int "idx" [ 1; 0; 1; 2; 0; 2 ] in
  let ldf = Data_frame.create_exn [ values; idxs ] |> Data_frame.lazy_ in
  let filtered =
    Lazy_frame.filter
      ldf
      ~predicate:
        Expr.(
          (* Remove index 0, keep index 1 and 2 *)
          series (Series.bool "filter" [ false; true; true ]) |> take ~idx:(col "idx"))
    |> Lazy_frame.collect_exn
  in
  Data_frame.print filtered;
  [%expect
    {|
    shape: (4, 2)
    ┌─────┬─────┐
    │ val ┆ idx │
    │ --- ┆ --- │
    │ f64 ┆ i64 │
    ╞═════╪═════╡
    │ 1.0 ┆ 1   │
    │ 3.0 ┆ 1   │
    │ 4.0 ┆ 2   │
    │ 6.0 ┆ 2   │
    └─────┴─────┘ |}]
;;

let%expect_test "is_in" =
  let values = Series.string "val" [ "1"; "2"; "3"; "4"; "5"; "6" ] in
  let other = Series.string "is_in" [ "2"; "3"; "5" ] in
  let is_in =
    Data_frame.create_exn [ values ]
    |> Data_frame.with_columns_exn
         ~exprs:Expr.[ is_in (col "val") ~other:(series other) |> alias ~name:"is_in" ]
  in
  Data_frame.print is_in;
  [%expect
    {|
    shape: (6, 2)
    ┌─────┬───────┐
    │ val ┆ is_in │
    │ --- ┆ ---   │
    │ str ┆ bool  │
    ╞═════╪═══════╡
    │ 1   ┆ false │
    │ 2   ┆ true  │
    │ 3   ┆ true  │
    │ 4   ┆ false │
    │ 5   ┆ true  │
    │ 6   ┆ false │
    └─────┴───────┘
    |}]
;;

let%expect_test "is_in_left" =
  let values =
    Series.create (List Utf8) "val" [ [ "a"; "b"; "c" ]; [ "2"; "3"; "4" ]; [ "3"; "a" ] ]
  in
  let is_in =
    Data_frame.create_exn [ values ]
    |> Data_frame.with_columns_exn
         ~exprs:Expr.[ is_in (string "a") ~other:(col "val") |> alias ~name:"contains_a" ]
  in
  Data_frame.print is_in;
  [%expect
    {|
    shape: (3, 2)
    ┌─────────────────┬────────────┐
    │ val             ┆ contains_a │
    │ ---             ┆ ---        │
    │ list[str]       ┆ bool       │
    ╞═════════════════╪════════════╡
    │ ["a", "b", "c"] ┆ true       │
    │ ["2", "3", "4"] ┆ false      │
    │ ["3", "a"]      ┆ true       │
    └─────────────────┴────────────┘
    |}]
;;
