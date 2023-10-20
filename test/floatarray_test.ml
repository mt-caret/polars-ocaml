open Core
open Polars

let%expect_test "Series.float'" =
  Base_quickcheck.Test.run_exn
    (module struct
      type t = float list [@@deriving sexp, quickcheck]
    end)
    ~f:(fun values ->
      let series = Series.float' "series_name" (Stdlib.Float.Array.of_list values) in
      let values' = Series.to_list Float64 series in
      [%test_result: float list] ~expect:values values';
      let values' = Series.to_option_list Float64 series |> List.filter_opt in
      [%test_result: float list] ~expect:values values';
      List.iteri values ~f:(fun i value ->
        [%test_result: float] ~expect:value (Series.get_exn Float64 series i)))
;;
