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

external scan_jsonl : string -> (t, string) result = "rust_lazy_frame_scan_jsonl"

let scan_jsonl_exn path =
  scan_jsonl path |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
;;

external explain
  :  t
  -> optimized:bool
  -> (string, string) result
  = "rust_lazy_frame_explain"

let explain ?(optimized = true) t = explain t ~optimized

let explain_exn ?optimized t =
  explain ?optimized t |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
;;

external to_dot
  :  t
  -> optimized:bool
  -> (string, string) result
  = "rust_lazy_frame_to_dot"

let to_dot ?(optimized = true) t = to_dot t ~optimized

let to_dot_exn ?optimized t =
  to_dot ?optimized t |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
;;

external cache : t -> t = "rust_lazy_frame_cache"

external collect
  :  t
  -> streaming:bool
  -> release_runtime:bool
  -> (Data_frame0.t, string) result
  = "rust_lazy_frame_collect"

external collect_all
  :  t list
  -> release_runtime:bool
  -> (Data_frame0.t list, string) result
  = "rust_lazy_frame_collect_all"

external profile
  :  t
  -> release_runtime:bool
  -> (Data_frame0.t * Data_frame0.t, string) result
  = "rust_lazy_frame_profile"

external fetch
  :  t
  -> n_rows:int
  -> release_runtime:bool
  -> (Data_frame0.t, string) result
  = "rust_lazy_frame_fetch"

type profile_result =
  { collected : Data_frame0.t
  ; profile : Data_frame0.t
  }

module Deferred = struct
  open Async

  let collect ?(streaming = false) t =
    In_thread.run (fun () -> collect t ~streaming ~release_runtime:true)
  ;;

  let collect_exn ?streaming t =
    collect ?streaming t >>| Result.map_error ~f:Error.of_string >>| Or_error.ok_exn
  ;;

  let collect_all t = In_thread.run (fun () -> collect_all t ~release_runtime:true)

  let collect_all_exn ts =
    collect_all ts >>| Result.map_error ~f:Error.of_string >>| Or_error.ok_exn
  ;;

  let profile t =
    let%map result = In_thread.run (fun () -> profile t ~release_runtime:true) in
    let%map.Result collected, profile = result in
    { collected; profile }
  ;;

  let profile_exn t =
    profile t >>| Result.map_error ~f:Error.of_string >>| Or_error.ok_exn
  ;;

  let fetch t ~n_rows = In_thread.run (fun () -> fetch t ~n_rows ~release_runtime:true )

  let fetch_exn t ~n_rows =
    fetch t ~n_rows >>| Result.map_error ~f:Error.of_string >>| Or_error.ok_exn
  ;;
end

let collect ?(streaming = false) t = collect t ~streaming ~release_runtime:false

let collect_exn ?streaming t =
  collect ?streaming t |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
;;

let collect_all t = collect_all t ~release_runtime:false

let collect_all t = collect_all t

let collect_all_exn ts =
  collect_all ts |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
;;

let profile t =
  let%map.Result collected, profile = profile t ~release_runtime:false in
  { collected; profile }
;;

type profile_result =
  { collected : Data_frame0.t
  ; profile : Data_frame0.t
  }

let profile t =
  let%map.Result collected, profile = profile t in
  { collected; profile }
;;

let profile_exn t = profile t |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
let fetch t ~n_rows = fetch t ~n_rows ~release_runtime:false

let fetch t ~n_rows = fetch t ~n_rows

let fetch_exn t ~n_rows =
  fetch t ~n_rows |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
;;

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

external groupby_dynamic
  :  t
  -> index_column:Expr.t
  -> by:Expr.t list
  -> every:string option
  -> period:string option
  -> offset:string option
  -> truncate:bool option
  -> include_boundaries:bool option
  -> closed_window:[ `Left | `Right | `Both | `None_ ] option
  -> start_by:
       [ `Window_bound
       | `Data_point
       | `Monday
       | `Tuesday
       | `Wednesday
       | `Thursday
       | `Friday
       | `Saturday
       | `Sunday
       ]
       option
  -> check_sorted:bool option
  -> agg:Expr.t list
  -> t
  = "rust_lazy_frame_groupby_dynamic_bytecode" "rust_lazy_frame_groupby_dynamic"

let groupby_dynamic
  ?every
  ?period
  ?offset
  ?truncate
  ?include_boundaries
  ?closed_window
  ?start_by
  ?check_sorted
  t
  ~index_column
  ~by
  ~agg
  =
  (* Following the logic of:
     https://github.com/pola-rs/polars/blob/a8489558008652fe06e182dbdf082e8d9f0159ab/py-polars/polars/lazyframe/frame.py#L2327
  *)
  let offset =
    Option.value
      offset
      ~default:
        (match period with
         | None -> "-" ^ Option.value_exn every
         | Some _ -> "0ns")
  in
  let period = Option.value period ~default:(Option.value_exn every) in
  groupby_dynamic
    t
    ~index_column
    ~by
    ~every
    ~period:(Some period)
    ~offset:(Some offset)
    ~truncate
    ~include_boundaries
    ~closed_window
    ~start_by
    ~check_sorted
    ~agg
;;

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
  -> to_supertypes:bool
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
  | `Vertical -> vertical_concat ts ~rechunk ~parallel ~to_supertypes:false
  | `Vertical_relaxed -> vertical_concat ts ~rechunk ~parallel ~to_supertypes:true
  | `Diagonal -> diagonal_concat ts ~rechunk ~parallel
;;

external melt
  :  t
  -> id_vars:string list
  -> value_vars:string list
  -> variable_name:string option
  -> value_name:string option
  -> streamable:bool
  -> t
  = "rust_lazy_frame_melt_bytecode" "rust_lazy_frame_melt"

let melt ?variable_name ?value_name ?(streamable = false) t ~id_vars ~value_vars =
  melt t ~id_vars ~value_vars ~variable_name ~value_name ~streamable
;;

external sort
  :  t
  -> by_column:string
  -> descending:bool option
  -> nulls_last:bool option
  -> multithreaded:bool option
  -> maintain_order:bool option
  -> t
  = "rust_lazy_frame_sort_bytecode" "rust_lazy_frame_sort"

let sort ?descending ?nulls_last t ~by_column =
  sort
    t
    ~by_column
    ~descending
    ~nulls_last (* TODO: make following parameters configurable *)
    ~multithreaded:(Some false)
    ~maintain_order:(Some true)
;;

external limit : t -> n:int -> t = "rust_lazy_frame_limit"
external explode : t -> columns:Expr.t list -> t = "rust_lazy_frame_explode"
external with_streaming : t -> toggle:bool -> t = "rust_lazy_frame_with_streaming"
external schema : t -> (Schema.t, string) result = "rust_lazy_frame_schema"

let schema_exn t = schema t |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
