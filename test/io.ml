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
    [%expect.unreachable];
    Lazy_frame.scan_parquet_exn path |> Lazy_frame.collect_exn |> Data_frame.print;
    [%expect.unreachable])
[@@expect.uncaught_exn
  {|
  (* CR expect_test_collector: This test expectation appears to contain a backtrace.
     This is strongly discouraged as backtraces are fragile.
     Please change this test to not include a backtrace. *)

  "External format error: File out of specification: A parquet file must containt a header and footer with at least 12 bytes"
  Raised at Base__Error.raise in file "src/error.ml" (inlined), line 9, characters 14-30
  Called from Base__Or_error.ok_exn in file "src/or_error.ml", line 107, characters 17-32
  Called from Polars_tests__Io.(fun) in file "test/io.ml", line 76, characters 4-36
  Called from Base__Exn.protectx in file "src/exn.ml", line 79, characters 8-11
  Re-raised at Base__Exn.raise_with_original_backtrace in file "src/exn.ml" (inlined), line 59, characters 2-50
  Called from Base__Exn.protectx in file "src/exn.ml", line 86, characters 13-49
  Called from Expect_test_collector.Make.Instance_io.exec in file "collector/expect_test_collector.ml", line 234, characters 12-19

  Trailing output
  ---------------
  Wrote file of size 623 bytes |}]
;;
