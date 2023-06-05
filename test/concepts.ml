open! Core
open! Polars

(* Examples from https://pola-rs.github.io/polars-book/user-guide/concepts/data-structures/ *)
let%expect_test "Basic Operators" =
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
;;
