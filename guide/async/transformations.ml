open! Core
open Async
open! Polars
open Polars_async

(* Examples from https://pola-rs.github.io/polars-book/user-guide/transformations/joins/ *)
let%expect_test "Joins" =
  let df_customers =
    Data_frame.create_exn
      Series.
        [ int "customer_id" [ 1; 2; 3 ]; string "name" [ "Alice"; "Bob"; "Charlie" ] ]
  in
  Data_frame.print df_customers;
  [%expect
    {|
    shape: (3, 2)
    ┌─────────────┬─────────┐
    │ customer_id ┆ name    │
    │ ---         ┆ ---     │
    │ i64         ┆ str     │
    ╞═════════════╪═════════╡
    │ 1           ┆ Alice   │
    │ 2           ┆ Bob     │
    │ 3           ┆ Charlie │
    └─────────────┴─────────┘ |}];
  let df_orders =
    Data_frame.create_exn
      Series.
        [ string "order_id" [ "a"; "b"; "c" ]
        ; int "customer_id" [ 1; 2; 2 ]
        ; int "amount" [ 100; 200; 300 ]
        ]
  in
  Data_frame.print df_orders;
  [%expect
    {|
    shape: (3, 3)
    ┌──────────┬─────────────┬────────┐
    │ order_id ┆ customer_id ┆ amount │
    │ ---      ┆ ---         ┆ ---    │
    │ str      ┆ i64         ┆ i64    │
    ╞══════════╪═════════════╪════════╡
    │ a        ┆ 1           ┆ 100    │
    │ b        ┆ 2           ┆ 200    │
    │ c        ┆ 2           ┆ 300    │
    └──────────┴─────────────┴────────┘ |}];
  let%bind df_inner_join =
    Data_frame.lazy_ df_customers
    |> Lazy_frame.join
         ~other:(Data_frame.lazy_ df_orders)
         ~on:Expr.[ col "customer_id" ]
         ~how:Inner
    |> Lazy_frame.collect_exn
  in
  Data_frame.print df_inner_join;
  [%expect
    {|
    shape: (3, 4)
    ┌─────────────┬───────┬──────────┬────────┐
    │ customer_id ┆ name  ┆ order_id ┆ amount │
    │ ---         ┆ ---   ┆ ---      ┆ ---    │
    │ i64         ┆ str   ┆ str      ┆ i64    │
    ╞═════════════╪═══════╪══════════╪════════╡
    │ 1           ┆ Alice ┆ a        ┆ 100    │
    │ 2           ┆ Bob   ┆ b        ┆ 200    │
    │ 2           ┆ Bob   ┆ c        ┆ 300    │
    └─────────────┴───────┴──────────┴────────┘ |}];
  let%bind df_left_join =
    Data_frame.lazy_ df_customers
    |> Lazy_frame.join
         ~other:(Data_frame.lazy_ df_orders)
         ~on:Expr.[ col "customer_id" ]
         ~how:Left
    |> Lazy_frame.collect_exn
  in
  Data_frame.print df_left_join;
  [%expect
    {|
    shape: (4, 4)
    ┌─────────────┬─────────┬──────────┬────────┐
    │ customer_id ┆ name    ┆ order_id ┆ amount │
    │ ---         ┆ ---     ┆ ---      ┆ ---    │
    │ i64         ┆ str     ┆ str      ┆ i64    │
    ╞═════════════╪═════════╪══════════╪════════╡
    │ 1           ┆ Alice   ┆ a        ┆ 100    │
    │ 2           ┆ Bob     ┆ b        ┆ 200    │
    │ 2           ┆ Bob     ┆ c        ┆ 300    │
    │ 3           ┆ Charlie ┆ null     ┆ null   │
    └─────────────┴─────────┴──────────┴────────┘ |}];
  let%bind df_outer_join =
    Data_frame.lazy_ df_customers
    |> Lazy_frame.join
         ~other:(Data_frame.lazy_ df_orders)
         ~on:Expr.[ col "customer_id" ]
         ~how:Outer
    |> Lazy_frame.collect_exn
  in
  Data_frame.print df_outer_join;
  [%expect
    {|
    shape: (4, 4)
    ┌─────────────┬─────────┬──────────┬────────┐
    │ customer_id ┆ name    ┆ order_id ┆ amount │
    │ ---         ┆ ---     ┆ ---      ┆ ---    │
    │ i64         ┆ str     ┆ str      ┆ i64    │
    ╞═════════════╪═════════╪══════════╪════════╡
    │ 1           ┆ Alice   ┆ a        ┆ 100    │
    │ 2           ┆ Bob     ┆ b        ┆ 200    │
    │ 2           ┆ Bob     ┆ c        ┆ 300    │
    │ 3           ┆ Charlie ┆ null     ┆ null   │
    └─────────────┴─────────┴──────────┴────────┘ |}];
  let df_colors =
    Data_frame.create_exn Series.[ string "color" [ "red"; "green"; "blue" ] ]
  in
  Data_frame.print df_colors;
  [%expect
    {|
    shape: (3, 1)
    ┌───────┐
    │ color │
    │ ---   │
    │ str   │
    ╞═══════╡
    │ red   │
    │ green │
    │ blue  │
    └───────┘ |}];
  let df_sizes = Data_frame.create_exn Series.[ string "size" [ "S"; "M"; "L" ] ] in
  Data_frame.print df_sizes;
  [%expect
    {|
    shape: (3, 1)
    ┌──────┐
    │ size │
    │ ---  │
    │ str  │
    ╞══════╡
    │ S    │
    │ M    │
    │ L    │
    └──────┘ |}];
  let%bind df_cross_join =
    Data_frame.lazy_ df_colors
    |> Lazy_frame.join ~other:(Data_frame.lazy_ df_sizes) ~on:[] ~how:Cross
    |> Lazy_frame.collect_exn
  in
  Data_frame.print df_cross_join;
  [%expect
    {|
    shape: (9, 2)
    ┌───────┬──────┐
    │ color ┆ size │
    │ ---   ┆ ---  │
    │ str   ┆ str  │
    ╞═══════╪══════╡
    │ red   ┆ S    │
    │ red   ┆ M    │
    │ red   ┆ L    │
    │ green ┆ S    │
    │ green ┆ M    │
    │ green ┆ L    │
    │ blue  ┆ S    │
    │ blue  ┆ M    │
    │ blue  ┆ L    │
    └───────┴──────┘ |}];
  let df_cars =
    Data_frame.create_exn
      Series.[ string "id" [ "a"; "b"; "c" ]; string "make" [ "ford"; "toyota"; "bmw" ] ]
  in
  Data_frame.print df_cars;
  [%expect
    {|
    shape: (3, 2)
    ┌─────┬────────┐
    │ id  ┆ make   │
    │ --- ┆ ---    │
    │ str ┆ str    │
    ╞═════╪════════╡
    │ a   ┆ ford   │
    │ b   ┆ toyota │
    │ c   ┆ bmw    │
    └─────┴────────┘ |}];
  let df_repairs =
    Data_frame.create_exn Series.[ string "id" [ "c"; "c" ]; int "cost" [ 100; 200 ] ]
  in
  Data_frame.print df_repairs;
  [%expect
    {|
    shape: (2, 2)
    ┌─────┬──────┐
    │ id  ┆ cost │
    │ --- ┆ ---  │
    │ str ┆ i64  │
    ╞═════╪══════╡
    │ c   ┆ 100  │
    │ c   ┆ 200  │
    └─────┴──────┘ |}];
  let%bind df_inner_join =
    Data_frame.lazy_ df_cars
    |> Lazy_frame.join
         ~other:(Data_frame.lazy_ df_repairs)
         ~on:Expr.[ col "id" ]
         ~how:Inner
    |> Lazy_frame.collect_exn
  in
  Data_frame.print df_inner_join;
  [%expect
    {|
    shape: (2, 3)
    ┌─────┬──────┬──────┐
    │ id  ┆ make ┆ cost │
    │ --- ┆ ---  ┆ ---  │
    │ str ┆ str  ┆ i64  │
    ╞═════╪══════╪══════╡
    │ c   ┆ bmw  ┆ 100  │
    │ c   ┆ bmw  ┆ 200  │
    └─────┴──────┴──────┘ |}];
  let%bind df_semi_join =
    Data_frame.lazy_ df_cars
    |> Lazy_frame.join
         ~other:(Data_frame.lazy_ df_repairs)
         ~on:Expr.[ col "id" ]
         ~how:Semi
    |> Lazy_frame.collect_exn
  in
  Data_frame.print df_semi_join;
  [%expect
    {|
    shape: (1, 2)
    ┌─────┬──────┐
    │ id  ┆ make │
    │ --- ┆ ---  │
    │ str ┆ str  │
    ╞═════╪══════╡
    │ c   ┆ bmw  │
    └─────┴──────┘ |}];
  let%bind df_anti_join =
    Data_frame.lazy_ df_cars
    |> Lazy_frame.join
         ~other:(Data_frame.lazy_ df_repairs)
         ~on:Expr.[ col "id" ]
         ~how:Anti
    |> Lazy_frame.collect_exn
  in
  Data_frame.print df_anti_join;
  [%expect
    {|
    shape: (2, 2)
    ┌─────┬────────┐
    │ id  ┆ make   │
    │ --- ┆ ---    │
    │ str ┆ str    │
    ╞═════╪════════╡
    │ a   ┆ ford   │
    │ b   ┆ toyota │
    └─────┴────────┘ |}];
  let df_trades =
    Data_frame.create_exn
      Series.
        [ datetime
            "time"
            (List.map
               [ "2020-01-01 09:01:00"
               ; "2020-01-01 09:01:00"
               ; "2020-01-01 09:03:00"
               ; "2020-01-01 09:06:00"
               ]
               ~f:Common.Naive_datetime.of_string)
        ; string "stock" [ "A"; "B"; "B"; "C" ]
        ; int "trade" [ 101; 299; 301; 500 ]
        ]
  in
  Data_frame.print df_trades;
  [%expect
    {|
    shape: (4, 3)
    ┌─────────────────────┬───────┬───────┐
    │ time                ┆ stock ┆ trade │
    │ ---                 ┆ ---   ┆ ---   │
    │ datetime[ms]        ┆ str   ┆ i64   │
    ╞═════════════════════╪═══════╪═══════╡
    │ 2020-01-01 09:01:00 ┆ A     ┆ 101   │
    │ 2020-01-01 09:01:00 ┆ B     ┆ 299   │
    │ 2020-01-01 09:03:00 ┆ B     ┆ 301   │
    │ 2020-01-01 09:06:00 ┆ C     ┆ 500   │
    └─────────────────────┴───────┴───────┘ |}];
  let df_quotes =
    Data_frame.create_exn
      Series.
        [ datetime
            "time"
            (List.map
               [ "2020-01-01 09:00:00"
               ; "2020-01-01 09:02:00"
               ; "2020-01-01 09:04:00"
               ; "2020-01-01 09:06:00"
               ]
               ~f:Common.Naive_datetime.of_string)
        ; string "stock" [ "A"; "B"; "C"; "A" ]
        ; int "trade" [ 100; 300; 501; 102 ]
        ]
  in
  Data_frame.print df_quotes;
  [%expect
    {|
    shape: (4, 3)
    ┌─────────────────────┬───────┬───────┐
    │ time                ┆ stock ┆ trade │
    │ ---                 ┆ ---   ┆ ---   │
    │ datetime[ms]        ┆ str   ┆ i64   │
    ╞═════════════════════╪═══════╪═══════╡
    │ 2020-01-01 09:00:00 ┆ A     ┆ 100   │
    │ 2020-01-01 09:02:00 ┆ B     ┆ 300   │
    │ 2020-01-01 09:04:00 ┆ C     ┆ 501   │
    │ 2020-01-01 09:06:00 ┆ A     ┆ 102   │
    └─────────────────────┴───────┴───────┘ |}];
  let%bind df_asof_join =
    Data_frame.lazy_ df_trades
    |> Lazy_frame.join
         ~other:(Data_frame.lazy_ df_quotes)
         ~on:Expr.[ col "time" ]
         ~how:
           (As_of
              { strategy = `Backward
              ; tolerance = None
              ; left_by = Some [ "stock" ]
              ; right_by = Some [ "stock" ]
              })
    |> Lazy_frame.collect_exn
  in
  Data_frame.print df_asof_join;
  [%expect
    {|
    shape: (4, 4)
    ┌─────────────────────┬───────┬───────┬─────────────┐
    │ time                ┆ stock ┆ trade ┆ trade_right │
    │ ---                 ┆ ---   ┆ ---   ┆ ---         │
    │ datetime[ms]        ┆ str   ┆ i64   ┆ i64         │
    ╞═════════════════════╪═══════╪═══════╪═════════════╡
    │ 2020-01-01 09:01:00 ┆ A     ┆ 101   ┆ 100         │
    │ 2020-01-01 09:01:00 ┆ B     ┆ 299   ┆ null        │
    │ 2020-01-01 09:03:00 ┆ B     ┆ 301   ┆ 300         │
    │ 2020-01-01 09:06:00 ┆ C     ┆ 500   ┆ 501         │
    └─────────────────────┴───────┴───────┴─────────────┘ |}]
  |> return
;;

(* Examples from https://pola-rs.github.io/polars-book/user-guide/transformations/concatenation/ *)
let%expect_test "Concatenation" =
  let df_v1 = Data_frame.create_exn Series.[ int "a" [ 1 ]; int "b" [ 2 ] ] in
  let df_v2 = Data_frame.create_exn Series.[ int "a" [ 2 ]; int "b" [ 4 ] ] in
  let df_vertical_concat = Data_frame.concat_exn [ df_v1; df_v2 ] in
  Data_frame.print df_vertical_concat;
  [%expect
    {|
    shape: (2, 2)
    ┌─────┬─────┐
    │ a   ┆ b   │
    │ --- ┆ --- │
    │ i64 ┆ i64 │
    ╞═════╪═════╡
    │ 1   ┆ 2   │
    │ 2   ┆ 4   │
    └─────┴─────┘ |}];
  let df_h1 = Data_frame.create_exn Series.[ int "l1" [ 1; 2 ]; int "l2" [ 3; 4 ] ] in
  let df_h2 =
    Data_frame.create_exn
      Series.[ int "r1" [ 5; 6 ]; int "r2" [ 7; 8 ]; int "r3" [ 9; 10 ] ]
  in
  let df_horizontal_concat = Data_frame.concat_exn ~how:`Horizontal [ df_h1; df_h2 ] in
  Data_frame.print df_horizontal_concat;
  [%expect
    {|
    shape: (2, 5)
    ┌─────┬─────┬─────┬─────┬─────┐
    │ l1  ┆ l2  ┆ r1  ┆ r2  ┆ r3  │
    │ --- ┆ --- ┆ --- ┆ --- ┆ --- │
    │ i64 ┆ i64 ┆ i64 ┆ i64 ┆ i64 │
    ╞═════╪═════╪═════╪═════╪═════╡
    │ 1   ┆ 3   ┆ 5   ┆ 7   ┆ 9   │
    │ 2   ┆ 4   ┆ 6   ┆ 8   ┆ 10  │
    └─────┴─────┴─────┴─────┴─────┘ |}];
  let df_d1 = Data_frame.create_exn Series.[ int "a" [ 1 ]; int "b" [ 3 ] ] in
  let df_d2 = Data_frame.create_exn Series.[ int "a" [ 2 ]; int "d" [ 4 ] ] in
  let df_diagonal_concat = Data_frame.concat_exn ~how:`Diagonal [ df_d1; df_d2 ] in
  Data_frame.print df_diagonal_concat;
  [%expect
    {|
    shape: (2, 3)
    ┌─────┬──────┬──────┐
    │ a   ┆ b    ┆ d    │
    │ --- ┆ ---  ┆ ---  │
    │ i64 ┆ i64  ┆ i64  │
    ╞═════╪══════╪══════╡
    │ 1   ┆ 3    ┆ null │
    │ 2   ┆ null ┆ 4    │
    └─────┴──────┴──────┘ |}]
  |> return
;;

(* Examples from https://pola-rs.github.io/polars-book/user-guide/transformations/pivots/ *)
let%expect_test "Pivots" =
  let df =
    Data_frame.create_exn
      Series.
        [ string "foo" [ "A"; "A"; "B"; "B"; "C" ]
        ; int "N" [ 1; 2; 2; 4; 2 ]
        ; string "bar" [ "k"; "l"; "m"; "n"; "o" ]
        ]
  in
  Data_frame.print df;
  [%expect
    {|
    shape: (5, 3)
    ┌─────┬─────┬─────┐
    │ foo ┆ N   ┆ bar │
    │ --- ┆ --- ┆ --- │
    │ str ┆ i64 ┆ str │
    ╞═════╪═════╪═════╡
    │ A   ┆ 1   ┆ k   │
    │ A   ┆ 2   ┆ l   │
    │ B   ┆ 2   ┆ m   │
    │ B   ┆ 4   ┆ n   │
    │ C   ┆ 2   ┆ o   │
    └─────┴─────┴─────┘ |}];
  let out =
    Data_frame.pivot_exn
      df
      ~agg_expr:`First
      ~index:[ "foo" ]
      ~columns:[ "bar" ]
      ~values:[ "N" ]
  in
  Data_frame.print out;
  [%expect
    {|
    shape: (3, 6)
    ┌─────┬──────┬──────┬──────┬──────┬──────┐
    │ foo ┆ k    ┆ l    ┆ m    ┆ n    ┆ o    │
    │ --- ┆ ---  ┆ ---  ┆ ---  ┆ ---  ┆ ---  │
    │ str ┆ i64  ┆ i64  ┆ i64  ┆ i64  ┆ i64  │
    ╞═════╪══════╪══════╪══════╪══════╪══════╡
    │ A   ┆ 1    ┆ 2    ┆ null ┆ null ┆ null │
    │ B   ┆ null ┆ null ┆ 2    ┆ 4    ┆ null │
    │ C   ┆ null ┆ null ┆ null ┆ null ┆ 2    │
    └─────┴──────┴──────┴──────┴──────┴──────┘ |}];
  let%bind df = Data_frame.lazy_ df |> Lazy_frame.collect_exn in
  let out =
    Data_frame.pivot_exn
      df
      ~agg_expr:`First
      ~index:[ "foo" ]
      ~columns:[ "bar" ]
      ~values:[ "N" ]
  in
  Data_frame.print out;
  [%expect
    {|
    shape: (3, 6)
    ┌─────┬──────┬──────┬──────┬──────┬──────┐
    │ foo ┆ k    ┆ l    ┆ m    ┆ n    ┆ o    │
    │ --- ┆ ---  ┆ ---  ┆ ---  ┆ ---  ┆ ---  │
    │ str ┆ i64  ┆ i64  ┆ i64  ┆ i64  ┆ i64  │
    ╞═════╪══════╪══════╪══════╪══════╪══════╡
    │ A   ┆ 1    ┆ 2    ┆ null ┆ null ┆ null │
    │ B   ┆ null ┆ null ┆ 2    ┆ 4    ┆ null │
    │ C   ┆ null ┆ null ┆ null ┆ null ┆ 2    │
    └─────┴──────┴──────┴──────┴──────┴──────┘ |}]
  |> return
;;

(* Examples from https://pola-rs.github.io/polars-book/user-guide/transformations/melt/ *)
let%expect_test "Melt" =
  let df =
    Data_frame.create_exn
      Series.
        [ string "A" [ "a"; "b"; "a" ]
        ; int "B" [ 1; 3; 5 ]
        ; int "C" [ 10; 11; 12 ]
        ; int "D" [ 2; 4; 6 ]
        ]
  in
  Data_frame.print df;
  [%expect
    {|
    shape: (3, 4)
    ┌─────┬─────┬─────┬─────┐
    │ A   ┆ B   ┆ C   ┆ D   │
    │ --- ┆ --- ┆ --- ┆ --- │
    │ str ┆ i64 ┆ i64 ┆ i64 │
    ╞═════╪═════╪═════╪═════╡
    │ a   ┆ 1   ┆ 10  ┆ 2   │
    │ b   ┆ 3   ┆ 11  ┆ 4   │
    │ a   ┆ 5   ┆ 12  ┆ 6   │
    └─────┴─────┴─────┴─────┘ |}];
  Data_frame.melt_exn df ~id_vars:[ "A"; "B" ] ~value_vars:[ "C"; "D" ]
  |> Data_frame.print;
  [%expect
    {|
    shape: (6, 4)
    ┌─────┬─────┬──────────┬───────┐
    │ A   ┆ B   ┆ variable ┆ value │
    │ --- ┆ --- ┆ ---      ┆ ---   │
    │ str ┆ i64 ┆ str      ┆ i64   │
    ╞═════╪═════╪══════════╪═══════╡
    │ a   ┆ 1   ┆ C        ┆ 10    │
    │ b   ┆ 3   ┆ C        ┆ 11    │
    │ a   ┆ 5   ┆ C        ┆ 12    │
    │ a   ┆ 1   ┆ D        ┆ 2     │
    │ b   ┆ 3   ┆ D        ┆ 4     │
    │ a   ┆ 5   ┆ D        ┆ 6     │
    └─────┴─────┴──────────┴───────┘ |}]
  |> return
;;

let%expect_test "Time Series Parsing" =
  let df = Data_frame.read_csv_exn ~try_parse_dates:true "../data/appleStock.csv" in
  Data_frame.print df;
  [%expect
    {|
    shape: (100, 2)
    ┌────────────┬────────┐
    │ Date       ┆ Close  │
    │ ---        ┆ ---    │
    │ date       ┆ f64    │
    ╞════════════╪════════╡
    │ 1981-02-23 ┆ 24.62  │
    │ 1981-05-06 ┆ 27.38  │
    │ 1981-05-18 ┆ 28.0   │
    │ 1981-09-25 ┆ 14.25  │
    │ …          ┆ …      │
    │ 2012-12-04 ┆ 575.85 │
    │ 2013-07-05 ┆ 417.42 │
    │ 2013-11-07 ┆ 512.49 │
    │ 2014-02-25 ┆ 522.06 │
    └────────────┴────────┘ |}];
  let df =
    Data_frame.read_csv_exn ~try_parse_dates:false "../data/appleStock.csv"
    |> Data_frame.with_columns_exn
         ~exprs:Expr.[ col "Date" |> Str.strptime ~type_:Date ~format:"%Y-%m-%d" ]
  in
  Data_frame.print df;
  [%expect
    {|
    shape: (100, 2)
    ┌────────────┬────────┐
    │ Date       ┆ Close  │
    │ ---        ┆ ---    │
    │ date       ┆ f64    │
    ╞════════════╪════════╡
    │ 1981-02-23 ┆ 24.62  │
    │ 1981-05-06 ┆ 27.38  │
    │ 1981-05-18 ┆ 28.0   │
    │ 1981-09-25 ┆ 14.25  │
    │ …          ┆ …      │
    │ 2012-12-04 ┆ 575.85 │
    │ 2013-07-05 ┆ 417.42 │
    │ 2013-11-07 ┆ 512.49 │
    │ 2014-02-25 ┆ 522.06 │
    └────────────┴────────┘ |}];
  let df_with_year =
    Data_frame.with_columns_exn
      df
      ~exprs:Expr.[ col "Date" |> Dt.year |> alias ~name:"year" ]
  in
  Data_frame.print df_with_year;
  [%expect
    {|
    shape: (100, 3)
    ┌────────────┬────────┬──────┐
    │ Date       ┆ Close  ┆ year │
    │ ---        ┆ ---    ┆ ---  │
    │ date       ┆ f64    ┆ i32  │
    ╞════════════╪════════╪══════╡
    │ 1981-02-23 ┆ 24.62  ┆ 1981 │
    │ 1981-05-06 ┆ 27.38  ┆ 1981 │
    │ 1981-05-18 ┆ 28.0   ┆ 1981 │
    │ 1981-09-25 ┆ 14.25  ┆ 1981 │
    │ …          ┆ …      ┆ …    │
    │ 2012-12-04 ┆ 575.85 ┆ 2012 │
    │ 2013-07-05 ┆ 417.42 ┆ 2013 │
    │ 2013-11-07 ┆ 512.49 ┆ 2013 │
    │ 2014-02-25 ┆ 522.06 ┆ 2014 │
    └────────────┴────────┴──────┘ |}];
  let df =
    Data_frame.create_exn
      Series.
        [ string
            "date"
            [ "2021-03-27T00:00:00+0100"
            ; "2021-03-28T00:00:00+0100"
            ; "2021-03-29T00:00:00+0200"
            ; "2021-03-30T00:00:00+0200"
            ]
        ]
    |> Data_frame.with_columns_exn
         ~exprs:
           Expr.
             [ col "date"
               |> Str.strptime
                    ~type_:(Datetime (Microseconds, None))
                    ~format:"%Y-%m-%dT%H:%M:%S%z"
               |> Dt.convert_time_zone ~to_:"Europe/Brussels"
             ]
  in
  Data_frame.print df;
  [%expect
    {|
    shape: (4, 1)
    ┌───────────────────────────────┐
    │ date                          │
    │ ---                           │
    │ datetime[μs, Europe/Brussels] │
    ╞═══════════════════════════════╡
    │ 2021-03-27 00:00:00 CET       │
    │ 2021-03-28 00:00:00 CET       │
    │ 2021-03-29 00:00:00 CEST      │
    │ 2021-03-30 00:00:00 CEST      │
    └───────────────────────────────┘ |}]
  |> return
;;

(* Examples from https://pola-rs.github.io/polars-book/user-guide/transformations/time-series/filter/ *)
let%expect_test "Filtering" =
  let df = Data_frame.read_csv_exn ~try_parse_dates:true "../data/appleStock.csv" in
  Data_frame.print df;
  [%expect
    {|
    shape: (100, 2)
    ┌────────────┬────────┐
    │ Date       ┆ Close  │
    │ ---        ┆ ---    │
    │ date       ┆ f64    │
    ╞════════════╪════════╡
    │ 1981-02-23 ┆ 24.62  │
    │ 1981-05-06 ┆ 27.38  │
    │ 1981-05-18 ┆ 28.0   │
    │ 1981-09-25 ┆ 14.25  │
    │ …          ┆ …      │
    │ 2012-12-04 ┆ 575.85 │
    │ 2013-07-05 ┆ 417.42 │
    │ 2013-11-07 ┆ 512.49 │
    │ 2014-02-25 ┆ 522.06 │
    └────────────┴────────┘ |}];
  let%bind filtered_df =
    Data_frame.lazy_ df
    |> Lazy_frame.filter
         ~predicate:
           Expr.(
             col "Date" = naive_datetime (Common.Naive_datetime.of_string "1995-10-16"))
    |> Lazy_frame.collect_exn
  in
  Data_frame.print filtered_df;
  [%expect
    {|
    shape: (1, 2)
    ┌────────────┬───────┐
    │ Date       ┆ Close │
    │ ---        ┆ ---   │
    │ date       ┆ f64   │
    ╞════════════╪═══════╡
    │ 1995-10-16 ┆ 36.13 │
    └────────────┴───────┘ |}];
  let%bind filtered_range_df =
    Data_frame.lazy_ df
    |> Lazy_frame.filter
         ~predicate:
           Expr.(
             naive_datetime (Common.Naive_datetime.of_string "1995-07-01") < col "Date"
             && col "Date" < naive_datetime (Common.Naive_datetime.of_string "1995-11-01"))
    |> Lazy_frame.collect_exn
  in
  Data_frame.print filtered_range_df;
  [%expect
    {|
    shape: (2, 2)
    ┌────────────┬───────┐
    │ Date       ┆ Close │
    │ ---        ┆ ---   │
    │ date       ┆ f64   │
    ╞════════════╪═══════╡
    │ 1995-07-06 ┆ 47.0  │
    │ 1995-10-16 ┆ 36.13 │
    └────────────┴───────┘ |}]
  |> return
;;

(* Examples from https://pola-rs.github.io/polars-book/user-guide/transformations/time-series/rolling/ *)
let%expect_test "Grouping" =
  let df =
    Data_frame.read_csv_exn ~try_parse_dates:true "../data/appleStock.csv"
    |> Data_frame.sort_exn ~by_column:[ "Date" ]
  in
  Data_frame.print df;
  [%expect
    {|
    shape: (100, 2)
    ┌────────────┬────────┐
    │ Date       ┆ Close  │
    │ ---        ┆ ---    │
    │ date       ┆ f64    │
    ╞════════════╪════════╡
    │ 1981-02-23 ┆ 24.62  │
    │ 1981-05-06 ┆ 27.38  │
    │ 1981-05-18 ┆ 28.0   │
    │ 1981-09-25 ┆ 14.25  │
    │ …          ┆ …      │
    │ 2012-12-04 ┆ 575.85 │
    │ 2013-07-05 ┆ 417.42 │
    │ 2013-11-07 ┆ 512.49 │
    │ 2014-02-25 ┆ 522.06 │
    └────────────┴────────┘ |}];
  let annual_average_df =
    Data_frame.groupby_dynamic_exn
      df
      ~index_column:(Expr.col "Date")
      ~every:"1y"
      ~by:[]
      ~agg:Expr.[ col "Close" |> mean ]
  in
  let df_with_year =
    Data_frame.with_columns_exn
      annual_average_df
      ~exprs:Expr.[ col "Date" |> Dt.year |> alias ~name:"year" ]
  in
  Data_frame.print df_with_year;
  [%expect
    {|
    shape: (34, 3)
    ┌────────────┬───────────┬──────┐
    │ Date       ┆ Close     ┆ year │
    │ ---        ┆ ---       ┆ ---  │
    │ date       ┆ f64       ┆ i32  │
    ╞════════════╪═══════════╪══════╡
    │ 1981-01-01 ┆ 23.5625   ┆ 1981 │
    │ 1982-01-01 ┆ 11.0      ┆ 1982 │
    │ 1983-01-01 ┆ 30.543333 ┆ 1983 │
    │ 1984-01-01 ┆ 27.583333 ┆ 1984 │
    │ …          ┆ …         ┆ …    │
    │ 2011-01-01 ┆ 368.225   ┆ 2011 │
    │ 2012-01-01 ┆ 560.965   ┆ 2012 │
    │ 2013-01-01 ┆ 464.955   ┆ 2013 │
    │ 2014-01-01 ┆ 522.06    ┆ 2014 │
    └────────────┴───────────┴──────┘ |}];
  let df =
    Series.date_range_exn
      ~every:"1d"
      ~start:(Date.of_string "2021-01-01")
      ~stop:(Date.of_string "2021-12-31")
      "time"
    |> Series.to_data_frame
  in
  let out =
    Data_frame.groupby_dynamic_exn
      df
      ~index_column:(Expr.col "time")
      ~every:"1mo"
      ~period:"1mo"
      ~by:[]
      ~agg:
        Expr.
          [ col "time" |> cum_count |> reverse |> head ~length:3 |> alias ~name:"day/eom"
          ; col "time" - first (col "time")
            |> last
            |> Dt.days
            |> add (int 1)
            |> alias ~name:"days_in_month"
          ]
    |> Data_frame.explode_exn ~columns:[ "day/eom" ]
  in
  Data_frame.print out;
  [%expect
    {|
    shape: (36, 3)
    ┌────────────┬─────────┬───────────────┐
    │ time       ┆ day/eom ┆ days_in_month │
    │ ---        ┆ ---     ┆ ---           │
    │ date       ┆ u32     ┆ i64           │
    ╞════════════╪═════════╪═══════════════╡
    │ 2021-01-01 ┆ 30      ┆ 31            │
    │ 2021-01-01 ┆ 29      ┆ 31            │
    │ 2021-01-01 ┆ 28      ┆ 31            │
    │ 2021-02-01 ┆ 27      ┆ 28            │
    │ …          ┆ …       ┆ …             │
    │ 2021-11-01 ┆ 27      ┆ 30            │
    │ 2021-12-01 ┆ 30      ┆ 31            │
    │ 2021-12-01 ┆ 29      ┆ 31            │
    │ 2021-12-01 ┆ 28      ┆ 31            │
    └────────────┴─────────┴───────────────┘ |}];
  let df =
    Data_frame.create_exn
      Series.
        [ datetime_range_exn
            ~every:"30m"
            ~start:(Common.Naive_datetime.of_string "2021-12-16")
            ~stop:(Common.Naive_datetime.of_string "2021-12-16 3")
            "time"
        ; string "groups" [ "a"; "a"; "a"; "b"; "b"; "a"; "a" ]
        ]
  in
  Data_frame.print df;
  [%expect
    {|
    shape: (7, 2)
    ┌─────────────────────┬────────┐
    │ time                ┆ groups │
    │ ---                 ┆ ---    │
    │ datetime[ms]        ┆ str    │
    ╞═════════════════════╪════════╡
    │ 2021-12-16 00:00:00 ┆ a      │
    │ 2021-12-16 00:30:00 ┆ a      │
    │ 2021-12-16 01:00:00 ┆ a      │
    │ 2021-12-16 01:30:00 ┆ b      │
    │ 2021-12-16 02:00:00 ┆ b      │
    │ 2021-12-16 02:30:00 ┆ a      │
    │ 2021-12-16 03:00:00 ┆ a      │
    └─────────────────────┴────────┘ |}];
  let out =
    Data_frame.groupby_dynamic_exn
      df
      ~every:"1h"
      ~closed_window:`Both
      ~include_boundaries:true
      ~index_column:(Expr.col "time")
      ~by:[ Expr.col "groups" ]
      ~agg:[ Expr.count_ () ]
  in
  Data_frame.print out;
  [%expect
    {|
    shape: (7, 5)
    ┌────────┬─────────────────────┬─────────────────────┬─────────────────────┬───────┐
    │ groups ┆ _lower_boundary     ┆ _upper_boundary     ┆ time                ┆ count │
    │ ---    ┆ ---                 ┆ ---                 ┆ ---                 ┆ ---   │
    │ str    ┆ datetime[ms]        ┆ datetime[ms]        ┆ datetime[ms]        ┆ u32   │
    ╞════════╪═════════════════════╪═════════════════════╪═════════════════════╪═══════╡
    │ a      ┆ 2021-12-15 23:00:00 ┆ 2021-12-16 00:00:00 ┆ 2021-12-15 23:00:00 ┆ 1     │
    │ a      ┆ 2021-12-16 00:00:00 ┆ 2021-12-16 01:00:00 ┆ 2021-12-16 00:00:00 ┆ 3     │
    │ a      ┆ 2021-12-16 01:00:00 ┆ 2021-12-16 02:00:00 ┆ 2021-12-16 01:00:00 ┆ 1     │
    │ a      ┆ 2021-12-16 02:00:00 ┆ 2021-12-16 03:00:00 ┆ 2021-12-16 02:00:00 ┆ 2     │
    │ a      ┆ 2021-12-16 03:00:00 ┆ 2021-12-16 04:00:00 ┆ 2021-12-16 03:00:00 ┆ 1     │
    │ b      ┆ 2021-12-16 01:00:00 ┆ 2021-12-16 02:00:00 ┆ 2021-12-16 01:00:00 ┆ 2     │
    │ b      ┆ 2021-12-16 02:00:00 ┆ 2021-12-16 03:00:00 ┆ 2021-12-16 02:00:00 ┆ 1     │
    └────────┴─────────────────────┴─────────────────────┴─────────────────────┴───────┘ |}]
  |> return
;;

(* Examples from https://pola-rs.github.io/polars-book/user-guide/transformations/time-series/resampling/#upsampling-strategies *)
let%expect_test "Resampling" =
  let df =
    Data_frame.create_exn
      Series.
        [ datetime_range_exn
            ~every:"30m"
            ~start:(Common.Naive_datetime.of_string "2021-12-16")
            ~stop:(Common.Naive_datetime.of_string "2021-12-16 3")
            "time"
        ; string "groups" [ "a"; "a"; "a"; "b"; "b"; "a"; "a" ]
        ; float "values" [ 1.0; 2.0; 3.0; 4.0; 5.0; 6.0; 7.0 ]
        ]
  in
  Data_frame.print df;
  [%expect
    {|
    shape: (7, 3)
    ┌─────────────────────┬────────┬────────┐
    │ time                ┆ groups ┆ values │
    │ ---                 ┆ ---    ┆ ---    │
    │ datetime[ms]        ┆ str    ┆ f64    │
    ╞═════════════════════╪════════╪════════╡
    │ 2021-12-16 00:00:00 ┆ a      ┆ 1.0    │
    │ 2021-12-16 00:30:00 ┆ a      ┆ 2.0    │
    │ 2021-12-16 01:00:00 ┆ a      ┆ 3.0    │
    │ 2021-12-16 01:30:00 ┆ b      ┆ 4.0    │
    │ 2021-12-16 02:00:00 ┆ b      ┆ 5.0    │
    │ 2021-12-16 02:30:00 ┆ a      ┆ 6.0    │
    │ 2021-12-16 03:00:00 ┆ a      ┆ 7.0    │
    └─────────────────────┴────────┴────────┘ |}];
  Data_frame.upsample_exn df ~by:[] ~time_column:"time" ~every:"15m" ~offset:"0m"
  |> Data_frame.fill_null_exn ~strategy:(Forward None)
  |> Data_frame.print;
  [%expect
    {|
    shape: (13, 3)
    ┌─────────────────────┬────────┬────────┐
    │ time                ┆ groups ┆ values │
    │ ---                 ┆ ---    ┆ ---    │
    │ datetime[ms]        ┆ str    ┆ f64    │
    ╞═════════════════════╪════════╪════════╡
    │ 2021-12-16 00:00:00 ┆ a      ┆ 1.0    │
    │ 2021-12-16 00:15:00 ┆ a      ┆ 1.0    │
    │ 2021-12-16 00:30:00 ┆ a      ┆ 2.0    │
    │ 2021-12-16 00:45:00 ┆ a      ┆ 2.0    │
    │ …                   ┆ …      ┆ …      │
    │ 2021-12-16 02:15:00 ┆ b      ┆ 5.0    │
    │ 2021-12-16 02:30:00 ┆ a      ┆ 6.0    │
    │ 2021-12-16 02:45:00 ┆ a      ┆ 6.0    │
    │ 2021-12-16 03:00:00 ┆ a      ┆ 7.0    │
    └─────────────────────┴────────┴────────┘ |}];
  Data_frame.upsample_exn df ~by:[] ~time_column:"time" ~every:"15m" ~offset:"0m"
  |> Data_frame.interpolate_exn ~method_:`Linear
  |> Data_frame.fill_null_exn ~strategy:(Forward None)
  |> Data_frame.print;
  [%expect
    {|
    shape: (13, 3)
    ┌─────────────────────┬────────┬────────┐
    │ time                ┆ groups ┆ values │
    │ ---                 ┆ ---    ┆ ---    │
    │ datetime[ms]        ┆ str    ┆ f64    │
    ╞═════════════════════╪════════╪════════╡
    │ 2021-12-16 00:00:00 ┆ a      ┆ 1.0    │
    │ 2021-12-16 00:15:00 ┆ a      ┆ 1.5    │
    │ 2021-12-16 00:30:00 ┆ a      ┆ 2.0    │
    │ 2021-12-16 00:45:00 ┆ a      ┆ 2.5    │
    │ …                   ┆ …      ┆ …      │
    │ 2021-12-16 02:15:00 ┆ b      ┆ 5.5    │
    │ 2021-12-16 02:30:00 ┆ a      ┆ 6.0    │
    │ 2021-12-16 02:45:00 ┆ a      ┆ 6.5    │
    │ 2021-12-16 03:00:00 ┆ a      ┆ 7.0    │
    └─────────────────────┴────────┴────────┘ |}]
  |> return
;;

(* Examples from https://pola-rs.github.io/polars-book/user-guide/transformations/time-series/timezones/ *)
let%expect_test "Time zones" =
  (* TODO: The Polars Python library does roughly the below automatically when
     applying Expr-expecting operations to Series. I'm not sure doing that
     automatically is possible (or the best idea) for OCaml.

     Perhaps this function belongs in the Series module, but this causes a fair
     bit of pain associated with circular dependencies. *)
  let map_expr series ~f =
    let df =
      Series.to_data_frame series
      |> Data_frame.select_exn ~exprs:[ f (Expr.col (Series.name series)) ]
    in
    Data_frame.column_exn df ~name:(Data_frame.get_column_names df |> List.hd_exn)
  in
  let tz_naive =
    Series.string "tz_naive" [ "2021-03-27 03:00"; "2021-03-28 03:00" ]
    |> map_expr
         ~f:
           (Expr.Str.strptime
              ~type_:(Datetime (Milliseconds, None))
              ~format:"%Y-%m-%d %H:%M")
  in
  let tz_aware =
    map_expr tz_naive ~f:(Expr.Dt.replace_time_zone ~to_:(Some "UTC"))
    |> Series.rename ~name:"tz_aware"
  in
  let time_zones_df = Data_frame.create_exn [ tz_naive; tz_aware ] in
  Data_frame.print time_zones_df;
  [%expect
    {|
    shape: (2, 2)
    ┌─────────────────────┬─────────────────────────┐
    │ tz_naive            ┆ tz_aware                │
    │ ---                 ┆ ---                     │
    │ datetime[ms]        ┆ datetime[ms, UTC]       │
    ╞═════════════════════╪═════════════════════════╡
    │ 2021-03-27 03:00:00 ┆ 2021-03-27 03:00:00 UTC │
    │ 2021-03-28 03:00:00 ┆ 2021-03-28 03:00:00 UTC │
    └─────────────────────┴─────────────────────────┘ |}];
  let time_zones_operations =
    Data_frame.select_exn
      time_zones_df
      ~exprs:
        Expr.
          [ col "tz_aware"
            |> Dt.replace_time_zone ~to_:(Some "Europe/Brussels")
            |> alias ~name:"replace time zone"
          ; col "tz_aware"
            |> Dt.convert_time_zone ~to_:"Asia/Kathmandu"
            |> alias ~name:"convert time zone"
          ; col "tz_aware"
            |> Dt.replace_time_zone ~to_:None
            |> alias ~name:"unset time zone"
          ]
  in
  Data_frame.print time_zones_operations;
  [%expect
    {|
    shape: (2, 3)
    ┌───────────────────────────────┬──────────────────────────────┬─────────────────────┐
    │ replace time zone             ┆ convert time zone            ┆ unset time zone     │
    │ ---                           ┆ ---                          ┆ ---                 │
    │ datetime[ms, Europe/Brussels] ┆ datetime[ms, Asia/Kathmandu] ┆ datetime[ms]        │
    ╞═══════════════════════════════╪══════════════════════════════╪═════════════════════╡
    │ 2021-03-27 03:00:00 CET       ┆ 2021-03-27 08:45:00 +0545    ┆ 2021-03-27 03:00:00 │
    │ 2021-03-28 03:00:00 CEST      ┆ 2021-03-28 08:45:00 +0545    ┆ 2021-03-28 03:00:00 │
    └───────────────────────────────┴──────────────────────────────┴─────────────────────┘ |}]
  |> return
;;

let%expect_test "profile lazy_frame operations" =
  let df = Data_frame.create_exn Series.[ int "a" [ 3; 1; 5; 4; 2 ] ] in
  let sorted_ldf = Data_frame.lazy_ df |> Lazy_frame.sort ~by_column:"a" in
  (* Profile is non-determinstic so we don't print it *)
  let%bind { Lazy_frame.profile = _; collected } = Lazy_frame.profile_exn sorted_ldf in
  Data_frame.print collected;
  [%expect
    {|
    shape: (5, 1)
    ┌─────┐
    │ a   │
    │ --- │
    │ i64 │
    ╞═════╡
    │ 1   │
    │ 2   │
    │ 3   │
    │ 4   │
    │ 5   │
    └─────┘ |}]
  |> return
;;
