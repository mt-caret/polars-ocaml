open! Core

type t

external of_span : Time_ns.Span.t -> t = "rust_time_ns_span_to_duration"
external to_nanoseconds : t -> int = "rust_duration_to_nanoseconds"

let to_span t = to_nanoseconds t |> Time_ns.Span.of_int_ns

let%expect_test "roundtrip" =
  Quickcheck.test Time_ns.Span.quickcheck_generator ~f:(fun span ->
    of_span span |> to_span |> [%test_result: Time_ns.Span.t] ~expect:span)
;;

external to_string : t -> string = "rust_duration_to_string"

include Pretty_printer.Register (struct
    type nonrec t = t

    let module_name = "Polars.Duration"
    let to_string = to_string
  end)

module For_testing = struct
  external round_to_time_unit
    :  t
    -> time_unit:Time_unit.t
    -> t
    = "rust_duration_round_to_time_unit"
end
