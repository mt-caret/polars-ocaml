open! Core
open Polars

let%expect_test "check serializations" =
  (* TODO: use quickcheck to randomly generate and verify. *)
  let some_fields =
    [ Data_type.Boolean
    ; UInt8
    ; UInt16
    ; UInt32
    ; UInt64
    ; Int8
    ; Int16
    ; Int32
    ; Int64
    ; Float32
    ; Float64
    ; Utf8
    ; Binary
    ; Date
    ]
    @ List.map Data_type.Time_unit.all ~f:(fun time_unit ->
      Data_type.Datetime (time_unit, None))
    @ List.map Data_type.Time_unit.all ~f:(fun time_unit -> Data_type.Duration time_unit)
    @ [ Time; List Boolean; Null; Unknown ]
    |> List.map ~f:(fun data_type ->
      let name = [%sexp_of: Data_type.t] data_type |> Sexp.to_string in
      name, data_type)
  in
  Schema.create some_fields |> [%sexp_of: Schema.t] |> print_s;
  [%expect
    {|
    ((Boolean Boolean) (UInt8 UInt8) (UInt16 UInt16) (UInt32 UInt32)
     (UInt64 UInt64) (Int8 Int8) (Int16 Int16) (Int32 Int32) (Int64 Int64)
     (Float32 Float32) (Float64 Float64) (Utf8 Utf8) (Binary Binary) (Date Date)
     ("(Datetime Nanoseconds())" (Datetime Nanoseconds ()))
     ("(Datetime Microseconds())" (Datetime Microseconds ()))
     ("(Datetime Milliseconds())" (Datetime Milliseconds ()))
     ("(Duration Nanoseconds)" (Duration Nanoseconds))
     ("(Duration Microseconds)" (Duration Microseconds))
     ("(Duration Milliseconds)" (Duration Milliseconds)) (Time Time)
     ("(List Boolean)" (List Boolean)) (Null Null) (Unknown Unknown)) |}]
;;
