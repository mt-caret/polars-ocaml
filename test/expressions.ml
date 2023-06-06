open! Core
open! Polars

(* Examples from https://pola-rs.github.io/polars-book/user-guide/expressions/operators/ *)
let%expect_test "Basic Operators" =
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
      Series.
        [ int_option "nrs" [ Some 1; Some 2; Some 3; None; Some 5 ]
        ; string "names" [ "foo"; "ham"; "spam"; "egg"; "spam" ]
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

(* Examples from https://pola-rs.github.io/polars-book/user-guide/expressions/casting/ *)
let%expect_test "Casting" =
  let df =
    Data_frame.create_exn
      Series.
        [ int "integers" [ 1; 2; 3; 4; 5 ]
        ; int "big_integers" [ 1; 10000002; 3; 10000004; 10000005 ]
        ; float "floats" [ 4.; 5.; 6.; 7.; 8. ]
        ; float "floats_with_decimal" [ 4.532; 5.5; 6.5; 7.5; 8.5 ]
        ]
  in
  Data_frame.print df;
  [%expect
    {|
    shape: (5, 4)
    ┌──────────┬──────────────┬────────┬─────────────────────┐
    │ integers ┆ big_integers ┆ floats ┆ floats_with_decimal │
    │ ---      ┆ ---          ┆ ---    ┆ ---                 │
    │ i64      ┆ i64          ┆ f64    ┆ f64                 │
    ╞══════════╪══════════════╪════════╪═════════════════════╡
    │ 1        ┆ 1            ┆ 4.0    ┆ 4.532               │
    │ 2        ┆ 10000002     ┆ 5.0    ┆ 5.5                 │
    │ 3        ┆ 3            ┆ 6.0    ┆ 6.5                 │
    │ 4        ┆ 10000004     ┆ 7.0    ┆ 7.5                 │
    │ 5        ┆ 10000005     ┆ 8.0    ┆ 8.5                 │
    └──────────┴──────────────┴────────┴─────────────────────┘ |}];
  let out =
    Data_frame.select_exn
      df
      ~exprs:
        Expr.
          [ col "integers" |> cast ~to_:Float32 |> alias ~name:"integers_as_floats"
          ; col "floats" |> cast ~to_:Int32 |> alias ~name:"floats_as_integers"
          ; col "floats_with_decimal"
            |> cast ~to_:Int32
            |> alias ~name:"floats_decimals_as_integers"
          ]
  in
  Data_frame.print out;
  [%expect
    {|
    shape: (5, 3)
    ┌────────────────────┬────────────────────┬─────────────────────────────┐
    │ integers_as_floats ┆ floats_as_integers ┆ floats_decimals_as_integers │
    │ ---                ┆ ---                ┆ ---                         │
    │ f32                ┆ i32                ┆ i32                         │
    ╞════════════════════╪════════════════════╪═════════════════════════════╡
    │ 1.0                ┆ 4                  ┆ 4                           │
    │ 2.0                ┆ 5                  ┆ 5                           │
    │ 3.0                ┆ 6                  ┆ 6                           │
    │ 4.0                ┆ 7                  ┆ 7                           │
    │ 5.0                ┆ 8                  ┆ 8                           │
    └────────────────────┴────────────────────┴─────────────────────────────┘ |}];
  let out =
    Data_frame.select_exn
      df
      ~exprs:
        Expr.
          [ col "integers" |> cast ~to_:Int16 |> alias ~name:"integers_smallfootprint"
          ; col "floats" |> cast ~to_:Float32 |> alias ~name:"floats_smallfootprint"
          ]
  in
  Data_frame.print out;
  [%expect
    {|
    shape: (5, 2)
    ┌─────────────────────────┬───────────────────────┐
    │ integers_smallfootprint ┆ floats_smallfootprint │
    │ ---                     ┆ ---                   │
    │ i16                     ┆ f32                   │
    ╞═════════════════════════╪═══════════════════════╡
    │ 1                       ┆ 4.0                   │
    │ 2                       ┆ 5.0                   │
    │ 3                       ┆ 6.0                   │
    │ 4                       ┆ 7.0                   │
    │ 5                       ┆ 8.0                   │
    └─────────────────────────┴───────────────────────┘ |}];
  Data_frame.select df ~exprs:Expr.[ col "big_integers" |> cast ~to_:Int8 ]
  |> Result.iter_error ~f:print_endline;
  [%expect
    {| strict conversion from `i64` to `i8` failed for value(s) [10000002, 10000004, 10000005]; if you were trying to cast Utf8 to temporal dtypes, consider using `strptime` |}];
  let out =
    Data_frame.select_exn
      df
      ~exprs:Expr.[ col "big_integers" |> cast ~strict:false ~to_:Int8 ]
  in
  Data_frame.print out;
  [%expect
    {|
    shape: (5, 1)
    ┌──────────────┐
    │ big_integers │
    │ ---          │
    │ i8           │
    ╞══════════════╡
    │ 1            │
    │ null         │
    │ 3            │
    │ null         │
    │ null         │
    └──────────────┘ |}];
  let df =
    Data_frame.create_exn
      Series.
        [ int "integers" [ 1; 2; 3; 4; 5 ]
        ; float "float" [ 4.; 5.03; 6.; 7.; 8. ]
        ; string "floats_as_strings" [ "4.0"; "5.03"; "6.0"; "7.0"; "8.0" ]
        ]
  in
  let out =
    Data_frame.select_exn
      df
      ~exprs:
        Expr.
          [ col "integers" |> cast ~to_:Utf8
          ; col "float" |> cast ~to_:Utf8
          ; col "floats_as_strings" |> cast ~to_:Float64
          ]
  in
  Data_frame.print out;
  [%expect
    {|
    shape: (5, 3)
    ┌──────────┬───────┬───────────────────┐
    │ integers ┆ float ┆ floats_as_strings │
    │ ---      ┆ ---   ┆ ---               │
    │ str      ┆ str   ┆ f64               │
    ╞══════════╪═══════╪═══════════════════╡
    │ 1        ┆ 4.0   ┆ 4.0               │
    │ 2        ┆ 5.03  ┆ 5.03              │
    │ 3        ┆ 6.0   ┆ 6.0               │
    │ 4        ┆ 7.0   ┆ 7.0               │
    │ 5        ┆ 8.0   ┆ 8.0               │
    └──────────┴───────┴───────────────────┘ |}];
  let df =
    Data_frame.create_exn
      Series.[ string "strings_not_float" [ "4.0"; "not_a_number"; "6.0"; "7.0"; "8.0" ] ]
  in
  Data_frame.select df ~exprs:Expr.[ col "strings_not_float" |> cast ~to_:Float64 ]
  |> Result.iter_error ~f:print_endline;
  [%expect
    {| strict conversion from `str` to `f64` failed for value(s) ["not_a_number"]; if you were trying to cast Utf8 to temporal dtypes, consider using `strptime` |}];
  let df =
    Data_frame.create_exn
      Series.
        [ int "integers" [ -1; 0; 2; 3; 4 ]
        ; float "floats" [ 0.; 1.; 2.; 3.; 4. ]
        ; bool "bools" [ true; false; true; false; true ]
        ]
  in
  let out =
    Data_frame.select_exn
      df
      ~exprs:
        Expr.[ col "integers" |> cast ~to_:Boolean; col "floats" |> cast ~to_:Boolean ]
  in
  Data_frame.print out;
  [%expect
    {|
    shape: (5, 2)
    ┌──────────┬────────┐
    │ integers ┆ floats │
    │ ---      ┆ ---    │
    │ bool     ┆ bool   │
    ╞══════════╪════════╡
    │ true     ┆ false  │
    │ false    ┆ true   │
    │ true     ┆ true   │
    │ true     ┆ true   │
    │ true     ┆ true   │
    └──────────┴────────┘ |}];
  let df =
    Data_frame.create_exn
      Series.
        [ date_range_exn
            "date"
            ~start:(Date.of_string "2022-01-01")
            ~stop:(Date.of_string "2022-01-05")
        ; datetime_range_exn
            "datetime"
            ~start:(Date.of_string "2022-01-01")
            ~stop:(Date.of_string "2022-01-05")
        ]
  in
  Data_frame.print df;
  [%expect
    {|
    shape: (5, 2)
    ┌────────────┬─────────────────────┐
    │ date       ┆ datetime            │
    │ ---        ┆ ---                 │
    │ date       ┆ datetime[ms]        │
    ╞════════════╪═════════════════════╡
    │ 2022-01-01 ┆ 2022-01-01 00:00:00 │
    │ 2022-01-02 ┆ 2022-01-02 00:00:00 │
    │ 2022-01-03 ┆ 2022-01-03 00:00:00 │
    │ 2022-01-04 ┆ 2022-01-04 00:00:00 │
    │ 2022-01-05 ┆ 2022-01-05 00:00:00 │
    └────────────┴─────────────────────┘ |}];
  Data_frame.select_exn
    ~exprs:Expr.[ col "date" |> cast ~to_:Int64; col "datetime" |> cast ~to_:Int64 ]
    df
  |> Data_frame.print;
  [%expect
    {|
    shape: (5, 2)
    ┌───────┬───────────────┐
    │ date  ┆ datetime      │
    │ ---   ┆ ---           │
    │ i64   ┆ i64           │
    ╞═══════╪═══════════════╡
    │ 18993 ┆ 1640995200000 │
    │ 18994 ┆ 1641081600000 │
    │ 18995 ┆ 1641168000000 │
    │ 18996 ┆ 1641254400000 │
    │ 18997 ┆ 1641340800000 │
    └───────┴───────────────┘ |}];
  let df =
    Data_frame.create_exn
      Series.
        [ date_range_exn
            "date"
            ~start:(Date.of_string "2022-01-01")
            ~stop:(Date.of_string "2022-01-05")
        ; string
            "string"
            [ "2022-01-01"; "2022-01-02"; "2022-01-03"; "2022-01-04"; "2022-01-05" ]
        ]
  in
  Data_frame.select_exn
    df
    ~exprs:
      Expr.
        [ col "date" |> Dt.strftime ~format:"%Y-%m-%d"
        ; col "string"
          |> Str.strptime ~type_:(Datetime (Microseconds, None)) ~format:"%Y-%m-%d"
        ]
  |> Data_frame.print;
  [%expect
    {|
    shape: (5, 2)
    ┌────────────┬─────────────────────┐
    │ date       ┆ string              │
    │ ---        ┆ ---                 │
    │ str        ┆ datetime[μs]        │
    ╞════════════╪═════════════════════╡
    │ 2022-01-01 ┆ 2022-01-01 00:00:00 │
    │ 2022-01-02 ┆ 2022-01-02 00:00:00 │
    │ 2022-01-03 ┆ 2022-01-03 00:00:00 │
    │ 2022-01-04 ┆ 2022-01-04 00:00:00 │
    │ 2022-01-05 ┆ 2022-01-05 00:00:00 │
    └────────────┴─────────────────────┘ |}]
;;
