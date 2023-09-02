open! Core
open! Polars

let%expect_test "Time_ns.t roundtrips between Common.Naive_datetime.t" =
  Base_quickcheck.Test.run_exn
    (module Time_ns_unix)
    ~f:(fun time_ns ->
      let time_ns' =
        Common.Naive_datetime.of_time_ns_exn time_ns |> Common.Naive_datetime.to_time_ns
      in
      [%test_result: Time_ns_unix.t] ~expect:time_ns time_ns')
;;
