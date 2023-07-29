open! Core

type t = Data_frame0.t

val create : Series.t list -> (t, string) result
val create_exn : Series.t list -> t
val read_csv : ?schema:Schema.t -> ?try_parse_dates:bool -> string -> (t, string) result
val read_csv_exn : ?schema:Schema.t -> ?try_parse_dates:bool -> string -> t
val describe : ?percentiles:float list -> t -> (t, string) result
val describe_exn : ?percentiles:float list -> t -> t
val lazy_ : t -> Lazy_frame.t
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
  -> (t, string) result

val groupby_dynamic_exn
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

val column : t -> name:string -> (Series.t, string) result
val column_exn : t -> name:string -> Series.t
val columns : t -> names:string list -> (Series.t list, string) result
val columns_exn : t -> names:string list -> Series.t list
val get_column_names : t -> string list
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

val sort
  :  ?descending:bool list
  -> ?maintain_order:bool
  -> t
  -> by_column:string list
  -> (t, string) result

val sort_exn
  :  ?descending:bool list
  -> ?maintain_order:bool
  -> t
  -> by_column:string list
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
val sum : t -> t
val mean : t -> t
val median : t -> t
val null_count : t -> t
val explode : t -> columns:string list -> (t, string) result
val explode_exn : t -> columns:string list -> t
val schema : t -> Schema.t
val to_string_hum : t -> string
val print : t -> unit
