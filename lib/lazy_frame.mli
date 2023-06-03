open! Core

type t

external scan_parquet : string -> (t, string) Caml.result = "rust_lazy_frame_scan_parquet"
external to_dot : t -> (string, string) Caml.result = "rust_lazy_frame_to_dot"
external collect : t -> (Data_frame0.t, string) Caml.result = "rust_lazy_frame_collect"
val collect_exn : t -> Data_frame0.t
external select : t -> exprs:Expr.t list -> t = "rust_lazy_frame_select"
