open! Core

type t = Data_frame0.t

val create : Series.t list -> (t, string) result
val create_exn : Series.t list -> t
val read_csv : ?schema:Schema.t -> ?try_parse_dates:bool -> string -> (t, string) result
val read_csv_exn : ?schema:Schema.t -> ?try_parse_dates:bool -> string -> t
val write_csv : t -> string -> (unit, string) result
val write_csv_exn : t -> string -> unit
val read_parquet : string -> (t, string) result
val read_parquet_exn : string -> t
val write_parquet : t -> string -> (unit, string) result
val write_parquet_exn : t -> string -> unit
val read_json : string -> (t, string) result
val read_json_exn : string -> t
val write_json : t -> string -> (unit, string) result
val write_json_exn : t -> string -> unit
val read_jsonl : string -> (t, string) result
val read_jsonl_exn : string -> t
val write_jsonl : t -> string -> (unit, string) result
val write_jsonl_exn : t -> string -> unit
val clear : t -> t
val describe_exn : ?percentiles:float list -> t -> t
val height : t -> int
val estimated_size : t -> int
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
val as_single_chunk_par : t -> unit
val vstack : t -> other:t -> (unit, string) result
val vstack_exn : t -> other:t -> unit

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
val fill_null : t -> strategy:Fill_null_strategy.t -> (t, string) result
val fill_null_exn : t -> strategy:Fill_null_strategy.t -> t
val interpolate : t -> method_:[ `Linear | `Nearest ] -> (t, string) result
val interpolate_exn : t -> method_:[ `Linear | `Nearest ] -> t

val upsample
  :  ?stable:bool
  -> t
  -> by:string list
  -> time_column:string
  -> every:string
  -> offset:string
  -> (t, string) result

val upsample_exn
  :  ?stable:bool
  -> t
  -> by:string list
  -> time_column:string
  -> every:string
  -> offset:string
  -> t

val explode : t -> columns:string list -> (t, string) result
val explode_exn : t -> columns:string list -> t
val partition_by : ?maintain_order:bool -> t -> by:string list -> (t list, string) result
val partition_by_exn : ?maintain_order:bool -> t -> by:string list -> t list
val schema : t -> Schema.t
val to_string_hum : t -> string
val print : t -> unit
val pp : Format.formatter -> t -> unit [@@ocaml.toplevel_printer]

module Expert : sig
  (** Edit the values of a chunk of a non-nullable series in a dataframe.

      This function will result in undesired behavior when applied to a series containing
      any null values -- use [modify_optional_series_at_chunk_index] to get proper null
      handling. *)
  val modify_series_at_chunk_index
    :  t
    -> dtype:'a Data_type.Typed.t
    -> series_index:int (** The column number of the series to modify, 0-indexed. *)
    -> chunk_index:int
         (** Within the series, this is the index of the chunk to modify, 0-indexed. *)
    -> indices_and_values:(int * 'a) list
         (** A list of (index, value) tuples to set within the chunk. The index is 0-indexed
             and refers to an index within the chunk, not the entire series. Therefore, index
             should satisfy 0 <= index < chunk_length. *)
    -> (unit, string) result

  val modify_optional_series_at_chunk_index
    :  t
    -> dtype:'a Data_type.Typed.t
    -> series_index:int
    -> chunk_index:int
    -> indices_and_values:(int * 'a option) list
    -> (unit, string) result

  val clear_mut : t -> unit
end
