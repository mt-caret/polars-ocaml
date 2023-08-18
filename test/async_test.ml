open! Core
open Async
open! Polars
open Polars_async

let%expect_test "Joins (with async collections)" =
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

let%expect_test "Query execution (with async fetch and collect)" =
  let q1 =
    Lazy_frame.scan_csv_exn "../guide/data/reddit.csv"
    |> Lazy_frame.with_columns ~exprs:Expr.[ col "name" |> Str.to_uppercase ]
    |> Lazy_frame.filter ~predicate:Expr.(col "comment_karma" > int 0)
  in
  let%bind () = Lazy_frame.collect_exn q1 >>| Data_frame.print in
  [%expect
    {|
    shape: (27, 6)
    ┌───────┬───────────────────────────┬─────────────┬────────────┬───────────────┬────────────┐
    │ id    ┆ name                      ┆ created_utc ┆ updated_on ┆ comment_karma ┆ link_karma │
    │ ---   ┆ ---                       ┆ ---         ┆ ---        ┆ ---           ┆ ---        │
    │ i64   ┆ str                       ┆ i64         ┆ i64        ┆ i64           ┆ i64        │
    ╞═══════╪═══════════════════════════╪═════════════╪════════════╪═══════════════╪════════════╡
    │ 6     ┆ TAOJIANLONG_JASONBROKEN   ┆ 1397113510  ┆ 1536527864 ┆ 4             ┆ 0          │
    │ 17    ┆ SSAIG_JASONBROKEN         ┆ 1397113544  ┆ 1536527864 ┆ 1             ┆ 0          │
    │ 19    ┆ FDBVFDSSDGFDS_JASONBROKEN ┆ 1397113552  ┆ 1536527864 ┆ 3             ┆ 0          │
    │ 37    ┆ IHATEWHOWEARE_JASONBROKEN ┆ 1397113636  ┆ 1536527864 ┆ 61            ┆ 0          │
    │ …     ┆ …                         ┆ …           ┆ …          ┆ …             ┆ …          │
    │ 77763 ┆ LUNCHY                    ┆ 1137599510  ┆ 1536528275 ┆ 65            ┆ 0          │
    │ 77765 ┆ COMPOSTELLAS              ┆ 1137474000  ┆ 1536528276 ┆ 6             ┆ 0          │
    │ 77766 ┆ GENERICBOB                ┆ 1137474000  ┆ 1536528276 ┆ 291           ┆ 14         │
    │ 77768 ┆ TINHEADNED                ┆ 1139665457  ┆ 1536497404 ┆ 4434          ┆ 103        │
    └───────┴───────────────────────────┴─────────────┴────────────┴───────────────┴────────────┘ |}];
  let q5 = Lazy_frame.collect ~streaming:true q1 in
  ignore q5;
  let%bind q9 =
    Lazy_frame.scan_csv_exn "../guide/data/reddit.csv"
    |> Lazy_frame.with_columns ~exprs:Expr.[ col "name" |> Str.to_uppercase ]
    |> Lazy_frame.filter ~predicate:Expr.(col "comment_karma" > int 0)
    |> Lazy_frame.fetch_exn ~n_rows:100
  in
  Data_frame.print q9;
  [%expect
    {|
    shape: (27, 6)
    ┌───────┬───────────────────────────┬─────────────┬────────────┬───────────────┬────────────┐
    │ id    ┆ name                      ┆ created_utc ┆ updated_on ┆ comment_karma ┆ link_karma │
    │ ---   ┆ ---                       ┆ ---         ┆ ---        ┆ ---           ┆ ---        │
    │ i64   ┆ str                       ┆ i64         ┆ i64        ┆ i64           ┆ i64        │
    ╞═══════╪═══════════════════════════╪═════════════╪════════════╪═══════════════╪════════════╡
    │ 6     ┆ TAOJIANLONG_JASONBROKEN   ┆ 1397113510  ┆ 1536527864 ┆ 4             ┆ 0          │
    │ 17    ┆ SSAIG_JASONBROKEN         ┆ 1397113544  ┆ 1536527864 ┆ 1             ┆ 0          │
    │ 19    ┆ FDBVFDSSDGFDS_JASONBROKEN ┆ 1397113552  ┆ 1536527864 ┆ 3             ┆ 0          │
    │ 37    ┆ IHATEWHOWEARE_JASONBROKEN ┆ 1397113636  ┆ 1536527864 ┆ 61            ┆ 0          │
    │ …     ┆ …                         ┆ …           ┆ …          ┆ …             ┆ …          │
    │ 77763 ┆ LUNCHY                    ┆ 1137599510  ┆ 1536528275 ┆ 65            ┆ 0          │
    │ 77765 ┆ COMPOSTELLAS              ┆ 1137474000  ┆ 1536528276 ┆ 6             ┆ 0          │
    │ 77766 ┆ GENERICBOB                ┆ 1137474000  ┆ 1536528276 ┆ 291           ┆ 14         │
    │ 77768 ┆ TINHEADNED                ┆ 1139665457  ┆ 1536497404 ┆ 4434          ┆ 103        │
    └───────┴───────────────────────────┴─────────────┴────────────┴───────────────┴────────────┘ |}]
  |> return
;;

let%expect_test "async profile lazy_frame operations" =
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
