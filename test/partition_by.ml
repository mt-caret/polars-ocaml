open Core
open Polars

let%expect_test "partition by" =
  (* Same as polars python example. *)
  let df =
    Data_frame.create_exn
      Series.
        [ string "a" [ "a"; "b"; "a"; "b"; "c" ]
        ; int "b" [ 1; 2; 1; 3; 3 ]
        ; int "c" [ 5; 4; 3; 2; 1 ]
        ]
  in
  Data_frame.partition_by_exn df ~by:[ "a" ] |> List.iter ~f:Data_frame.print;
  [%expect
    {|
    shape: (2, 3)
    ┌─────┬─────┬─────┐
    │ a   ┆ b   ┆ c   │
    │ --- ┆ --- ┆ --- │
    │ str ┆ i64 ┆ i64 │
    ╞═════╪═════╪═════╡
    │ a   ┆ 1   ┆ 5   │
    │ a   ┆ 1   ┆ 3   │
    └─────┴─────┴─────┘
    shape: (2, 3)
    ┌─────┬─────┬─────┐
    │ a   ┆ b   ┆ c   │
    │ --- ┆ --- ┆ --- │
    │ str ┆ i64 ┆ i64 │
    ╞═════╪═════╪═════╡
    │ b   ┆ 2   ┆ 4   │
    │ b   ┆ 3   ┆ 2   │
    └─────┴─────┴─────┘
    shape: (1, 3)
    ┌─────┬─────┬─────┐
    │ a   ┆ b   ┆ c   │
    │ --- ┆ --- ┆ --- │
    │ str ┆ i64 ┆ i64 │
    ╞═════╪═════╪═════╡
    │ c   ┆ 3   ┆ 1   │
    └─────┴─────┴─────┘ |}]
;;
