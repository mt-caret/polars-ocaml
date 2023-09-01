open! Core
open! Polars

let%expect_test "Series.date_option" =
  [ Some "2020-01-01"; Some "2020-01-02"; None; Some "2020-01-03" ]
  |> List.map ~f:(Option.map ~f:Date.of_string)
  |> Series.date_option "date_option"
  |> Series.print;
  [%expect
    {|
    shape: (4,)
    Series: 'date_option' [date]
    [
    	2020-01-01
    	2020-01-02
    	null
    	2020-01-03
    ] |}]
;;

let%expect_test "Series.datetime_option" =
  [ Some "2020-01-01"; Some "2020-01-02"; None; Some "2020-01-03" ]
  |> List.map ~f:(Option.map ~f:Common.Naive_datetime.of_string)
  |> Series.datetime_option "datetime_option"
  |> Series.print;
  [%expect
    {|
    shape: (4,)
    Series: 'datetime_option' [datetime[ms]]
    [
    	2020-01-01 00:00:00
    	2020-01-02 00:00:00
    	null
    	2020-01-03 00:00:00
    ] |}]
;;
