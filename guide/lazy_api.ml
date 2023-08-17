open! Core
open! Polars

(* Examples from https://pola-rs.github.io/polars-book/user-guide/lazy/schemas/ *)
let%expect_test "Schema" =
  Data_frame.create_exn Series.[ string "foo" [ "a"; "b"; "c" ]; int "bar" [ 0; 1; 2 ] ]
  |> Data_frame.lazy_
  |> Lazy_frame.schema_exn
  |> [%sexp_of: Schema.t]
  |> print_s;
  [%expect {| ((foo Utf8) (bar Int64)) |}];
  let lazy_eager_query =
    Data_frame.create_exn
      Series.
        [ string "id" [ "a"; "b"; "c" ]
        ; string "month" [ "jan"; "feb"; "mar" ]
        ; int "values" [ 0; 1; 2 ]
        ]
    |> Data_frame.lazy_
    |> Lazy_frame.with_columns
         ~exprs:Expr.[ int 2 * col "values" |> alias ~name:"double_values" ]
    |> Lazy_frame.collect_exn
    |> Data_frame.pivot_exn
         ~values:[ "double_values" ]
         ~index:[ "id" ]
         ~columns:[ "month" ]
         ~agg_expr:`First
    |> Data_frame.lazy_
    |> Lazy_frame.filter ~predicate:Expr.(col "mar" |> is_null)
    |> Lazy_frame.collect_exn
  in
  Data_frame.print lazy_eager_query;
  [%expect
    {|
    shape: (2, 4)
    ┌─────┬──────┬──────┬──────┐
    │ id  ┆ jan  ┆ feb  ┆ mar  │
    │ --- ┆ ---  ┆ ---  ┆ ---  │
    │ str ┆ i64  ┆ i64  ┆ i64  │
    ╞═════╪══════╪══════╪══════╡
    │ a   ┆ 0    ┆ null ┆ null │
    │ b   ┆ null ┆ 2    ┆ null │
    └─────┴──────┴──────┴──────┘ |}]
;;

(* Examples from https://pola-rs.github.io/polars-book/user-guide/lazy/query_plan/ *)
(* TODO: it seems like to_dot seems to be unstable (!!!), so commenting out test. *)
(* {[
     let%expect_test "Query Plan" =
       for _ = 0 to 1000 do
         let q1 =
           Lazy_frame.scan_csv_exn "./data/reddit.csv"
           |> Lazy_frame.with_columns ~exprs:Expr.[ col "name" |> Str.to_uppercase ]
           |> Lazy_frame.filter ~predicate:Expr.(col "comment_karma" > int 0)
         in
         Lazy_frame.to_dot_exn q1 ~optimized:false |> print_endline;
         [%expect
           {|
         graph  polars_query {
         "FILTER BY (col(\"comment_karma\")) > ... [(0, 0)]" -- "WITH COLUMNS [\"name\"] [(0, 1)]"
         "WITH COLUMNS [\"name\"] [(0, 1)]" -- "Csv SCAN ./data/reddit.csv;
         π */6;
         σ - [(0, 2)]"

         "FILTER BY (col(\"comment_karma\")) > ... [(0, 0)]"[label="FILTER BY (col(\"comment_karma\")) > ..."]
         "Csv SCAN ./data/reddit.csv;
         π */6;
         σ - [(0, 2)]"[label="Csv SCAN ./data/reddit.csv;
         π */6;
         σ -"]
         "WITH COLUMNS [\"name\"] [(0, 1)]"[label="WITH COLUMNS [\"name\"]"]

         } |}];
         Lazy_frame.explain_exn q1 ~optimized:false |> print_endline;
         [%expect
           {|
         FILTER [(col("comment_karma")) > (0)] FROM WITH_COLUMNS:
          [col("name").str.uppercase()]

             Csv SCAN ./data/reddit.csv
             PROJECT */6 COLUMNS |}];
         Lazy_frame.to_dot_exn q1 |> print_endline;
         [%expect
           {|
         graph  polars_query {
         "WITH COLUMNS [\"name\"] [(0, 0)]" -- "FILTER BY (col(\"comment_karma\")) > ... [(0, 1)]"
         "FILTER BY (col(\"comment_karma\")) > ... [(0, 1)]" -- "Csv SCAN ./data/reddit.csv;
         π */6;
         σ - [(0, 2)]"

         "Csv SCAN ./data/reddit.csv;
         π */6;
         σ - [(0, 2)]"[label="Csv SCAN ./data/reddit.csv;
         π */6;
         σ -"]
         "FILTER BY (col(\"comment_karma\")) > ... [(0, 1)]"[label="FILTER BY (col(\"comment_karma\")) > ..."]
         "WITH COLUMNS [\"name\"] [(0, 0)]"[label="WITH COLUMNS [\"name\"]"]

         } |}];
         Lazy_frame.explain_exn q1 |> print_endline;
         [%expect
           {|
         WITH_COLUMNS:
         [col("name").str.uppercase()]
          FILTER [(col("comment_karma")) > (0)] FROM

            Csv SCAN ./data/reddit.csv
            PROJECT */6 COLUMNS |}]
       done
     ;;
   ]} *)

(* Examples from https://pola-rs.github.io/polars-book/user-guide/lazy/execution/ *)
let%expect_test "Query execution" =
  let q1 =
    Lazy_frame.scan_csv_exn "./data/reddit.csv"
    |> Lazy_frame.with_columns ~exprs:Expr.[ col "name" |> Str.to_uppercase ]
    |> Lazy_frame.filter ~predicate:Expr.(col "comment_karma" > int 0)
  in
  Lazy_frame.collect_exn q1 |> Data_frame.print;
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
  let q9 =
    Lazy_frame.scan_csv_exn "./data/reddit.csv"
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
;;
