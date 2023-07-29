open Core
open Polars

let () =
  let df =
    Data_frame.read_csv_exn ~try_parse_dates:true "../test/data/appleStock.csv"
    |> Data_frame.sort_exn ~by_column:[ "Date" ]
  in
  Data_frame.print df;
  let annual_average_df =
    Data_frame.groupby_dynamic_exn
      df
      ~index_column:(Expr.col "Date")
      ~every:"1y"
      ~by:[]
      ~agg:Expr.[ col "Close" |> mean ]
  in
  Data_frame.print annual_average_df
;;
