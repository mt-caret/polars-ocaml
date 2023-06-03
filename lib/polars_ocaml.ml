open! Core
module Expr = Expr
module Series = Series
module Data_frame = Data_frame
module Lazy_frame = Lazy_frame

(* TODO: what's an ergonomic way to create series and dataframes easily? *)
(* Examples from https://pola-rs.github.io/polars-book/user-guide/expressions/operators/ *)
let%expect_test "Basic Operators" =
  let r = Random.State.make [||] in
  let df =
    Data_frame.create_exn
      [ Series.int_option "nrs" [ Some 1; Some 2; Some 3; None; Some 5 ]
      ; Series.string_option
          "names"
          [ Some "foo"; Some "ham"; Some "spam"; Some "egg"; None ]
      ; Series.float "random" (List.init 5 ~f:(fun _ -> Random.State.float r 5.))
      ; Series.string "groups" [ "A"; "A"; "B"; "C"; "B" ]
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
  let df_numerical =
    Data_frame.select_exn
      df
      ~exprs:
        Expr.
          [ col "nrs" + int 5 |> alias ~name:"nrs + 5"
          ; col "nrs" - int 5 |> alias ~name:"nrs - 5"
          ; col "nrs" * col "random" |> alias ~name:"nrs * random"
          ; col "nrs" / col "random" |> alias ~name:"nrs / random"
          ]
  in
  Data_frame.print df_numerical;
  [%expect
    {|
    shape: (5, 4)
    ┌─────────┬─────────┬──────────────┬──────────────┐
    │ nrs + 5 ┆ nrs - 5 ┆ nrs * random ┆ nrs / random │
    │ ---     ┆ ---     ┆ ---          ┆ ---          │
    │ i64     ┆ i64     ┆ f64          ┆ f64          │
    ╞═════════╪═════════╪══════════════╪══════════════╡
    │ 6       ┆ -4      ┆ 1.848939     ┆ 0.540851     │
    │ 7       ┆ -3      ┆ 8.980802     ┆ 0.445394     │
    │ 8       ┆ -2      ┆ 9.442697     ┆ 0.953117     │
    │ null    ┆ null    ┆ null         ┆ null         │
    │ 10      ┆ 0       ┆ 4.159012     ┆ 6.011044     │
    └─────────┴─────────┴──────────────┴──────────────┘ |}];
  let df_logical =
    Data_frame.select_exn
      df
      ~exprs:
        Expr.
          [ col "nrs" > int 1 |> alias ~name:"nrs > 1"
          ; col "random" <= float 0.5 |> alias ~name:"random <= 0.5"
          ; col "nrs" <> int 1 |> alias ~name:"nrs != 1"
          ; col "nrs" = int 1 |> alias ~name:"nrs == 1"
          ; (col "random" <= float 0.5 && col "nrs" > int 1) |> alias ~name:"and_expr"
          ; (col "random" <= float 0.5 || col "nrs" > int 1) |> alias ~name:"or_expr"
          ]
  in
  Data_frame.print df_logical;
  [%expect
    {|
    shape: (5, 6)
    ┌─────────┬───────────────┬──────────┬──────────┬──────────┬─────────┐
    │ nrs > 1 ┆ random <= 0.5 ┆ nrs != 1 ┆ nrs == 1 ┆ and_expr ┆ or_expr │
    │ ---     ┆ ---           ┆ ---      ┆ ---      ┆ ---      ┆ ---     │
    │ bool    ┆ bool          ┆ bool     ┆ bool     ┆ bool     ┆ bool    │
    ╞═════════╪═══════════════╪══════════╪══════════╪══════════╪═════════╡
    │ false   ┆ false         ┆ false    ┆ true     ┆ false    ┆ false   │
    │ true    ┆ false         ┆ true     ┆ false    ┆ false    ┆ true    │
    │ true    ┆ false         ┆ true     ┆ false    ┆ false    ┆ true    │
    │ null    ┆ true          ┆ null     ┆ null     ┆ null     ┆ true    │
    │ true    ┆ false         ┆ true     ┆ false    ┆ false    ┆ true    │
    └─────────┴───────────────┴──────────┴──────────┴──────────┴─────────┘ |}]
;;

(* Examples from https://pola-rs.github.io/polars-book/user-guide/expressions/functions/ *)
let%expect_test "Functions" =
  let r = Random.State.make [||] in
  let df =
    Data_frame.create_exn
      [ Series.int_option "nrs" [ Some 1; Some 2; Some 3; None; Some 5 ]
      ; Series.string "names" [ "foo"; "ham"; "spam"; "egg"; "spam" ]
      ; Series.float "random" (List.init 5 ~f:(fun _ -> Random.State.float r 5.))
      ; Series.string "groups" [ "A"; "A"; "B"; "C"; "B" ]
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
    │ 5    ┆ spam  ┆ 0.831802 ┆ B      │
    └──────┴───────┴──────────┴────────┘ |}];
  let df_alias =
    Data_frame.select_exn
      df
      ~exprs:
        Expr.
          [ col "nrs" + int 5 |> alias ~name:"nrs + 5"
          ; col "nrs" - int 5 |> alias ~name:"nrs - 5"
          ]
  in
  Data_frame.print df_alias;
  [%expect
    {|
    shape: (5, 2)
    ┌─────────┬─────────┐
    │ nrs + 5 ┆ nrs - 5 │
    │ ---     ┆ ---     │
    │ i64     ┆ i64     │
    ╞═════════╪═════════╡
    │ 6       ┆ -4      │
    │ 7       ┆ -3      │
    │ 8       ┆ -2      │
    │ null    ┆ null    │
    │ 10      ┆ 0       │
    └─────────┴─────────┘ |}];
  let df_alias =
    Data_frame.select_exn
      df
      ~exprs:
        Expr.
          [ col "names" |> n_unique |> alias ~name:"unique"
          ; col "names" |> approx_unique |> alias ~name:"approx_unique"
          ]
  in
  Data_frame.print df_alias;
  [%expect
    {|
    shape: (1, 2)
    ┌────────┬───────────────┐
    │ unique ┆ approx_unique │
    │ ---    ┆ ---           │
    │ u32    ┆ u32           │
    ╞════════╪═══════════════╡
    │ 4      ┆ 4             │
    └────────┴───────────────┘ |}];
  let df_conditional =
    Data_frame.select_exn
      df
      ~exprs:
        Expr.[ col "nrs"; when_ [ col "nrs" > int 2, bool true ] ~otherwise:(bool false) ]
  in
  Data_frame.print df_conditional;
  [%expect
    {|
    shape: (5, 2)
    ┌──────┬─────────┐
    │ nrs  ┆ literal │
    │ ---  ┆ ---     │
    │ i64  ┆ bool    │
    ╞══════╪═════════╡
    │ 1    ┆ false   │
    │ 2    ┆ false   │
    │ 3    ┆ true    │
    │ null ┆ false   │
    │ 5    ┆ true    │
    └──────┴─────────┘ |}]
;;
