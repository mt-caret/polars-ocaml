open! Core

type t

external scan_parquet : string -> (t, string) result = "rust_lazy_frame_scan_parquet"

let scan_parquet_exn path =
  scan_parquet path |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
;;

external scan_csv : string -> (t, string) result = "rust_lazy_frame_scan_csv"

let scan_csv_exn path =
  scan_csv path |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
;;

external to_dot : t -> (string, string) result = "rust_lazy_frame_to_dot"
external collect : t -> (Data_frame0.t, string) result = "rust_lazy_frame_collect"

let collect_exn t = collect t |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn

external filter : t -> predicate:Expr.t -> t = "rust_lazy_frame_filter"
external select : t -> exprs:Expr.t list -> t = "rust_lazy_frame_select"
external with_columns : t -> exprs:Expr.t list -> t = "rust_lazy_frame_with_columns"

external groupby
  :  t
  -> is_stable:bool
  -> by:Expr.t list
  -> agg:Expr.t list
  -> t
  = "rust_lazy_frame_groupby"

let groupby ?(is_stable = false) t ~by ~agg = groupby t ~is_stable ~by ~agg

external join_
  :  t
  -> other:t
  -> left_on:Expr.t list
  -> right_on:Expr.t list
  -> how:Join_type.t
  -> t
  = "rust_lazy_frame_join"

let join t ~other ~on ~how = join_ t ~other ~left_on:on ~right_on:on ~how
let join' = join_

external vertical_concat
  :  t list
  -> rechunk:bool
  -> parallel:bool
  -> t
  = "rust_lazy_frame_vertical_concat"

external diagonal_concat
  :  t list
  -> rechunk:bool
  -> parallel:bool
  -> t
  = "rust_lazy_frame_diagonal_concat"

let concat ?(how = `Vertical) ?(rechunk = false) ?(parallel = false) ts =
  match how with
  | `Vertical -> vertical_concat ts ~rechunk ~parallel
  | `Diagonal -> diagonal_concat ts ~rechunk ~parallel
;;

external sort
  :  t
  -> by_column:string
  -> descending:bool option
  -> nulls_last:bool option
  -> multithreaded:bool option
  -> t
  = "rust_lazy_frame_sort"

external limit : t -> n:int -> t option = "rust_lazy_frame_limit"

let limit t ~n = limit t ~n |> Option.value_exn ~here:[%here]

let sort ?descending ?nulls_last t ~by_column =
  sort t ~by_column ~descending ~nulls_last ~multithreaded:(Some false)
;;

external explode : t -> columns:Expr.t list -> t = "rust_lazy_frame_explode"
external with_streaming : t -> toggle:bool -> t = "rust_lazy_frame_with_streaming"
external schema : t -> (Schema.t, string) result = "rust_lazy_frame_schema"

let schema_exn t = schema t |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
