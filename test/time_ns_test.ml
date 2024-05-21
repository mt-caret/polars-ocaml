open! Core
open! Polars

let%expect_test "Time_ns.t roundtrips between Common.Naive_datetime.t" =
  Base_quickcheck.Test.run_exn
    (module Time_ns_unix)
    ~f:(fun time_ns ->
      let time_ns' = Naive_datetime.of_time_ns_exn time_ns |> Naive_datetime.to_time_ns in
      [%test_result: Time_ns_unix.t] ~expect:time_ns time_ns')
;;

let to_zone = function
  | `Nyc -> Time_ns_unix.Zone.of_string "nyc"
  | `Ldn -> Time_ns_unix.Zone.of_string "ldn"
  | `Hkg -> Time_ns_unix.Zone.of_string "hkg"
;;

module Time_and_zone = struct
  type t =
    { time : Time_ns_unix.t
    ; ofday : Time_ns.Ofday.t
    ; zone : [ `Nyc | `Ldn | `Hkg ]
    }
  [@@deriving sexp]

  module Format = struct
    type t =
      { time : Time_ns_unix.t
      ; zone : [ `Nyc | `Ldn | `Hkg ]
      }
    [@@deriving sexp, quickcheck]
  end

  let of_format { Format.time; zone } =
    (* [Time_ns.t] technically supports time before epoch via negative numbers or values
       faaar in the future, but this results in dumb edge cases like [((time (1823-11-12
        00:06:21.572612096Z)) (zone Nyc)))] causing overflow exceptions in
       [Time_ns.to_ofday]. *)
    Option.try_with (fun () -> Time_ns.to_ofday time ~zone:(to_zone zone))
    |> Option.map ~f:(fun ofday -> { time; ofday; zone })
  ;;

  let quickcheck_generator =
    Base_quickcheck.Generator.filter_map Format.quickcheck_generator ~f:of_format
  ;;

  let quickcheck_shrinker =
    Base_quickcheck.Shrinker.filter_map
      Format.quickcheck_shrinker
      ~f:of_format
      ~f_inverse:(fun { time; ofday = _; zone } -> { Format.time; zone })
  ;;
end

let%expect_test "Expr.Dt.time round trip" =
  Base_quickcheck.Test.run_exn
    (module Time_and_zone)
    ~f:(fun { Time_and_zone.time; ofday; zone } ->
      if Time_ns_unix.(
           between
             time
             (* CR-someday mtakeda: Converting to a time that's pre-epoch (at least in the
                given zone) causes a panic in Polars with message "invalid time". We can
                avoid this with code filtering generated times like the following:

                {[
                  Option.try_with (fun () -> Time_ns.utc_offset time ~zone:core_zone)
                  |> Option.bind ~f:(fun utc_offset ->
                    let time_utc = Time_ns.add_saturating time utc_offset in
                    Option.some_if
                      (not (Time_ns.is_earlier time_utc ~than:Time_ns.epoch))
                      ())
                ]}

                but this feels like something that should just not panic in Rust/Polars?

                mskarupke: It also fails for a few days after the epoch. I don't know when
                exactly this starts working reliably so I bumped it to 1975. An example of a
                failing date is 1971-10-30:

                ("Base_quickcheck.Test.run: test failed"
                (input
                ((time (1971-10-30 21:17:21.269262792-04:00)) (ofday 02:17:21.269262792)
                (zone Ldn)))
                (error
                ((runtime.ml.E "got unexpected result"
                ((expected 02:17:21.269262792) (got 01:17:21.269262792) *)
             ~low:(of_string "1975-01-01 05:00 utc")
               (* round-trippability fails for dates that are far in the future. We
                  suspect this is due to some disagreement between OCaml and Rust around DST
                  handling. *)
               (* CR-someday mtakeda: this cutoff seems... extremely suspicious?
                  https://en.wikipedia.org/wiki/Year_2038_problem

                  mskarupke: Fwiw I picked this time because I suspected the year 2038
                  problem. I didn't check when exactly the cutoff happens. If you change this
                  to 2039, the test fails, but it could also be that this is broken on
                  2038-01-01, I haven't checked. *)
             ~high:(of_string "2038-01-19 00:00 utc"))
      then (
        let zone = to_zone zone |> Time_ns_unix.Zone.to_string in
        let ofday' =
          Data_frame.create_exn []
          |> Data_frame.select_exn
               ~exprs:
                 [ Expr.lit Data_type.Typed.Core.time time
                   |> Expr.Dt.replace_time_zone ~to_:(Some "UTC")
                   |> Expr.Dt.convert_time_zone ~to_:zone
                   |> Expr.Dt.time
                   |> Expr.alias ~name:"col"
                 ]
          |> Data_frame.column_exn ~name:"col"
          |> Series.to_list Data_type.Typed.Core.ofday
          |> List.hd_exn
        in
        [%test_result: Time_ns.Ofday.t] ~expect:ofday ofday'))
;;

let%expect_test "ofday works" =
  let time = Time_ns_unix.of_string in
  let time_col = "time" in
  let df =
    Data_frame.create_exn
      [ Series.datetime'
          time_col
          [ time "2024-04-08 9:30 nyc"
          ; time "2024-04-08 12:00 nyc"
          ; time "2024-04-08 16:00 nyc"
          ; time "2024-04-08 23:59 nyc"
          ; time "2040-09-07 15:55 nyc"
          ]
      ]
  in
  let time_col = Expr.col time_col in
  let zones = [ "America/New_York"; "Europe/London"; "Asia/Hong_Kong" ] in
  let as_of_day expr ~zone =
    Expr.Dt.replace_time_zone expr ~to_:(Some "UTC")
    |> Expr.Dt.convert_time_zone ~to_:zone
    |> Expr.Dt.time
  in
  let converted =
    Data_frame.select_exn
      df
      ~exprs:
        (List.map zones ~f:(fun zone -> as_of_day time_col ~zone |> Expr.alias ~name:zone))
  in
  let results =
    List.map zones ~f:(fun zone ->
      ( zone
      , Data_frame.column_exn converted ~name:zone
        |> Series.to_list Data_type.Typed.Core.ofday ))
  in
  print_s [%message (results : (string * Time_ns.Ofday.t list) list)];
  [%expect
    {|
    (results
     ((America/New_York
       (09:30:00.000000000 12:00:00.000000000 16:00:00.000000000
        23:59:00.000000000 16:55:00.000000000))
      (Europe/London
       (14:30:00.000000000 17:00:00.000000000 21:00:00.000000000
        04:59:00.000000000 21:55:00.000000000))
      (Asia/Hong_Kong
       (21:30:00.000000000 00:00:00.000000000 04:00:00.000000000
        11:59:00.000000000 04:55:00.000000000))))
    |}]
;;

let%expect_test "span works" =
  let time = Time_ns_unix.of_string in
  let time_col = "time" in
  let df =
    Data_frame.create_exn
      [ Series.datetime'
          time_col
          [ time "2024-04-08 9:30 nyc"
          ; time "2024-04-08 12:00 nyc"
          ; time "2024-04-08 16:00 nyc"
          ]
      ]
  in
  let plus_1h =
    Data_frame.select_exn
      df
      ~exprs:[ Expr.(col time_col + lit Data_type.Typed.Core.span Time_ns.Span.hour) ]
  in
  let times =
    Data_frame.column_exn plus_1h ~name:time_col
    |> Series.to_list Data_type.Typed.Core.time
  in
  print_s [%message (times : Time_ns_unix.t list)];
  [%expect
    {|
    (times
     ((2024-04-08 10:30:00.000000000-04:00) (2024-04-08 13:00:00.000000000-04:00)
      (2024-04-08 17:00:00.000000000-04:00)))
    |}];
  let now = time "2024-04-08 16:30 nyc" in
  let now_col = "now" in
  let recent =
    Data_frame.with_columns_exn
      df
      ~exprs:[ Expr.lit Data_type.Typed.Core.time now |> Expr.alias ~name:now_col ]
    |> Data_frame.select_exn
         ~exprs:
           [ Expr.(
               col time_col + lit Data_type.Typed.Core.span Time_ns.Span.hour
               > col now_col)
           ]
    |> Data_frame.column_exn ~name:time_col
    |> Series.to_list Boolean
  in
  print_s [%message (recent : bool list)];
  [%expect {| (recent (false false true)) |}]
;;
