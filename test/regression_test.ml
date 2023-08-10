open! Core
open! Polars

(* TODO: This used to SEGV since I mistyped the name of the extern call to point
   to a different function; https://github.com/mt-caret/polars-ocaml/issues/5
   and MDX-style doctest should be a more natural place for these sorts of
   sanity checks to make sure these functions actually work. *)
let%expect_test "Data_frame.columns_exn" =
  let df =
    Data_frame.create_exn
      Series.[ int "integer" [ 1; 2; 3; 4; 5 ]; float "float" [ 4.; 5.; 6.; 7.; 8. ] ]
  in
  Data_frame.columns_exn df ~names:[ "integer"; "float" ] |> List.iter ~f:Series.print;
  [%expect
    {|
    shape: (5,)
    Series: 'integer' [i64]
    [
    	1
    	2
    	3
    	4
    	5
    ]
    shape: (5,)
    Series: 'float' [f64]
    [
    	4.0
    	5.0
    	6.0
    	7.0
    	8.0
    ] |}]
;;
