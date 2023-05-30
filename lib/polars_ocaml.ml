open! Core

type lazy_frame

external rust_twice : int -> int = "rust_twice"

external rust_lazy_frame_of_parquet
  :  string
  -> (lazy_frame, string) result
  = "rust_lazy_frame_of_parquet"

external rust_lazy_frame_to_dot
  :  lazy_frame
  -> (string, string) result
  = "rust_lazy_frame_to_dot"

let%expect_test "twice" =
  print_s [%sexp (rust_twice 2 : int)];
  [%expect {| 4 |}];
  print_s [%sexp (rust_twice Int.max_value : int)];
  [%expect {| -2 |}];
  rust_lazy_frame_of_parquet "some_parquet_file.parquet"
  |> [%sexp_of: (_, string) Result.t]
  |> print_s;
  [%expect {| (Error "No such file or directory (os error 2)") |}]
;;
