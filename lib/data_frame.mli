open! Core

type t = Data_frame0.t

external create : Series.t list -> (t, string) result = "rust_data_frame_new"
val create_exn : Series.t list -> t
external read_csv : string -> (t, string) result = "rust_data_frame_read_csv"
val read_csv_exn : string -> t
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
external schema : t -> Schema.t = "rust_data_frame_schema"
external to_string_hum : t -> string = "rust_data_frame_to_string_hum"
val print : t -> unit
