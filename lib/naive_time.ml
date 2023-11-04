open! Core

type t

external of_ofday : Time_ns.Ofday.t -> t option = "rust_time_ns_ofday_to_naive_time"

let of_ofday_exn ofday =
  of_ofday ofday |> Option.value_exn ~here:[%here] ~message:"invalid Time_ns.Ofday.t"
;;

external to_nanoseconds : t -> int = "rust_naive_time_to_nanoseconds"

let to_ofday t = Time_ns.Ofday.create ~ns:(to_nanoseconds t) ()

let%expect_test "roundtrip" =
  Quickcheck.test Time_ns.Ofday.quickcheck_generator ~f:(fun ofday ->
    of_ofday ofday
    |> Option.iter ~f:(fun t ->
      to_ofday t |> [%test_result: Time_ns.Ofday.t] ~expect:ofday))
;;

external to_string : t -> string = "rust_naive_time_to_string"

include Pretty_printer.Register (struct
    type nonrec t = t

    let module_name = "Polars.Naive_time"
    let to_string = to_string
  end)
