open! Core
open! Polars

let%expect_test "Series.estimated_size" =
  let series = Series.[
    int "integer0" [];
    int "integer1" [ 1; 2; 3; 4];
    floato "float" [ None ; Some 2. ]] in
  List.iter series ~f:(fun s -> print_endline [%string "%{Series.estimated_size s#Int}"]);
  [%expect {|
    0
    32
    17
    |}]
;;

let%expect_test "Data_frame.estimated_size" =
  let df =
    Data_frame.create_exn
      Series.[ int "integer" [ 1; 2; 3; 4; 5 ]; float "float" [ 4.; 5.; 6.; 7.; 8. ] ] in
  print_endline [%string "%{Data_frame.estimated_size df#Int}"];
  [%expect {| 80 |}]
