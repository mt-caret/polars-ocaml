open! Core

type t

external scan_parquet : string -> (t, string) result = "rust_lazy_frame_scan_parquet"
val scan_parquet_exn : string -> t
external scan_csv : string -> (t, string) result = "rust_lazy_frame_scan_csv"
val scan_csv_exn : string -> t
external to_dot : t -> (string, string) result = "rust_lazy_frame_to_dot"
external collect : t -> (Data_frame0.t, string) result = "rust_lazy_frame_collect"
val collect_exn : t -> Data_frame0.t
external filter : t -> predicate:Expr.t -> t = "rust_lazy_frame_filter"
external select : t -> exprs:Expr.t list -> t = "rust_lazy_frame_select"
external with_columns : t -> exprs:Expr.t list -> t = "rust_lazy_frame_with_columns"
val groupby : ?is_stable:bool -> t -> by:Expr.t list -> agg:Expr.t list -> t
val join : t -> other:t -> on:Expr.t list -> how:Join_type.t -> t

val join'
  :  t
  -> other:t
  -> left_on:Expr.t list
  -> right_on:Expr.t list
  -> how:Join_type.t
  -> t

val concat
  :  ?how:[ `Diagonal | `Vertical ]
  -> ?rechunk:bool
  -> ?parallel:bool
  -> t list
  -> t

val sort : ?descending:bool -> ?nulls_last:bool -> t -> by_column:string -> t
val limit : t -> n:int -> t
external explode : t -> columns:Expr.t list -> t = "rust_lazy_frame_explode"
external with_streaming : t -> toggle:bool -> t = "rust_lazy_frame_with_streaming"
external schema : t -> (Schema.t, string) result = "rust_lazy_frame_schema"
val schema_exn : t -> Schema.t
