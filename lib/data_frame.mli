open! Core

type t = Data_frame0.t

external create : Series.t list -> (t, string) result = "rust_data_frame_new"
val create_exn : Series.t list -> t
val read_csv : ?schema:Schema.t -> ?try_parse_dates:bool -> string -> (t, string) result
val read_csv_exn : ?schema:Schema.t -> ?try_parse_dates:bool -> string -> t
val describe : ?percentiles:float list -> t -> (t, string) result
val describe_exn : ?percentiles:float list -> t -> t
external lazy_ : t -> Lazy_frame.t = "rust_data_frame_lazy"
val select : t -> exprs:Expr.t list -> (t, string) result
val select_exn : t -> exprs:Expr.t list -> t
val with_columns : t -> exprs:Expr.t list -> (t, string) result
val with_columns_exn : t -> exprs:Expr.t list -> t

val groupby
  :  ?is_stable:bool
  -> t
  -> by:Expr.t list
  -> agg:Expr.t list
  -> (t, string) result

val groupby_exn : ?is_stable:bool -> t -> by:Expr.t list -> agg:Expr.t list -> t
external column : t -> name:string -> (Series.t, string) result = "rust_data_frame_column"
val column_exn : t -> name:string -> Series.t

external columns
  :  t
  -> names:string list
  -> (Series.t list, string) result
  = "rust_data_frame_column"

val columns_exn : t -> names:string list -> Series.t list
val concat : ?how:[ `Diagonal | `Horizontal | `Vertical ] -> t list -> (t, string) result
val concat_exn : ?how:[ `Diagonal | `Horizontal | `Vertical ] -> t list -> t

val pivot
  :  ?agg_expr:
       [ `Count
       | `Expr of Expr.t
       | `First
       | `Last
       | `Max
       | `Mean
       | `Median
       | `Min
       | `Sum
       ]
  -> ?sort_columns:bool
  -> ?separator:string
  -> ?stable:bool
  -> t
  -> values:string list
  -> index:string list
  -> columns:string list
  -> (t, string) result

val pivot_exn
  :  ?agg_expr:
       [ `Count
       | `Expr of Expr.t
       | `First
       | `Last
       | `Max
       | `Mean
       | `Median
       | `Min
       | `Sum
       ]
  -> ?sort_columns:bool
  -> ?separator:string
  -> ?stable:bool
  -> t
  -> values:string list
  -> index:string list
  -> columns:string list
  -> t

val melt
  :  ?variable_name:string
  -> ?value_name:string
  -> ?streamable:bool
  -> t
  -> id_vars:string list
  -> value_vars:string list
  -> (t, string) result

val melt_exn
  :  ?variable_name:string
  -> ?value_name:string
  -> ?streamable:bool
  -> t
  -> id_vars:string list
  -> value_vars:string list
  -> t

val head : ?length:int -> t -> t
val tail : ?length:int -> t -> t

val sample_n
  :  ?seed:int
  -> t
  -> n:int
  -> with_replacement:bool
  -> shuffle:bool
  -> (t, string) result

val sample_n_exn : ?seed:int -> t -> n:int -> with_replacement:bool -> shuffle:bool -> t
external sum : t -> t = "rust_data_frame_sum"
external mean : t -> t = "rust_data_frame_mean"
external median : t -> t = "rust_data_frame_median"
external null_count : t -> t = "rust_data_frame_null_count"

external explode
  :  t
  -> columns:string list
  -> (t, string) result
  = "rust_data_frame_explode"

val explode_exn : t -> columns:string list -> t
external schema : t -> Schema.t = "rust_data_frame_schema"
external to_string_hum : t -> string = "rust_data_frame_to_string_hum"
val print : t -> unit
