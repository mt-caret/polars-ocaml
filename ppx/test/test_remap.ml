open! Core

module M0 = struct
  type t =
    { f1 : int
    ; f2 : float
    }
  [@@deriving polars]
end

let%expect_test "remap" =
  let df =
    Polars.Data_frame.create_exn
      Polars.Series.[ int "F1" [ 1; 2; 3 ]; float "F2" [ 1.; 2.; 3. ] ]
  in
  Expect_test_helpers_core.require_does_raise (fun () ->
    M0.Data_frame.of_polars_data_frame df);
  [%expect
    {|
    (Invalid_argument
     "Field(s) not found in series or in [remap] parameter: f1, f2")
    |}];
  let remap = String.Map.of_alist_exn [ "f1", "F1"; "f2", "F2" ] in
  M0.Data_frame.of_polars_data_frame ~remap df
  |> M0.Data_frame.to_polars_data_frame
  |> Polars.Data_frame.print;
  [%expect
    {|
    shape: (3, 2)
    ┌─────┬─────┐
    │ F1  ┆ F2  │
    │ --- ┆ --- │
    │ i64 ┆ f64 │
    ╞═════╪═════╡
    │ 1   ┆ 1.0 │
    │ 2   ┆ 2.0 │
    │ 3   ┆ 3.0 │
    └─────┴─────┘
    |}]
;;
