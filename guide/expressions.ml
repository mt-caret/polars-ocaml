open! Core
open! Polars

(* Examples from https://pola-rs.github.io/polars-book/user-guide/expressions/operators/ *)
let%expect_test "Basic Operators" =
  let r = Random.State.make [||] in
  let df =
    Data_frame.create_exn
      Series.
        [ into "nrs" [ Some 1; Some 2; Some 3; None; Some 5 ]
        ; stringo "names" [ Some "foo"; Some "ham"; Some "spam"; Some "egg"; None ]
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
        [ into "nrs" [ Some 1; Some 2; Some 3; None; Some 5 ]
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
          ; col "names" |> approx_n_unique |> alias ~name:"approx_unique"
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
        ; datetime_range_exn'
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

(* Examples from https://pola-rs.github.io/polars-book/user-guide/expressions/strings/ *)
let%expect_test "Casting" =
  let df =
    Data_frame.create_exn
      Series.
        [ stringo "animal" [ Some "Crab"; Some "cat and dog"; Some "rab$bit"; None ] ]
  in
  Data_frame.select_exn
    df
    ~exprs:
      Expr.
        [ col "animal" |> Str.lengths |> alias ~name:"byte_count"
        ; col "animal" |> Str.n_chars |> alias ~name:"letter_count"
        ]
  |> Data_frame.print;
  [%expect
    {|
    shape: (4, 2)
    ┌────────────┬──────────────┐
    │ byte_count ┆ letter_count │
    │ ---        ┆ ---          │
    │ u32        ┆ u32          │
    ╞════════════╪══════════════╡
    │ 4          ┆ 4            │
    │ 11         ┆ 11           │
    │ 7          ┆ 7            │
    │ null       ┆ null         │
    └────────────┴──────────────┘ |}];
  Data_frame.select_exn
    df
    ~exprs:
      Expr.
        [ col "animal"
        ; col "animal" |> Str.contains ~pat:"cat|bit" |> alias ~name:"regex"
        ; col "animal" |> Str.contains ~pat:"rab$" ~literal:true |> alias ~name:"literal"
        ; col "animal" |> Str.starts_with ~prefix:"rab" |> alias ~name:"starts_with"
        ; col "animal" |> Str.ends_with ~suffix:"dog" |> alias ~name:"ends_with"
        ]
  |> Data_frame.print;
  [%expect
    {|
    shape: (4, 5)
    ┌─────────────┬───────┬─────────┬─────────────┬───────────┐
    │ animal      ┆ regex ┆ literal ┆ starts_with ┆ ends_with │
    │ ---         ┆ ---   ┆ ---     ┆ ---         ┆ ---       │
    │ str         ┆ bool  ┆ bool    ┆ bool        ┆ bool      │
    ╞═════════════╪═══════╪═════════╪═════════════╪═══════════╡
    │ Crab        ┆ false ┆ false   ┆ false       ┆ false     │
    │ cat and dog ┆ true  ┆ false   ┆ false       ┆ true      │
    │ rab$bit     ┆ true  ┆ true    ┆ true        ┆ false     │
    │ null        ┆ null  ┆ null    ┆ null        ┆ null      │
    └─────────────┴───────┴─────────┴─────────────┴───────────┘ |}];
  let df =
    Data_frame.create_exn
      Series.
        [ string
            "a"
            [ "http://vote.com/ballon_dor?candidate=messi&ref=polars"
            ; "http://vote.com/ballon_dor?candidat=jorginho&ref=polars"
            ; "http://vote.com/ballon_dor?candidate=ronaldo&ref=polars"
            ]
        ]
  in
  Data_frame.select_exn
    df
    ~exprs:Expr.[ col "a" |> Str.extract ~pat:{|candidate=(\w+)|} ~group:1 ]
  |> Data_frame.print;
  [%expect
    {|
    shape: (3, 1)
    ┌─────────┐
    │ a       │
    │ ---     │
    │ str     │
    ╞═════════╡
    │ messi   │
    │ null    │
    │ ronaldo │
    └─────────┘ |}];
  let df =
    Data_frame.create_exn Series.[ string "foo" [ "123 bla 45 asd"; "xyz 678 910t" ] ]
  in
  Data_frame.select_exn
    df
    ~exprs:
      Expr.[ col "foo" |> Str.extract_all ~pat:{|(\d+)|} |> alias ~name:"extracted_nrs" ]
  |> Data_frame.print;
  [%expect
    {|
    shape: (2, 1)
    ┌────────────────┐
    │ extracted_nrs  │
    │ ---            │
    │ list[str]      │
    ╞════════════════╡
    │ ["123", "45"]  │
    │ ["678", "910"] │
    └────────────────┘ |}];
  let df =
    Data_frame.create_exn
      Series.[ int "id" [ 1; 2 ]; string "text" [ "123abc"; "abc456" ] ]
  in
  Data_frame.with_columns_exn
    df
    ~exprs:
      Expr.
        [ col "text" |> Str.replace ~pat:{|abc\b|} ~with_:"ABC"
        ; col "text"
          |> Str.replace_all ~pat:"a" ~with_:"-" ~literal:true
          |> alias ~name:"text_replace_all"
        ]
  |> Data_frame.print;
  [%expect
    {|
    shape: (2, 3)
    ┌─────┬────────┬──────────────────┐
    │ id  ┆ text   ┆ text_replace_all │
    │ --- ┆ ---    ┆ ---              │
    │ i64 ┆ str    ┆ str              │
    ╞═════╪════════╪══════════════════╡
    │ 1   ┆ 123ABC ┆ 123-bc           │
    │ 2   ┆ abc456 ┆ -bc456           │
    └─────┴────────┴──────────────────┘ |}]
;;

(* Examples from https://pola-rs.github.io/polars-book/user-guide/expressions/aggregation/ *)
let%expect_test "Aggregation" =
  let schema =
    Schema.create
      [ "first_name", Categorical None
      ; "gender", Categorical None
      ; "type", Categorical None
      ; "state", Categorical None
      ; "party", Categorical None
      ; "birthday", Date
      ]
  in
  let dataset =
    Data_frame.read_csv_exn
      ~schema
      ~try_parse_dates:true
      "./data/legislators-historical.csv"
  in
  Data_frame.print dataset;
  [%expect
    {|
    shape: (12_136, 36)
    ┌────────────┬────────────┬────────────┬────────┬───┬───────────┬───────────┬──────────┬───────────┐
    │ last_name  ┆ first_name ┆ middle_nam ┆ suffix ┆ … ┆ ballotped ┆ washingto ┆ icpsr_id ┆ wikipedia │
    │ ---        ┆ ---        ┆ e          ┆ ---    ┆   ┆ ia_id     ┆ n_post_id ┆ ---      ┆ _id       │
    │ str        ┆ cat        ┆ ---        ┆ str    ┆   ┆ ---       ┆ ---       ┆ i64      ┆ ---       │
    │            ┆            ┆ str        ┆        ┆   ┆ str       ┆ str       ┆          ┆ str       │
    ╞════════════╪════════════╪════════════╪════════╪═══╪═══════════╪═══════════╪══════════╪═══════════╡
    │ Bassett    ┆ Richard    ┆ null       ┆ null   ┆ … ┆ null      ┆ null      ┆ 507      ┆ Richard   │
    │            ┆            ┆            ┆        ┆   ┆           ┆           ┆          ┆ Bassett   │
    │            ┆            ┆            ┆        ┆   ┆           ┆           ┆          ┆ (Delaware │
    │            ┆            ┆            ┆        ┆   ┆           ┆           ┆          ┆ politi…   │
    │ Bland      ┆ Theodorick ┆ null       ┆ null   ┆ … ┆ null      ┆ null      ┆ 786      ┆ Theodoric │
    │            ┆            ┆            ┆        ┆   ┆           ┆           ┆          ┆ k Bland   │
    │            ┆            ┆            ┆        ┆   ┆           ┆           ┆          ┆ (congress │
    │            ┆            ┆            ┆        ┆   ┆           ┆           ┆          ┆ man)      │
    │ Burke      ┆ Aedanus    ┆ null       ┆ null   ┆ … ┆ null      ┆ null      ┆ 1260     ┆ Aedanus   │
    │            ┆            ┆            ┆        ┆   ┆           ┆           ┆          ┆ Burke     │
    │ Carroll    ┆ Daniel     ┆ null       ┆ null   ┆ … ┆ null      ┆ null      ┆ 1538     ┆ Daniel    │
    │            ┆            ┆            ┆        ┆   ┆           ┆           ┆          ┆ Carroll   │
    │ …          ┆ …          ┆ …          ┆ …      ┆ … ┆ …         ┆ …         ┆ …        ┆ …         │
    │ Flores     ┆ Mayra      ┆ null       ┆ null   ┆ … ┆ Mayra     ┆ null      ┆ null     ┆ Mayra     │
    │            ┆            ┆            ┆        ┆   ┆ Flores    ┆           ┆          ┆ Flores    │
    │ Sempolinsk ┆ Joseph     ┆ null       ┆ null   ┆ … ┆ Joe Sempo ┆ null      ┆ null     ┆ Joe Sempo │
    │ i          ┆            ┆            ┆        ┆   ┆ linski    ┆           ┆          ┆ linski    │
    │ Inhofe     ┆ James      ┆ M.         ┆ null   ┆ … ┆ Jim       ┆ null      ┆ 15424    ┆ Jim       │
    │            ┆            ┆            ┆        ┆   ┆ Inhofe    ┆           ┆          ┆ Inhofe    │
    │ Sasse      ┆ Benjamin   ┆ Eric       ┆ null   ┆ … ┆ Ben Sasse ┆ null      ┆ 41503    ┆ Ben Sasse │
    └────────────┴────────────┴────────────┴────────┴───┴───────────┴───────────┴──────────┴───────────┘ |}];
  let df =
    Data_frame.lazy_ dataset
    |> Lazy_frame.groupby
         ~by:Expr.[ col "first_name" ]
         ~agg:
           Expr.
             [ col "first_name" |> count |> alias ~name:"count"
             ; col "gender"
             ; col "last_name" |> first
             ]
    |> Lazy_frame.sort ~descending:true ~nulls_last:true ~by_column:"count"
    |> Lazy_frame.limit ~n:5
    |> Lazy_frame.collect_exn
  in
  Data_frame.print df;
  [%expect
    {|
    shape: (5, 4)
    ┌────────────┬───────┬───────────────────┬───────────┐
    │ first_name ┆ count ┆ gender            ┆ last_name │
    │ ---        ┆ ---   ┆ ---               ┆ ---       │
    │ cat        ┆ u32   ┆ list[cat]         ┆ str       │
    ╞════════════╪═══════╪═══════════════════╪═══════════╡
    │ John       ┆ 1256  ┆ ["M", "M", … "M"] ┆ Walker    │
    │ William    ┆ 1022  ┆ ["M", "M", … "M"] ┆ Few       │
    │ James      ┆ 714   ┆ ["M", "M", … "M"] ┆ Armstrong │
    │ Thomas     ┆ 454   ┆ ["M", "M", … "M"] ┆ Tucker    │
    │ Charles    ┆ 439   ┆ ["M", "M", … "M"] ┆ Carroll   │
    └────────────┴───────┴───────────────────┴───────────┘ |}];
  let df =
    Data_frame.lazy_ dataset
    |> Lazy_frame.groupby
         ~by:Expr.[ col "state" ]
         ~agg:
           Expr.
             [ col "party" = string "Anti-Administration" |> sum |> alias ~name:"anti"
             ; col "party" = string "Pro-Administration" |> sum |> alias ~name:"pro"
             ]
    |> Lazy_frame.sort ~by_column:"pro" ~descending:true ~nulls_last:false
    |> Lazy_frame.limit ~n:5
    |> Lazy_frame.collect_exn
  in
  Data_frame.print df;
  [%expect
    {|
    shape: (5, 3)
    ┌───────┬──────┬─────┐
    │ state ┆ anti ┆ pro │
    │ ---   ┆ ---  ┆ --- │
    │ cat   ┆ u32  ┆ u32 │
    ╞═══════╪══════╪═════╡
    │ NJ    ┆ 0    ┆ 3   │
    │ CT    ┆ 0    ┆ 3   │
    │ NC    ┆ 1    ┆ 2   │
    │ VA    ┆ 3    ┆ 1   │
    │ SC    ┆ 0    ┆ 1   │
    └───────┴──────┴─────┘ |}];
  let df =
    Data_frame.lazy_ dataset
    |> Lazy_frame.groupby
         ~by:Expr.[ col "state"; col "party" ]
         ~agg:Expr.[ col "party" |> count |> alias ~name:"count" ]
    |> Lazy_frame.filter
         ~predicate:
           Expr.(
             col "party" = string "Anti-Administration"
             || col "party" = string "Pro-Administration")
    |> Lazy_frame.sort ~by_column:"count" ~descending:true ~nulls_last:true
    |> Lazy_frame.limit ~n:5
    |> Lazy_frame.collect_exn
  in
  Data_frame.print df;
  [%expect
    {|
    shape: (5, 3)
    ┌───────┬─────────────────────┬───────┐
    │ state ┆ party               ┆ count │
    │ ---   ┆ ---                 ┆ ---   │
    │ cat   ┆ cat                 ┆ u32   │
    ╞═══════╪═════════════════════╪═══════╡
    │ NJ    ┆ Pro-Administration  ┆ 3     │
    │ VA    ┆ Anti-Administration ┆ 3     │
    │ CT    ┆ Pro-Administration  ┆ 3     │
    │ NC    ┆ Pro-Administration  ┆ 2     │
    │ DE    ┆ Anti-Administration ┆ 1     │
    └───────┴─────────────────────┴───────┘ |}];
  let compute_age = Expr.(int 2022 - (col "birthday" |> Dt.year)) in
  let avg_birthday gender =
    Expr.(
      filter compute_age ~predicate:(col "gender" = string gender)
      |> mean
      |> alias ~name:[%string {|avg %{gender} birthday|}])
  in
  let df =
    Data_frame.lazy_ dataset
    |> Lazy_frame.groupby
         ~by:Expr.[ col "state" ]
         ~agg:
           Expr.
             [ avg_birthday "M"
             ; avg_birthday "F"
             ; col "gender" = string "M" |> sum |> alias ~name:"# male"
             ; col "gender" = string "F" |> sum |> alias ~name:"# female"
             ]
    |> Lazy_frame.limit ~n:5
    |> Lazy_frame.collect_exn
  in
  Data_frame.print df;
  [%expect
    {|
    shape: (5, 5)
    ┌───────┬────────────────┬────────────────┬────────┬──────────┐
    │ state ┆ avg M birthday ┆ avg F birthday ┆ # male ┆ # female │
    │ ---   ┆ ---            ┆ ---            ┆ ---    ┆ ---      │
    │ cat   ┆ f64            ┆ f64            ┆ u32    ┆ u32      │
    ╞═══════╪════════════════╪════════════════╪════════╪══════════╡
    │ DE    ┆ 182.593407     ┆ null           ┆ 97     ┆ 0        │
    │ VA    ┆ 192.542781     ┆ 66.2           ┆ 430    ┆ 5        │
    │ SC    ┆ 184.018349     ┆ 122.8          ┆ 247    ┆ 5        │
    │ MD    ┆ 188.280899     ┆ 94.375         ┆ 298    ┆ 8        │
    │ PA    ┆ 180.724846     ┆ 92.857143      ┆ 1050   ┆ 7        │
    └───────┴────────────────┴────────────────┴────────┴──────────┘ |}];
  let get_person = Expr.(col "first_name" + string " " + col "last_name") in
  let df =
    Data_frame.lazy_ dataset
    |> Lazy_frame.sort ~by_column:"birthday" ~descending:true ~nulls_last:true
    |> Lazy_frame.groupby
         ~by:Expr.[ col "state" ]
         ~agg:
           Expr.
             [ get_person |> first |> alias ~name:"youngest"
             ; get_person |> last |> alias ~name:"oldest"
             ]
    |> Lazy_frame.limit ~n:5
    |> Lazy_frame.collect_exn
  in
  Data_frame.print df;
  [%expect
    {|
    shape: (5, 3)
    ┌───────┬──────────────────┬───────────────────────┐
    │ state ┆ youngest         ┆ oldest                │
    │ ---   ┆ ---              ┆ ---                   │
    │ cat   ┆ str              ┆ str                   │
    ╞═══════╪══════════════════╪═══════════════════════╡
    │ NC    ┆ Madison Cawthorn ┆ John Ashe             │
    │ IA    ┆ Abby Finkenauer  ┆ Bernhart Henn         │
    │ MI    ┆ Peter Meijer     ┆ Edward Bradley        │
    │ CA    ┆ Katie Hill       ┆ Edward Gilbert        │
    │ NY    ┆ Mondaire Jones   ┆ Cornelius Schoonmaker │
    └───────┴──────────────────┴───────────────────────┘ |}];
  let df =
    Data_frame.lazy_ dataset
    |> Lazy_frame.sort ~by_column:"birthday" ~descending:true ~nulls_last:true
    |> Lazy_frame.groupby
         ~by:Expr.[ col "state" ]
         ~agg:
           Expr.
             [ get_person |> first |> alias ~name:"youngest"
             ; get_person |> last |> alias ~name:"oldest"
             ; get_person |> sort |> first |> alias ~name:"alphabetical_first"
             ]
    |> Lazy_frame.limit ~n:5
    |> Lazy_frame.collect_exn
  in
  Data_frame.print df;
  [%expect
    {|
    shape: (5, 4)
    ┌───────┬──────────────────┬───────────────────────┬────────────────────┐
    │ state ┆ youngest         ┆ oldest                ┆ alphabetical_first │
    │ ---   ┆ ---              ┆ ---                   ┆ ---                │
    │ cat   ┆ str              ┆ str                   ┆ str                │
    ╞═══════╪══════════════════╪═══════════════════════╪════════════════════╡
    │ NC    ┆ Madison Cawthorn ┆ John Ashe             ┆ Abraham Rencher    │
    │ IA    ┆ Abby Finkenauer  ┆ Bernhart Henn         ┆ Abby Finkenauer    │
    │ MI    ┆ Peter Meijer     ┆ Edward Bradley        ┆ Aaron Bliss        │
    │ CA    ┆ Katie Hill       ┆ Edward Gilbert        ┆ Aaron Sargent      │
    │ NY    ┆ Mondaire Jones   ┆ Cornelius Schoonmaker ┆ A. Foster          │
    └───────┴──────────────────┴───────────────────────┴────────────────────┘ |}];
  let df =
    Data_frame.lazy_ dataset
    |> Lazy_frame.sort ~by_column:"birthday" ~descending:true ~nulls_last:true
    |> Lazy_frame.groupby
         ~by:Expr.[ col "state" ]
         ~agg:
           Expr.
             [ get_person |> first |> alias ~name:"youngest"
             ; get_person |> last |> alias ~name:"oldest"
             ; get_person |> sort |> first |> alias ~name:"alphabetical_first"
             ; col "gender"
               |> sort_by
                    ~by:
                      [ (* The guide uses "first_name" to sort by, but I'm guessing
                           there's an nondeterminism bug causing output to be unstable
                           if we have multiple sorts or something, so I suspect
                           [get_person] is what we actually want *)
                        get_person
                      ]
               |> first
               |> alias ~name:"gender"
             ]
    |> Lazy_frame.limit ~n:5
    |> Lazy_frame.collect_exn
  in
  Data_frame.print df;
  [%expect
    {|
    shape: (5, 5)
    ┌───────┬──────────────────┬───────────────────────┬────────────────────┬────────┐
    │ state ┆ youngest         ┆ oldest                ┆ alphabetical_first ┆ gender │
    │ ---   ┆ ---              ┆ ---                   ┆ ---                ┆ ---    │
    │ cat   ┆ str              ┆ str                   ┆ str                ┆ cat    │
    ╞═══════╪══════════════════╪═══════════════════════╪════════════════════╪════════╡
    │ NC    ┆ Madison Cawthorn ┆ John Ashe             ┆ Abraham Rencher    ┆ M      │
    │ IA    ┆ Abby Finkenauer  ┆ Bernhart Henn         ┆ Abby Finkenauer    ┆ F      │
    │ MI    ┆ Peter Meijer     ┆ Edward Bradley        ┆ Aaron Bliss        ┆ M      │
    │ CA    ┆ Katie Hill       ┆ Edward Gilbert        ┆ Aaron Sargent      ┆ M      │
    │ NY    ┆ Mondaire Jones   ┆ Cornelius Schoonmaker ┆ A. Foster          ┆ M      │
    └───────┴──────────────────┴───────────────────────┴────────────────────┴────────┘ |}]
;;

(* Examples from https://pola-rs.github.io/polars-book/user-guide/expressions/null/ *)
let%expect_test "Missing data" =
  let df = Data_frame.create_exn Series.[ into "value" [ Some 1; None ] ] in
  Data_frame.print df;
  [%expect
    {|
    shape: (2, 1)
    ┌───────┐
    │ value │
    │ ---   │
    │ i64   │
    ╞═══════╡
    │ 1     │
    │ null  │
    └───────┘ |}];
  Data_frame.null_count df |> Data_frame.print;
  [%expect
    {|
    shape: (1, 1)
    ┌───────┐
    │ value │
    │ ---   │
    │ u32   │
    ╞═══════╡
    │ 1     │
    └───────┘ |}];
  Data_frame.select_exn ~exprs:Expr.[ col "value" |> is_null ] df |> Data_frame.print;
  [%expect
    {|
    shape: (2, 1)
    ┌───────┐
    │ value │
    │ ---   │
    │ bool  │
    ╞═══════╡
    │ false │
    │ true  │
    └───────┘ |}];
  let df =
    Data_frame.create_exn
      Series.[ int "col1" [ 1; 2; 3 ]; into "col2" [ Some 1; None; Some 3 ] ]
  in
  Data_frame.print df;
  [%expect
    {|
    shape: (3, 2)
    ┌──────┬──────┐
    │ col1 ┆ col2 │
    │ ---  ┆ ---  │
    │ i64  ┆ i64  │
    ╞══════╪══════╡
    │ 1    ┆ 1    │
    │ 2    ┆ null │
    │ 3    ┆ 3    │
    └──────┴──────┘ |}];
  let fill_literal_df =
    Data_frame.with_columns_exn df ~exprs:Expr.[ col "col2" |> fill_null ~with_:(int 2) ]
  in
  Data_frame.print fill_literal_df;
  [%expect
    {|
    shape: (3, 2)
    ┌──────┬──────┐
    │ col1 ┆ col2 │
    │ ---  ┆ ---  │
    │ i64  ┆ i64  │
    ╞══════╪══════╡
    │ 1    ┆ 1    │
    │ 2    ┆ 2    │
    │ 3    ┆ 3    │
    └──────┴──────┘ |}];
  let fill_forward_df =
    Data_frame.with_columns_exn
      df
      ~exprs:Expr.[ col "col2" |> fill_null' ~strategy:(Forward None) ]
  in
  Data_frame.print fill_forward_df;
  [%expect
    {|
    shape: (3, 2)
    ┌──────┬──────┐
    │ col1 ┆ col2 │
    │ ---  ┆ ---  │
    │ i64  ┆ i64  │
    ╞══════╪══════╡
    │ 1    ┆ 1    │
    │ 2    ┆ 1    │
    │ 3    ┆ 3    │
    └──────┴──────┘ |}];
  let fill_median_df =
    Data_frame.with_columns_exn
      df
      ~exprs:Expr.[ col "col2" |> fill_null ~with_:(col "col2" |> median) ]
  in
  Data_frame.print fill_median_df;
  [%expect
    {|
    shape: (3, 2)
    ┌──────┬──────┐
    │ col1 ┆ col2 │
    │ ---  ┆ ---  │
    │ i64  ┆ f64  │
    ╞══════╪══════╡
    │ 1    ┆ 1.0  │
    │ 2    ┆ 2.0  │
    │ 3    ┆ 3.0  │
    └──────┴──────┘ |}];
  let fill_quantile_df =
    Data_frame.with_columns_exn
      df
      ~exprs:Expr.[ col "col2" |> fill_null ~with_:(col "col2" |> quantile ~quantile_expr:(float 0.1)) ]
  in
  Data_frame.print fill_quantile_df;
  [%expect {|
    shape: (3, 2)
    ┌──────┬──────┐
    │ col1 ┆ col2 │
    │ ---  ┆ ---  │
    │ i64  ┆ f64  │
    ╞══════╪══════╡
    │ 1    ┆ 1.0  │
    │ 2    ┆ 2.0  │
    │ 3    ┆ 3.0  │
    └──────┴──────┘
    |}];
  let fill_interpolation_df =
    Data_frame.with_columns_exn
      df
      ~exprs:Expr.[ col "col2" |> fill_null ~with_:(col "col2" |> interpolate) ]
  in
  Data_frame.print fill_interpolation_df;
  [%expect
    {|
    shape: (3, 2)
    ┌──────┬──────┐
    │ col1 ┆ col2 │
    │ ---  ┆ ---  │
    │ i64  ┆ i64  │
    ╞══════╪══════╡
    │ 1    ┆ 1    │
    │ 2    ┆ 2    │
    │ 3    ┆ 3    │
    └──────┴──────┘ |}];
  let nan_df =
    Data_frame.create_exn Series.[ float "value" [ 1.; Float.nan; Float.nan; 3. ] ]
  in
  Data_frame.print nan_df;
  [%expect
    {|
    shape: (4, 1)
    ┌───────┐
    │ value │
    │ ---   │
    │ f64   │
    ╞═══════╡
    │ 1.0   │
    │ NaN   │
    │ NaN   │
    │ 3.0   │
    └───────┘ |}];
  let mean_nan_df =
    Data_frame.with_columns_exn
      nan_df
      ~exprs:Expr.[ col "value" |> fill_nan ~with_:(null ()) |> alias ~name:"value" ]
    |> Data_frame.mean
  in
  Data_frame.print mean_nan_df;
  [%expect
    {|
    shape: (1, 1)
    ┌───────┐
    │ value │
    │ ---   │
    │ f64   │
    ╞═══════╡
    │ 2.0   │
    └───────┘ |}]
;;

(* Examples from https://pola-rs.github.io/polars-book/user-guide/expressions/window/ *)
let%expect_test "Window functions" =
  let df = Data_frame.read_csv_exn "./data/pokemon.csv" in
  Data_frame.print (Data_frame.head df);
  [%expect
    {|
    shape: (10, 13)
    ┌─────┬───────────────────────────┬────────┬────────┬───┬─────────┬───────┬────────────┬───────────┐
    │ #   ┆ Name                      ┆ Type 1 ┆ Type 2 ┆ … ┆ Sp. Def ┆ Speed ┆ Generation ┆ Legendary │
    │ --- ┆ ---                       ┆ ---    ┆ ---    ┆   ┆ ---     ┆ ---   ┆ ---        ┆ ---       │
    │ i64 ┆ str                       ┆ str    ┆ str    ┆   ┆ i64     ┆ i64   ┆ i64        ┆ bool      │
    ╞═════╪═══════════════════════════╪════════╪════════╪═══╪═════════╪═══════╪════════════╪═══════════╡
    │ 1   ┆ Bulbasaur                 ┆ Grass  ┆ Poison ┆ … ┆ 65      ┆ 45    ┆ 1          ┆ false     │
    │ 2   ┆ Ivysaur                   ┆ Grass  ┆ Poison ┆ … ┆ 80      ┆ 60    ┆ 1          ┆ false     │
    │ 3   ┆ Venusaur                  ┆ Grass  ┆ Poison ┆ … ┆ 100     ┆ 80    ┆ 1          ┆ false     │
    │ 3   ┆ VenusaurMega Venusaur     ┆ Grass  ┆ Poison ┆ … ┆ 120     ┆ 80    ┆ 1          ┆ false     │
    │ …   ┆ …                         ┆ …      ┆ …      ┆ … ┆ …       ┆ …     ┆ …          ┆ …         │
    │ 6   ┆ Charizard                 ┆ Fire   ┆ Flying ┆ … ┆ 85      ┆ 100   ┆ 1          ┆ false     │
    │ 6   ┆ CharizardMega Charizard X ┆ Fire   ┆ Dragon ┆ … ┆ 85      ┆ 100   ┆ 1          ┆ false     │
    │ 6   ┆ CharizardMega Charizard Y ┆ Fire   ┆ Flying ┆ … ┆ 115     ┆ 100   ┆ 1          ┆ false     │
    │ 7   ┆ Squirtle                  ┆ Water  ┆ null   ┆ … ┆ 64      ┆ 43    ┆ 1          ┆ false     │
    └─────┴───────────────────────────┴────────┴────────┴───┴─────────┴───────┴────────────┴───────────┘ |}];
  let out =
    Data_frame.select_exn
      df
      ~exprs:
        Expr.
          [ col "Type 1"
          ; col "Type 2"
          ; col "Attack"
            |> mean
            |> over ~partition_by:[ col "Type 1" ]
            |> alias ~name:"avg_attack_by_type"
          ; col "Defense"
            |> mean
            |> over ~partition_by:[ col "Type 1"; col "Type 2" ]
            |> alias ~name:"avg_defense_by_type_combination"
          ; col "Attack" |> mean |> alias ~name:"avg_attack"
          ]
  in
  Data_frame.print out;
  [%expect
    {|
    shape: (163, 5)
    ┌─────────┬────────┬────────────────────┬─────────────────────────────────┬────────────┐
    │ Type 1  ┆ Type 2 ┆ avg_attack_by_type ┆ avg_defense_by_type_combination ┆ avg_attack │
    │ ---     ┆ ---    ┆ ---                ┆ ---                             ┆ ---        │
    │ str     ┆ str    ┆ f64                ┆ f64                             ┆ f64        │
    ╞═════════╪════════╪════════════════════╪═════════════════════════════════╪════════════╡
    │ Grass   ┆ Poison ┆ 72.923077          ┆ 67.8                            ┆ 75.349693  │
    │ Grass   ┆ Poison ┆ 72.923077          ┆ 67.8                            ┆ 75.349693  │
    │ Grass   ┆ Poison ┆ 72.923077          ┆ 67.8                            ┆ 75.349693  │
    │ Grass   ┆ Poison ┆ 72.923077          ┆ 67.8                            ┆ 75.349693  │
    │ …       ┆ …      ┆ …                  ┆ …                               ┆ …          │
    │ Dragon  ┆ null   ┆ 94.0               ┆ 55.0                            ┆ 75.349693  │
    │ Dragon  ┆ null   ┆ 94.0               ┆ 55.0                            ┆ 75.349693  │
    │ Dragon  ┆ Flying ┆ 94.0               ┆ 95.0                            ┆ 75.349693  │
    │ Psychic ┆ null   ┆ 53.875             ┆ 51.428571                       ┆ 75.349693  │
    └─────────┴────────┴────────────────────┴─────────────────────────────────┴────────────┘ |}];
  let filtered =
    Data_frame.lazy_ df
    |> Lazy_frame.filter ~predicate:Expr.(col "Type 2" = string "Psychic")
    |> Lazy_frame.select ~exprs:Expr.[ col "Name"; col "Type 1"; col "Speed" ]
    |> Lazy_frame.collect_exn
  in
  Data_frame.print filtered;
  [%expect
    {|
    shape: (7, 3)
    ┌─────────────────────┬────────┬───────┐
    │ Name                ┆ Type 1 ┆ Speed │
    │ ---                 ┆ ---    ┆ ---   │
    │ str                 ┆ str    ┆ i64   │
    ╞═════════════════════╪════════╪═══════╡
    │ Slowpoke            ┆ Water  ┆ 15    │
    │ Slowbro             ┆ Water  ┆ 30    │
    │ SlowbroMega Slowbro ┆ Water  ┆ 30    │
    │ Exeggcute           ┆ Grass  ┆ 40    │
    │ Exeggutor           ┆ Grass  ┆ 55    │
    │ Starmie             ┆ Water  ┆ 115   │
    │ Jynx                ┆ Ice    ┆ 95    │
    └─────────────────────┴────────┴───────┘ |}];
  let out =
    filtered
    |> Data_frame.lazy_
    |> Lazy_frame.with_columns
         ~exprs:
           Expr.
             [ cols [ "Name"; "Speed" ]
               |> sort_by ~descending:true ~by:[ col "Speed" ]
               |> over ~partition_by:[ col "Type 1" ]
             ]
    |> Lazy_frame.collect_exn
  in
  Data_frame.print out;
  [%expect
    {|
    shape: (7, 3)
    ┌─────────────────────┬────────┬───────┐
    │ Name                ┆ Type 1 ┆ Speed │
    │ ---                 ┆ ---    ┆ ---   │
    │ str                 ┆ str    ┆ i64   │
    ╞═════════════════════╪════════╪═══════╡
    │ Starmie             ┆ Water  ┆ 115   │
    │ Slowbro             ┆ Water  ┆ 30    │
    │ SlowbroMega Slowbro ┆ Water  ┆ 30    │
    │ Exeggutor           ┆ Grass  ┆ 55    │
    │ Exeggcute           ┆ Grass  ┆ 40    │
    │ Slowpoke            ┆ Water  ┆ 15    │
    │ Jynx                ┆ Ice    ┆ 95    │
    └─────────────────────┴────────┴───────┘ |}];
  let out =
    Data_frame.lazy_ df
    |> Lazy_frame.sort ~by_column:"Type 1"
    |> Lazy_frame.select
         ~exprs:
           Expr.
             [ col "Type 1"
               |> head ~length:3
               |> over ~mapping_strategy:`Explode ~partition_by:[ col "Type 1" ]
             ; col "Name"
               |> sort_by ~by:[ col "Speed" ]
               |> head ~length:3
               |> over ~mapping_strategy:`Explode ~partition_by:[ col "Type 1" ]
               |> alias ~name:"fastest/group"
             ; col "Name"
               |> sort_by ~by:[ col "Attack" ]
               |> head ~length:3
               |> over ~mapping_strategy:`Explode ~partition_by:[ col "Type 1" ]
               |> alias ~name:"strongest/group"
             ; col "Name"
               |> sort
               |> head ~length:3
               |> over ~mapping_strategy:`Explode ~partition_by:[ col "Type 1" ]
               |> alias ~name:"sorted_by_alphabet"
             ]
    |> Lazy_frame.collect_exn
  in
  Data_frame.print out;
  [%expect
    {|
    shape: (43, 4)
    ┌────────┬─────────────────────┬─────────────────┬─────────────────────────┐
    │ Type 1 ┆ fastest/group       ┆ strongest/group ┆ sorted_by_alphabet      │
    │ ---    ┆ ---                 ┆ ---             ┆ ---                     │
    │ str    ┆ str                 ┆ str             ┆ str                     │
    ╞════════╪═════════════════════╪═════════════════╪═════════════════════════╡
    │ Bug    ┆ Paras               ┆ Metapod         ┆ Beedrill                │
    │ Bug    ┆ Metapod             ┆ Kakuna          ┆ BeedrillMega Beedrill   │
    │ Bug    ┆ Parasect            ┆ Caterpie        ┆ Butterfree              │
    │ Dragon ┆ Dratini             ┆ Dratini         ┆ Dragonair               │
    │ …      ┆ …                   ┆ …               ┆ …                       │
    │ Rock   ┆ Omanyte             ┆ Omastar         ┆ Geodude                 │
    │ Water  ┆ Slowpoke            ┆ Magikarp        ┆ Blastoise               │
    │ Water  ┆ Slowbro             ┆ Tentacool       ┆ BlastoiseMega Blastoise │
    │ Water  ┆ SlowbroMega Slowbro ┆ Horsea          ┆ Cloyster                │
    └────────┴─────────────────────┴─────────────────┴─────────────────────────┘ |}]
;;

(* Examples from https://pola-rs.github.io/polars-book/user-guide/expressions/lists/ *)
let%expect_test "Lists and Arrays" =
  let weather =
    Data_frame.create_exn
      Series.
        [ string
            "station"
            (List.range 1 6 |> List.map ~f:(fun i -> [%string "Station %{i#Int}"]))
        ; string
            "temperatures"
            [ "20 5 5 E1 7 13 19 9 6 20"
            ; "18 8 16 11 23 E2 8 E2 E2 E2 90 70 40"
            ; "19 24 E9 16 6 12 10 22"
            ; "E2 E0 15 7 8 10 E1 24 17 13 6"
            ; "14 8 E0 16 22 24 E1"
            ]
        ]
  in
  Data_frame.print weather;
  [%expect
    {|
    shape: (5, 2)
    ┌───────────┬───────────────────────────────────┐
    │ station   ┆ temperatures                      │
    │ ---       ┆ ---                               │
    │ str       ┆ str                               │
    ╞═══════════╪═══════════════════════════════════╡
    │ Station 1 ┆ 20 5 5 E1 7 13 19 9 6 20          │
    │ Station 2 ┆ 18 8 16 11 23 E2 8 E2 E2 E2 90 7… │
    │ Station 3 ┆ 19 24 E9 16 6 12 10 22            │
    │ Station 4 ┆ E2 E0 15 7 8 10 E1 24 17 13 6     │
    │ Station 5 ┆ 14 8 E0 16 22 24 E1               │
    └───────────┴───────────────────────────────────┘ |}];
  Data_frame.with_columns_exn
    weather
    ~exprs:Expr.[ col "temperatures" |> Str.split ~by:" " ]
  |> Data_frame.print;
  [%expect
    {|
    shape: (5, 2)
    ┌───────────┬──────────────────────┐
    │ station   ┆ temperatures         │
    │ ---       ┆ ---                  │
    │ str       ┆ list[str]            │
    ╞═══════════╪══════════════════════╡
    │ Station 1 ┆ ["20", "5", … "20"]  │
    │ Station 2 ┆ ["18", "8", … "40"]  │
    │ Station 3 ┆ ["19", "24", … "22"] │
    │ Station 4 ┆ ["E2", "E0", … "6"]  │
    │ Station 5 ┆ ["14", "8", … "E1"]  │
    └───────────┴──────────────────────┘ |}];
  Data_frame.with_columns_exn
    weather
    ~exprs:Expr.[ col "temperatures" |> Str.split ~by:" " ]
  |> Data_frame.explode_exn ~columns:[ "temperatures" ]
  |> Data_frame.print;
  [%expect
    {|
    shape: (49, 2)
    ┌───────────┬──────────────┐
    │ station   ┆ temperatures │
    │ ---       ┆ ---          │
    │ str       ┆ str          │
    ╞═══════════╪══════════════╡
    │ Station 1 ┆ 20           │
    │ Station 1 ┆ 5            │
    │ Station 1 ┆ 5            │
    │ Station 1 ┆ E1           │
    │ …         ┆ …            │
    │ Station 5 ┆ 16           │
    │ Station 5 ┆ 22           │
    │ Station 5 ┆ 24           │
    │ Station 5 ┆ E1           │
    └───────────┴──────────────┘ |}];
  Data_frame.with_columns_exn
    weather
    ~exprs:Expr.[ col "temperatures" |> Str.split ~by:" " ]
  |> Data_frame.with_columns_exn
       ~exprs:
         Expr.
           [ col "temperatures" |> List.head ~n:(int 3) |> alias ~name:"top3"
           ; col "temperatures"
             |> List.slice ~offset:(int (-3)) ~length:(int 3)
             |> alias ~name:"bottom_3"
           ; col "temperatures" |> List.lengths |> alias ~name:"obs"
           ]
  |> Data_frame.print;
  [%expect
    {|
    shape: (5, 5)
    ┌───────────┬──────────────────────┬────────────────────┬────────────────────┬─────┐
    │ station   ┆ temperatures         ┆ top3               ┆ bottom_3           ┆ obs │
    │ ---       ┆ ---                  ┆ ---                ┆ ---                ┆ --- │
    │ str       ┆ list[str]            ┆ list[str]          ┆ list[str]          ┆ u32 │
    ╞═══════════╪══════════════════════╪════════════════════╪════════════════════╪═════╡
    │ Station 1 ┆ ["20", "5", … "20"]  ┆ ["20", "5", "5"]   ┆ ["9", "6", "20"]   ┆ 10  │
    │ Station 2 ┆ ["18", "8", … "40"]  ┆ ["18", "8", "16"]  ┆ ["90", "70", "40"] ┆ 13  │
    │ Station 3 ┆ ["19", "24", … "22"] ┆ ["19", "24", "E9"] ┆ ["12", "10", "22"] ┆ 8   │
    │ Station 4 ┆ ["E2", "E0", … "6"]  ┆ ["E2", "E0", "15"] ┆ ["17", "13", "6"]  ┆ 11  │
    │ Station 5 ┆ ["14", "8", … "E1"]  ┆ ["14", "8", "E0"]  ┆ ["22", "24", "E1"] ┆ 7   │
    └───────────┴──────────────────────┴────────────────────┴────────────────────┴─────┘ |}];
  Data_frame.with_columns_exn
    weather
    ~exprs:
      Expr.
        [ col "temperatures"
          |> Str.split ~by:" "
          |> List.eval ~expr:(element () |> cast ~strict:false ~to_:Int64 |> is_null)
          |> List.sum
          |> alias ~name:"errors"
        ]
  |> Data_frame.print;
  [%expect
    {|
    shape: (5, 3)
    ┌───────────┬───────────────────────────────────┬────────┐
    │ station   ┆ temperatures                      ┆ errors │
    │ ---       ┆ ---                               ┆ ---    │
    │ str       ┆ str                               ┆ u32    │
    ╞═══════════╪═══════════════════════════════════╪════════╡
    │ Station 1 ┆ 20 5 5 E1 7 13 19 9 6 20          ┆ 1      │
    │ Station 2 ┆ 18 8 16 11 23 E2 8 E2 E2 E2 90 7… ┆ 4      │
    │ Station 3 ┆ 19 24 E9 16 6 12 10 22            ┆ 1      │
    │ Station 4 ┆ E2 E0 15 7 8 10 E1 24 17 13 6     ┆ 3      │
    │ Station 5 ┆ 14 8 E0 16 22 24 E1               ┆ 2      │
    └───────────┴───────────────────────────────────┴────────┘ |}];
  Data_frame.with_columns_exn
    weather
    ~exprs:
      Expr.
        [ col "temperatures"
          |> Str.split ~by:" "
          |> List.eval ~expr:(element () |> Str.contains ~pat:"(?i)[a-z]")
          |> List.sum
          |> alias ~name:"errors"
        ]
  |> Data_frame.print;
  [%expect
    {|
    shape: (5, 3)
    ┌───────────┬───────────────────────────────────┬────────┐
    │ station   ┆ temperatures                      ┆ errors │
    │ ---       ┆ ---                               ┆ ---    │
    │ str       ┆ str                               ┆ u32    │
    ╞═══════════╪═══════════════════════════════════╪════════╡
    │ Station 1 ┆ 20 5 5 E1 7 13 19 9 6 20          ┆ 1      │
    │ Station 2 ┆ 18 8 16 11 23 E2 8 E2 E2 E2 90 7… ┆ 4      │
    │ Station 3 ┆ 19 24 E9 16 6 12 10 22            ┆ 1      │
    │ Station 4 ┆ E2 E0 15 7 8 10 E1 24 17 13 6     ┆ 3      │
    │ Station 5 ┆ 14 8 E0 16 22 24 E1               ┆ 2      │
    └───────────┴───────────────────────────────────┴────────┘ |}];
  let weather_by_day =
    Data_frame.create_exn
      Series.
        [ string
            "station"
            (List.range 1 11 |> List.map ~f:(fun i -> [%string "Station %{i#Int}"]))
        ; int "day_1" [ 17; 11; 8; 22; 9; 21; 20; 8; 8; 17 ]
        ; int "day_2" [ 15; 11; 10; 8; 7; 14; 18; 21; 15; 13 ]
        ; int "day_3" [ 16; 15; 24; 24; 8; 23; 19; 23; 16; 10 ]
        ]
  in
  Data_frame.print weather_by_day;
  [%expect
    {|
    shape: (10, 4)
    ┌────────────┬───────┬───────┬───────┐
    │ station    ┆ day_1 ┆ day_2 ┆ day_3 │
    │ ---        ┆ ---   ┆ ---   ┆ ---   │
    │ str        ┆ i64   ┆ i64   ┆ i64   │
    ╞════════════╪═══════╪═══════╪═══════╡
    │ Station 1  ┆ 17    ┆ 15    ┆ 16    │
    │ Station 2  ┆ 11    ┆ 11    ┆ 15    │
    │ Station 3  ┆ 8     ┆ 10    ┆ 24    │
    │ Station 4  ┆ 22    ┆ 8     ┆ 24    │
    │ …          ┆ …     ┆ …     ┆ …     │
    │ Station 7  ┆ 20    ┆ 18    ┆ 19    │
    │ Station 8  ┆ 8     ┆ 21    ┆ 23    │
    │ Station 9  ┆ 8     ┆ 15    ┆ 16    │
    │ Station 10 ┆ 17    ┆ 13    ┆ 10    │
    └────────────┴───────┴───────┴───────┘ |}];
  let rank_pct =
    Expr.(
      (element ()
       |> rank ~descending:true
       |> (* Division by default doesn't convert into floats so an explicit
             cast is required. *)
       cast ~to_:Float64)
      / (col "*" |> count)
      |> round ~decimals:2)
  in
  Data_frame.with_columns_exn
    weather_by_day
    ~exprs:
      Expr.
        [ concat_list [ all () |> exclude ~names:[ "station" ] ]
          |> alias ~name:"all_temps"
        ]
  |> Data_frame.select_exn
       ~exprs:
         Expr.
           [ all () |> exclude ~names:[ "all_temps" ]
           ; col "all_temps" |> List.eval ~expr:rank_pct |> alias ~name:"temps_rank"
           ]
  |> Data_frame.print;
  [%expect
    {|
    shape: (10, 5)
    ┌────────────┬───────┬───────┬───────┬────────────────────┐
    │ station    ┆ day_1 ┆ day_2 ┆ day_3 ┆ temps_rank         │
    │ ---        ┆ ---   ┆ ---   ┆ ---   ┆ ---                │
    │ str        ┆ i64   ┆ i64   ┆ i64   ┆ list[f64]          │
    ╞════════════╪═══════╪═══════╪═══════╪════════════════════╡
    │ Station 1  ┆ 17    ┆ 15    ┆ 16    ┆ [0.33, 1.0, 0.67]  │
    │ Station 2  ┆ 11    ┆ 11    ┆ 15    ┆ [0.67, 0.67, 0.33] │
    │ Station 3  ┆ 8     ┆ 10    ┆ 24    ┆ [1.0, 0.67, 0.33]  │
    │ Station 4  ┆ 22    ┆ 8     ┆ 24    ┆ [0.67, 1.0, 0.33]  │
    │ …          ┆ …     ┆ …     ┆ …     ┆ …                  │
    │ Station 7  ┆ 20    ┆ 18    ┆ 19    ┆ [0.33, 1.0, 0.67]  │
    │ Station 8  ┆ 8     ┆ 21    ┆ 23    ┆ [1.0, 0.67, 0.33]  │
    │ Station 9  ┆ 8     ┆ 15    ┆ 16    ┆ [1.0, 0.67, 0.33]  │
    │ Station 10 ┆ 17    ┆ 13    ┆ 10    ┆ [0.33, 0.67, 1.0]  │
    └────────────┴───────┴───────┴───────┴────────────────────┘ |}]
;;
