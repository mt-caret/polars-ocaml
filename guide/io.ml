open! Core
open! Polars

(* Examples from https://pola-rs.github.io/polars-book/user-guide/io/csv/ *)
let%expect_test "CSV" =
  Filename_extended.with_temp_dir "polars-ocaml" "csv" ~f:(fun temp_dir ->
    let path = temp_dir ^/ "path.csv" in
    let df =
      Data_frame.create_exn
        Series.[ int "foo" [ 1; 2; 3 ]; stringo "bar" [ None; Some "bak"; Some "baz" ] ]
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
    In_channel.read_all path |> print_endline;
    [%expect {|
      foo,bar
      1,
      2,bak
      3,baz
      |}];
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

(* Examples from https://pola-rs.github.io/polars-book/user-guide/io/parquet/ *)
let%expect_test "Parquet" =
  Filename_extended.with_temp_dir "polars-ocaml" "parquet" ~f:(fun temp_dir ->
    let path = temp_dir ^/ "path.parquet" in
    let df =
      Data_frame.create_exn
        Series.[ int "foo" [ 1; 2; 3 ]; stringo "bar" [ None; Some "bak"; Some "baz" ] ]
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

(* Examples from https://pola-rs.github.io/polars-book/user-guide/io/json_file/ *)
let%expect_test "JSON files" =
  Filename_extended.with_temp_dir "polars-ocaml" "json" ~f:(fun temp_dir ->
    let path = temp_dir ^/ "path.json" in
    let df =
      Data_frame.create_exn
        Series.[ int "foo" [ 1; 2; 3 ]; stringo "bar" [ None; Some "bak"; Some "baz" ] ]
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
    Data_frame.write_json_exn df path;
    In_channel.read_all path |> print_endline;
    [%expect
      {|
      [{"foo":1,"bar":null},{"foo":2,"bar":"bak"},{"foo":3,"bar":"baz"}]
      |}];
    Data_frame.read_json_exn path |> Data_frame.print;
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
    (* Polars currently doesn't provide a way to lazily scan a JSON file, so
       there is no [Lazy_frame.scan_json_exn]. *)
    Data_frame.write_jsonl_exn df path;
    In_channel.read_all path |> print_endline;
    [%expect
      {|
      {"foo":1,"bar":null}
      {"foo":2,"bar":"bak"}
      {"foo":3,"bar":"baz"}
      |}];
    Data_frame.read_jsonl_exn path |> Data_frame.print;
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
    Lazy_frame.scan_jsonl_exn path |> Lazy_frame.collect_exn |> Data_frame.print;
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

(* Examples from https://pola-rs.github.io/polars-book/user-guide/io/multiple/ *)
let%expect_test "Multiple" =
  Expect_test_helpers_core.require_does_raise [%here] (fun () ->
    Filename_extended.with_temp_dir "polars-ocaml" "json" ~f:(fun temp_dir ->
      let df =
        Data_frame.create_exn
          Series.
            [ int "foo" [ 1; 2; 3 ]; stringo "bar" [ None; Some "Ham"; Some "Spam" ] ]
      in
      for i = 0 to 4 do
        Data_frame.write_csv_exn df (temp_dir ^/ [%string "my_many_files_%{i#Int}.csv"])
      done;
      try
        Data_frame.read_csv_exn (temp_dir ^/ "my_many_files_*.csv") |> Data_frame.print
      with
      | exn ->
        let exn =
          Exn.to_string exn
          (* We truncate the directory portion of the path since it is unstable. *)
          |> Re2.replace_exn
               ~f:(fun _ -> ": my_many_files")
               (Re2.create_exn ":.*my_many_files")
          |> Failure
        in
        raise exn));
  (* Globs are currently not supported in polars-ocaml, so passing glob patterns
     in read/scan functions do not work. *)
  [%expect
    {|
    (Failure
     "\"error open file: my_many_files_*.csv, No such file or directory (os error 2)\"") |}]
;;

(* Implementation of database, AWS, and BigQuery IO are blocked by a good OCaml
   Arrow interop story: https://github.com/mt-caret/polars-ocaml/issues/26 *)
