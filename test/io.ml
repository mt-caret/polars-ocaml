open! Core
open! Polars

let%expect_test "CSV" =
  Filename_extended.with_temp_dir "polars-ocaml" "csv" ~f:(fun temp_file ->
    let path = temp_file ^/ "path.csv" in
    let df =
      Data_frame.create_exn
        Series.
          [ int "foo" [ 1; 2; 3 ]; string_option "bar" [ None; Some "bak"; Some "baz" ] ]
    in
    Data_frame.print df;
    [%expect
      {|
      shape: (3, 2)
      ┌─────┬──────┐
      │ foo ┆ bar  │
      │ --- ┆ ---  │
      │ i64 ┆ str  │
      ╞═════╪══════╡
      │ 1   ┆ null │
      │ 2   ┆ bak  │
      │ 3   ┆ baz  │
      └─────┴──────┘ |}];
    Data_frame.write_csv_exn df path;
    Data_frame.read_csv_exn path |> Data_frame.print;
    [%expect
      {|
      shape: (3, 2)
      ┌─────┬──────┐
      │ foo ┆ bar  │
      │ --- ┆ ---  │
      │ i64 ┆ str  │
      ╞═════╪══════╡
      │ 1   ┆ null │
      │ 2   ┆ bak  │
      │ 3   ┆ baz  │
      └─────┴──────┘ |}];
    Lazy_frame.scan_csv_exn path |> Lazy_frame.collect_exn |> Data_frame.print;
    [%expect
      {|
      shape: (3, 2)
      ┌─────┬──────┐
      │ foo ┆ bar  │
      │ --- ┆ ---  │
      │ i64 ┆ str  │
      ╞═════╪══════╡
      │ 1   ┆ null │
      │ 2   ┆ bak  │
      │ 3   ┆ baz  │
      └─────┴──────┘ |}])
;;

let%expect_test "Parquet" =
  Filename_extended.with_temp_dir "polars-ocaml" "parquet" ~f:(fun temp_file ->
    let path = temp_file ^/ "path.parquet" in
    let df =
      Data_frame.create_exn
        Series.
          [ int "foo" [ 1; 2; 3 ]; string_option "bar" [ None; Some "bak"; Some "baz" ] ]
    in
    Data_frame.print df;
    [%expect
      {|
      shape: (3, 2)
      ┌─────┬──────┐
      │ foo ┆ bar  │
      │ --- ┆ ---  │
      │ i64 ┆ str  │
      ╞═════╪══════╡
      │ 1   ┆ null │
      │ 2   ┆ bak  │
      │ 3   ┆ baz  │
      └─────┴──────┘ |}];
    Data_frame.write_parquet_exn df path;
    Data_frame.read_parquet_exn path |> Data_frame.print;
    [%expect
      {|
      Wrote file of size 623 bytes
      shape: (3, 2)
      ┌─────┬──────┐
      │ foo ┆ bar  │
      │ --- ┆ ---  │
      │ i64 ┆ str  │
      ╞═════╪══════╡
      │ 1   ┆ null │
      │ 2   ┆ bak  │
      │ 3   ┆ baz  │
      └─────┴──────┘ |}];
    Lazy_frame.scan_parquet_exn path |> Lazy_frame.collect_exn |> Data_frame.print;
    [%expect
      {|
      shape: (3, 2)
      ┌─────┬──────┐
      │ foo ┆ bar  │
      │ --- ┆ ---  │
      │ i64 ┆ str  │
      ╞═════╪══════╡
      │ 1   ┆ null │
      │ 2   ┆ bak  │
      │ 3   ┆ baz  │
      └─────┴──────┘ |}])
;;
