open! Core

type t = Data_frame0.t

external create : Series.t list -> (t, string) result = "rust_data_frame_new"
val create_exn : Series.t list -> t
external lazy_ : t -> Lazy_frame.t = "rust_data_frame_lazy"
val select : t -> exprs:Expr.t list -> (t, string) result
val select_exn : t -> exprs:Expr.t list -> t
external to_string_hum : t -> string = "rust_data_frame_to_string_hum"
val print : t -> unit
