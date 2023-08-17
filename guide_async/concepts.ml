open! Core
open Async
open! Polars
open! Polars_async

(* Examples from https://pola-rs.github.io/polars-book/user-guide/concepts/data-structures/ *)
let%expect_test "Data Structures" =
  let s = Series.int "a" [ 1; 2; 3; 4; 5 ] in
  Series.print s;
  [%expect
    {|
    shape: (5,)
    Series: 'a' [i64]
    [
    	1
    	2
    	3
    	4
    	5
    ] |}];
  let df =
    Data_frame.create_exn
      Series.
        [ int "integer" [ 1; 2; 3; 4; 5 ]
        ; List.map
            [ "2022-01-01"; "2022-01-02"; "2022-01-03"; "2022-01-04"; "2022-01-05" ]
            ~f:Date.of_string
          |> date "date"
        ; float "float" [ 4.; 5.; 6.; 7.; 8. ]
        ]
  in
  Data_frame.print df;
  [%expect
    {|
    shape: (5, 3)
    ┌─────────┬────────────┬───────┐
    │ integer ┆ date       ┆ float │
    │ ---     ┆ ---        ┆ ---   │
    │ i64     ┆ date       ┆ f64   │
    ╞═════════╪════════════╪═══════╡
    │ 1       ┆ 2022-01-01 ┆ 4.0   │
    │ 2       ┆ 2022-01-02 ┆ 5.0   │
    │ 3       ┆ 2022-01-03 ┆ 6.0   │
    │ 4       ┆ 2022-01-04 ┆ 7.0   │
    │ 5       ┆ 2022-01-05 ┆ 8.0   │
    └─────────┴────────────┴───────┘ |}];
  Data_frame.head df ~length:3 |> Data_frame.print;
  [%expect
    {|
    shape: (3, 3)
    ┌─────────┬────────────┬───────┐
    │ integer ┆ date       ┆ float │
    │ ---     ┆ ---        ┆ ---   │
    │ i64     ┆ date       ┆ f64   │
    ╞═════════╪════════════╪═══════╡
    │ 1       ┆ 2022-01-01 ┆ 4.0   │
    │ 2       ┆ 2022-01-02 ┆ 5.0   │
    │ 3       ┆ 2022-01-03 ┆ 6.0   │
    └─────────┴────────────┴───────┘ |}];
  Data_frame.tail df ~length:3 |> Data_frame.print;
  [%expect
    {|
    shape: (3, 3)
    ┌─────────┬────────────┬───────┐
    │ integer ┆ date       ┆ float │
    │ ---     ┆ ---        ┆ ---   │
    │ i64     ┆ date       ┆ f64   │
    ╞═════════╪════════════╪═══════╡
    │ 3       ┆ 2022-01-03 ┆ 6.0   │
    │ 4       ┆ 2022-01-04 ┆ 7.0   │
    │ 5       ┆ 2022-01-05 ┆ 8.0   │
    └─────────┴────────────┴───────┘ |}];
  Data_frame.sample_n_exn df ~seed:0 ~with_replacement:false ~shuffle:true ~n:2
  |> Data_frame.print;
  [%expect
    {|
    shape: (2, 3)
    ┌─────────┬────────────┬───────┐
    │ integer ┆ date       ┆ float │
    │ ---     ┆ ---        ┆ ---   │
    │ i64     ┆ date       ┆ f64   │
    ╞═════════╪════════════╪═══════╡
    │ 3       ┆ 2022-01-03 ┆ 6.0   │
    │ 1       ┆ 2022-01-01 ┆ 4.0   │
    └─────────┴────────────┴───────┘ |}];
  Data_frame.describe_exn df |> Data_frame.print;
  [%expect
    {|
    shape: (9, 4)
    ┌────────────┬──────────┬────────────┬──────────┐
    │ describe   ┆ integer  ┆ date       ┆ float    │
    │ ---        ┆ ---      ┆ ---        ┆ ---      │
    │ str        ┆ f64      ┆ str        ┆ f64      │
    ╞════════════╪══════════╪════════════╪══════════╡
    │ count      ┆ 5.0      ┆ 5          ┆ 5.0      │
    │ null_count ┆ 0.0      ┆ 0          ┆ 0.0      │
    │ mean       ┆ 3.0      ┆ null       ┆ 6.0      │
    │ std        ┆ 1.581139 ┆ null       ┆ 1.581139 │
    │ min        ┆ 1.0      ┆ 2022-01-01 ┆ 4.0      │
    │ 25%        ┆ 2.0      ┆ null       ┆ 5.0      │
    │ 50%        ┆ 3.0      ┆ null       ┆ 6.0      │
    │ 75%        ┆ 4.0      ┆ null       ┆ 7.0      │
    │ max        ┆ 5.0      ┆ 2022-01-05 ┆ 8.0      │
    └────────────┴──────────┴────────────┴──────────┘ |}]
  |> return
;;

(* Examples from https://pola-rs.github.io/polars-book/user-guide/concepts/contexts/ *)
let%expect_test "Contexts" =
  let r = Random.State.make [||] in
  let df =
    Data_frame.create_exn
      Series.
        [ int_option "nrs" [ Some 1; Some 2; Some 3; None; Some 5 ]
        ; string_option "names" [ Some "foo"; Some "ham"; Some "spam"; Some "egg"; None ]
        ; float "random" (List.init 5 ~f:(fun _ -> Random.State.float r 5.))
        ; string "groups" [ "A"; "A"; "B"; "C"; "B" ]
        ]
  in
  Data_frame.print df;
  [%expect
    {|
    shape: (5, 4)
    ┌──────┬───────┬──────────┬────────┐
    │ nrs  ┆ names ┆ random   ┆ groups │
    │ ---  ┆ ---   ┆ ---      ┆ ---    │
    │ i64  ┆ str   ┆ f64      ┆ str    │
    ╞══════╪═══════╪══════════╪════════╡
    │ 1    ┆ foo   ┆ 1.848939 ┆ A      │
    │ 2    ┆ ham   ┆ 4.490401 ┆ A      │
    │ 3    ┆ spam  ┆ 3.147566 ┆ B      │
    │ null ┆ egg   ┆ 0.156988 ┆ C      │
    │ 5    ┆ null  ┆ 0.831802 ┆ B      │
    └──────┴───────┴──────────┴────────┘ |}];
  let out =
    Data_frame.select_exn
      df
      ~exprs:
        Expr.
          [ col "nrs" |> Expr.sum
          ; col "names" |> Expr.sort
          ; col "names" |> Expr.first |> Expr.alias ~name:"first name"
          ; Expr.mean (col "nrs") * int 10 |> Expr.alias ~name:"10xnrs"
          ]
  in
  Data_frame.print out;
  [%expect
    {|
    shape: (5, 4)
    ┌─────┬───────┬────────────┬────────┐
    │ nrs ┆ names ┆ first name ┆ 10xnrs │
    │ --- ┆ ---   ┆ ---        ┆ ---    │
    │ i64 ┆ str   ┆ str        ┆ f64    │
    ╞═════╪═══════╪════════════╪════════╡
    │ 11  ┆ null  ┆ foo        ┆ 27.5   │
    │ 11  ┆ egg   ┆ foo        ┆ 27.5   │
    │ 11  ┆ foo   ┆ foo        ┆ 27.5   │
    │ 11  ┆ ham   ┆ foo        ┆ 27.5   │
    │ 11  ┆ spam  ┆ foo        ┆ 27.5   │
    └─────┴───────┴────────────┴────────┘ |}];
  let out =
    Data_frame.with_columns_exn
      df
      ~exprs:
        Expr.
          [ col "nrs" |> Expr.sum |> Expr.alias ~name:"nrs_sum"
          ; col "random" |> Expr.count |> Expr.alias ~name:"count"
          ]
  in
  Data_frame.print out;
  [%expect
    {|
    shape: (5, 6)
    ┌──────┬───────┬──────────┬────────┬─────────┬───────┐
    │ nrs  ┆ names ┆ random   ┆ groups ┆ nrs_sum ┆ count │
    │ ---  ┆ ---   ┆ ---      ┆ ---    ┆ ---     ┆ ---   │
    │ i64  ┆ str   ┆ f64      ┆ str    ┆ i64     ┆ u32   │
    ╞══════╪═══════╪══════════╪════════╪═════════╪═══════╡
    │ 1    ┆ foo   ┆ 1.848939 ┆ A      ┆ 11      ┆ 5     │
    │ 2    ┆ ham   ┆ 4.490401 ┆ A      ┆ 11      ┆ 5     │
    │ 3    ┆ spam  ┆ 3.147566 ┆ B      ┆ 11      ┆ 5     │
    │ null ┆ egg   ┆ 0.156988 ┆ C      ┆ 11      ┆ 5     │
    │ 5    ┆ null  ┆ 0.831802 ┆ B      ┆ 11      ┆ 5     │
    └──────┴───────┴──────────┴────────┴─────────┴───────┘ |}];
  let%bind out =
    Data_frame.lazy_ df
    |> Lazy_frame.filter ~predicate:Expr.(col "nrs" > int 2)
    |> Lazy_frame.collect_exn
  in
  Data_frame.print out;
  [%expect
    {|
    shape: (2, 4)
    ┌─────┬───────┬──────────┬────────┐
    │ nrs ┆ names ┆ random   ┆ groups │
    │ --- ┆ ---   ┆ ---      ┆ ---    │
    │ i64 ┆ str   ┆ f64      ┆ str    │
    ╞═════╪═══════╪══════════╪════════╡
    │ 3   ┆ spam  ┆ 3.147566 ┆ B      │
    │ 5   ┆ null  ┆ 0.831802 ┆ B      │
    └─────┴───────┴──────────┴────────┘ |}];
  let out =
    Data_frame.groupby_exn
      df
      (* TODO: the default is false, which originally caused this test to be
         nondeterministic (I assumed that this was unstable in the sense of an
         unstable sort, not nondeterminism). Perhaps the default should be
         is_stable=true? *)
      ~is_stable:true
      ~by:Expr.[ col "groups" ]
      ~agg:
        Expr.
          [ col "nrs" |> Expr.sum
          ; col "random" |> Expr.count |> Expr.alias ~name:"count"
          ; col "random"
            |> Expr.filter ~predicate:(col "names" |> Expr.is_not_null)
            |> Expr.sum
            |> Expr.suffix ~suffix:"_sum"
          ; col "names" |> Expr.reverse |> Expr.alias ~name:"reversed names"
          ]
  in
  Data_frame.print out;
  [%expect
    {|
    shape: (3, 5)
    ┌────────┬─────┬───────┬────────────┬────────────────┐
    │ groups ┆ nrs ┆ count ┆ random_sum ┆ reversed names │
    │ ---    ┆ --- ┆ ---   ┆ ---        ┆ ---            │
    │ str    ┆ i64 ┆ u32   ┆ f64        ┆ list[str]      │
    ╞════════╪═════╪═══════╪════════════╪════════════════╡
    │ A      ┆ 3   ┆ 2     ┆ 6.33934    ┆ ["ham", "foo"] │
    │ B      ┆ 8   ┆ 2     ┆ 3.147566   ┆ [null, "spam"] │
    │ C      ┆ 0   ┆ 1     ┆ 0.156988   ┆ ["egg"]        │
    └────────┴─────┴───────┴────────────┴────────────────┘ |}]
  |> return
;;

(* Examples from https://pola-rs.github.io/polars-book/user-guide/concepts/expressions/ *)
let%expect_test "Contexts" =
  ignore (fun df ->
    Data_frame.column_exn df ~name:"foo" |> Series.sort |> Series.head ~length:2)
  |> return
;;

(* Examples from https://pola-rs.github.io/polars-book/user-guide/concepts/lazy-vs-eager/ *)
let%expect_test "Lazy / Eager API" =
  (* eager API not included, since underlying Rust functions have been deprecated. *)
  let%bind () =
    Lazy_frame.scan_csv_exn "./data/iris.csv"
    |> Lazy_frame.filter ~predicate:Expr.(col "sepal_length" > int 5)
    |> Lazy_frame.groupby
         ~is_stable:true
         ~by:Expr.[ col "species" ]
         ~agg:Expr.[ col "sepal_width" |> Expr.mean ]
    |> Lazy_frame.collect_exn
    >>| Data_frame.print
  in
  [%expect
    {|
    shape: (3, 2)
    ┌─────────────────┬─────────────┐
    │ species         ┆ sepal_width │
    │ ---             ┆ ---         │
    │ str             ┆ f64         │
    ╞═════════════════╪═════════════╡
    │ Iris-setosa     ┆ 3.713636    │
    │ Iris-versicolor ┆ 2.804255    │
    │ Iris-virginica  ┆ 2.983673    │
    └─────────────────┴─────────────┘ |}]
  |> return
;;

(* Examples from https://pola-rs.github.io/polars-book/user-guide/concepts/streaming/ *)
let%expect_test "Streaming" =
  let%bind () =
    Lazy_frame.scan_csv_exn "./data/iris.csv"
    |> Lazy_frame.filter ~predicate:Expr.(col "sepal_length" > int 5)
    |> Lazy_frame.groupby
         ~is_stable:true
         ~by:Expr.[ col "species" ]
         ~agg:Expr.[ col "sepal_width" |> Expr.mean ]
    |> Lazy_frame.with_streaming ~toggle:true
    |> Lazy_frame.collect_exn
    >>| Data_frame.print
  in
  [%expect
    {|
    shape: (3, 2)
    ┌─────────────────┬─────────────┐
    │ species         ┆ sepal_width │
    │ ---             ┆ ---         │
    │ str             ┆ f64         │
    ╞═════════════════╪═════════════╡
    │ Iris-setosa     ┆ 3.713636    │
    │ Iris-versicolor ┆ 2.804255    │
    │ Iris-virginica  ┆ 2.983673    │
    └─────────────────┴─────────────┘ |}]
  |> return
;;
