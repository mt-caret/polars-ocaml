open Core

type t

external of_naive_date
  :  Naive_date.t
  -> hour:int option
  -> min:int option
  -> sec:int option
  -> t option
  = "rust_naive_date_to_naive_datetime"

let of_naive_date ?hour ?min ?sec naive_date =
  of_naive_date naive_date ~hour ~min ~sec |> Option.value_exn ~here:[%here]
;;

let of_date ?hour ?min ?sec date =
  Naive_date.of_date date |> of_naive_date ?hour ?min ?sec
;;

external to_string : t -> string = "rust_naive_datetime_to_string"

let of_string str =
  let naive_date, hour, min, sec =
    match String.split str ~on:' ' with
    | [ date ] -> Naive_date.of_string date, None, None, None
    | [ date; time ] ->
      let naive_date = Naive_date.of_string date in
      let hour, min, sec =
        match String.split time ~on:':' with
        | [ hour; min; sec ] ->
          Some (Int.of_string hour), Some (Int.of_string min), Some (Int.of_string sec)
        | [ hour; min ] -> Some (Int.of_string hour), Some (Int.of_string min), None
        | [ hour ] -> Some (Int.of_string hour), None, None
        | _ -> raise_s [%message "Unexpected time format" time]
      in
      naive_date, hour, min, sec
    | _ -> raise_s [%message "Unexpected datetime format" str]
  in
  of_naive_date naive_date ?hour ?min ?sec
;;

let%expect_test "of_string" =
  let parse_and_print str = of_string str |> to_string |> print_endline in
  parse_and_print "2023-01-02";
  [%expect {| 2023-01-02 00:00:00 |}];
  parse_and_print "2023-01-02 03:04:05";
  [%expect {| 2023-01-02 03:04:05 |}];
  parse_and_print "2023-01-02 03:04";
  [%expect {| 2023-01-02 03:04:00 |}];
  parse_and_print "2023-01-02 03";
  [%expect {| 2023-01-02 03:00:00 |}]
;;

external of_time_ns : Time_ns.t -> t option = "rust_time_ns_to_naive_datetime"

let of_time_ns_exn time_ns =
  of_time_ns time_ns
  |> Option.value_or_thunk ~default:(fun () ->
    raise_s [%message "Invalid time" (time_ns : Time_ns_unix.t)])
;;

let%expect_test "of_time_ns" =
  (* We need to specify the "-08:00" portion, since without it
     [Time_ns_unix.of_string] assumes the time zone is the local time zone,
     resulting in non-determinism. *)
  let time_ns = Time_ns_unix.of_string "2023-01-02 03:04:05.678-08:00" in
  of_time_ns_exn time_ns |> to_string |> print_endline;
  [%expect {| 2023-01-02 11:04:05.678 |}]
;;

external to_timestamp_nanos : t -> int = "rust_naive_datetime_to_timestamp_nanos"

let to_time_ns t = to_timestamp_nanos t |> Time_ns.of_int_ns_since_epoch

let%expect_test "roundtrip" =
  Quickcheck.test Time_ns_unix.quickcheck_generator ~f:(fun time_ns ->
    of_time_ns_exn time_ns |> to_time_ns |> [%test_result: Time_ns_unix.t] ~expect:time_ns)
;;

module For_testing = struct
  external round_to_time_unit
    :  t
    -> time_unit:Time_unit.t
    -> t
    = "rust_naive_datetime_round_to_time_unit"
end
