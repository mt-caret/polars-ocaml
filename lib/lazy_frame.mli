open! Core

type t

val scan_parquet : string -> (t, string) result
val scan_parquet_exn : string -> t
val scan_csv : string -> (t, string) result
val scan_csv_exn : string -> t
val explain : ?optimized:bool -> t -> (string, string) result
val explain_exn : ?optimized:bool -> t -> string
val to_dot : ?optimized:bool -> t -> (string, string) result
val to_dot_exn : ?optimized:bool -> t -> string
val cache : t -> t
val collect : ?streaming:bool -> t -> (Data_frame0.t, string) result
val collect_exn : ?streaming:bool -> t -> Data_frame0.t
val collect_all : t list -> (Data_frame0.t list, string) result
val collect_all_exn : t list -> Data_frame0.t list
val profile : t -> (Data_frame0.t * Data_frame0.t, string) result
val profile_exn : t -> Data_frame0.t * Data_frame0.t
val fetch : t -> n_rows:int -> (Data_frame0.t, string) result
val fetch_exn : t -> n_rows:int -> Data_frame0.t
val filter : t -> predicate:Expr.t -> t
val select : t -> exprs:Expr.t list -> t
val with_columns : t -> exprs:Expr.t list -> t
val groupby : ?is_stable:bool -> t -> by:Expr.t list -> agg:Expr.t list -> t

val groupby_dynamic
  :  ?every:string
  -> ?period:string
  -> ?offset:string
  -> ?truncate:bool
  -> ?include_boundaries:bool
  -> ?closed_window:[ `Both | `Left | `None_ | `Right ]
  -> ?start_by:
       [ `Data_point
       | `Friday
       | `Monday
       | `Saturday
       | `Sunday
       | `Thursday
       | `Tuesday
       | `Wednesday
       | `Window_bound
       ]
  -> ?check_sorted:bool
  -> t
  -> index_column:Expr.t
  -> by:Expr.t list
  -> agg:Expr.t list
  -> t

val join : t -> other:t -> on:Expr.t list -> how:Join_type.t -> t

val join'
  :  t
  -> other:t
  -> left_on:Expr.t list
  -> right_on:Expr.t list
  -> how:Join_type.t
  -> t

val concat
  :  ?how:[ `Diagonal | `Vertical | `Vertical_relaxed ]
  -> ?rechunk:bool
  -> ?parallel:bool
  -> t list
  -> t

val melt
  :  ?variable_name:string
  -> ?value_name:string
  -> ?streamable:bool
  -> t
  -> id_vars:string list
  -> value_vars:string list
  -> t

val sort : ?descending:bool -> ?nulls_last:bool -> t -> by_column:string -> t
val limit : t -> n:int -> t
val explode : t -> columns:Expr.t list -> t
val with_streaming : t -> toggle:bool -> t
val schema : t -> (Schema.t, string) result
val schema_exn : t -> Schema.t
