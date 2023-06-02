open! Core

type t

external scan_parquet : string -> (t, string) result = "rust_lazy_frame_scan_parquet"
external of_data_frame : Data_frame.t -> t = "rust_data_frame_lazy"
external to_dot : t -> (string, string) result = "rust_lazy_frame_to_dot"
external collect : t -> (Data_frame.t, string) result = "rust_lazy_frame_collect"

let collect_exn t = collect t |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn

external select : t -> exprs:Expr.t list -> t = "rust_lazy_frame_select"
