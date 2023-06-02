open! Core

type t

external scan_parquet : string -> (t, string) Caml.result = "rust_lazy_frame_scan_parquet"
external of_data_frame : Data_frame.t -> t = "rust_data_frame_lazy"
external to_dot : t -> (string, string) Caml.result = "rust_lazy_frame_to_dot"
external collect : t -> (Data_frame.t, string) Caml.result = "rust_lazy_frame_collect"
val collect_exn : t -> Data_frame.t
external select : t -> exprs:Expr.t list -> t = "rust_lazy_frame_select"
