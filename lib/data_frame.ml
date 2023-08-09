open! Core

type t = Data_frame0.t

external create : Series.t list -> (t, string) result = "rust_data_frame_new"

let create_exn series =
  create series |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
;;

external read_csv
  :  string
  -> schema:Schema.t option
  -> try_parse_dates:bool option
  -> (t, string) result
  = "rust_data_frame_read_csv"

let read_csv ?schema ?try_parse_dates path = read_csv path ~schema ~try_parse_dates

let read_csv_exn ?schema ?try_parse_dates path =
  read_csv ?schema ?try_parse_dates path
  |> Result.map_error ~f:Error.of_string
  |> Or_error.ok_exn
;;

external describe
  :  t
  -> percentiles:float list option
  -> (t, string) result
  = "rust_data_frame_describe"

let describe ?percentiles t = describe t ~percentiles

let describe_exn ?percentiles t =
  describe ?percentiles t |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
;;

external lazy_ : t -> Lazy_frame.t = "rust_data_frame_lazy"

let in_lazy t ~f = lazy_ t |> f |> Lazy_frame.collect
let in_lazy_exn t ~f = lazy_ t |> f |> Lazy_frame.collect_exn
let select t ~exprs = in_lazy t ~f:(Lazy_frame.select ~exprs)
let select_exn t ~exprs = in_lazy_exn t ~f:(Lazy_frame.select ~exprs)
let with_columns t ~exprs = in_lazy t ~f:(Lazy_frame.with_columns ~exprs)
let with_columns_exn t ~exprs = in_lazy_exn t ~f:(Lazy_frame.with_columns ~exprs)
let groupby ?is_stable t ~by ~agg = in_lazy t ~f:(Lazy_frame.groupby ?is_stable ~by ~agg)

let groupby_exn ?is_stable t ~by ~agg =
  in_lazy_exn t ~f:(Lazy_frame.groupby ?is_stable ~by ~agg)
;;

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
  in_lazy
    t
    ~f:
      (Lazy_frame.groupby_dynamic
         ?every
         ?period
         ?offset
         ?truncate
         ?include_boundaries
         ?closed_window
         ?start_by
         ?check_sorted
         ~index_column
         ~by
         ~agg)
;;

let groupby_dynamic_exn
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
  in_lazy_exn
    t
    ~f:
      (Lazy_frame.groupby_dynamic
         ?every
         ?period
         ?offset
         ?truncate
         ?include_boundaries
         ?closed_window
         ?start_by
         ?check_sorted
         ~index_column
         ~by
         ~agg)
;;

external column : t -> name:string -> (Series.t, string) result = "rust_data_frame_column"

let column_exn t ~name =
  column t ~name |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
;;

external columns
  :  t
  -> names:string list
  -> (Series.t list, string) result
  = "rust_data_frame_columns"

let columns_exn t ~names =
  columns t ~names |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
;;

external get_column_names : t -> string list = "rust_data_frame_get_column_names"

external vertical_concat
  :  t list
  -> (t, string) result
  = "rust_data_frame_vertical_concat"

external horizontal_concat
  :  t list
  -> (t, string) result
  = "rust_data_frame_horizontal_concat"

external diagonal_concat
  :  t list
  -> (t, string) result
  = "rust_data_frame_diagonal_concat"

let concat ?(how = `Vertical) ts =
  match how with
  | `Vertical -> vertical_concat ts
  | `Horizontal -> horizontal_concat ts
  | `Diagonal -> diagonal_concat ts
;;

let concat_exn ?how ts =
  concat ?how ts |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
;;

external pivot
  :  t
  -> values:string list
  -> index:string list
  -> columns:string list
  -> sort_columns:bool
  -> agg_expr:Expr.t option
  -> separator:string option
  -> stable:bool
  -> (t, string) result
  = "rust_data_frame_pivot_bytecode" "rust_data_frame_pivot"

let pivot
  ?agg_expr
  ?(sort_columns = false)
  ?separator
  ?(stable = true)
  t
  ~values
  ~index
  ~columns
  =
  let agg_expr =
    Option.map
      agg_expr
      ~f:
        Expr.(
          function
          | `First -> element () |> first
          | `Sum -> element () |> sum
          | `Max -> element () |> max
          | `Min -> element () |> min
          | `Mean -> element () |> mean
          | `Median -> element () |> median
          | `Last -> element () |> last
          | `Count -> count_ ()
          | `Expr expr -> expr)
  in
  pivot t ~values ~index ~columns ~sort_columns ~agg_expr ~separator ~stable
;;

let pivot_exn ?agg_expr ?sort_columns ?separator ?stable t ~values ~index ~columns =
  pivot ?agg_expr ?sort_columns ?separator ?stable t ~values ~index ~columns
  |> Result.map_error ~f:Error.of_string
  |> Or_error.ok_exn
;;

external melt
  :  t
  -> id_vars:string list
  -> value_vars:string list
  -> variable_name:string option
  -> value_name:string option
  -> streamable:bool
  -> (t, string) result
  = "rust_data_frame_melt_bytecode" "rust_data_frame_melt"

let melt ?variable_name ?value_name ?(streamable = false) t ~id_vars ~value_vars =
  melt t ~id_vars ~value_vars ~variable_name ~value_name ~streamable
;;

let melt_exn ?variable_name ?value_name ?streamable t ~id_vars ~value_vars =
  melt ?variable_name ?value_name ?streamable t ~id_vars ~value_vars
  |> Result.map_error ~f:Error.of_string
  |> Or_error.ok_exn
;;

external sort
  :  t
  -> by_column:string list
  -> descending:bool list
  -> maintain_order:bool
  -> (t, string) result
  = "rust_data_frame_sort"

let sort ?descending ?(maintain_order = true) t ~by_column =
  let descending =
    Option.value descending ~default:(List.map by_column ~f:(Fn.const false))
  in
  sort t ~by_column ~descending ~maintain_order
;;

let sort_exn ?descending ?maintain_order t ~by_column =
  sort ?descending ?maintain_order t ~by_column
  |> Result.map_error ~f:Error.of_string
  |> Or_error.ok_exn
;;

external head : t -> length:int option -> t = "rust_data_frame_head"

let head ?length t = head t ~length

external tail : t -> length:int option -> t = "rust_data_frame_tail"

let tail ?length t = tail t ~length

external sample_n
  :  t
  -> n:int
  -> with_replacement:bool
  -> shuffle:bool
  -> seed:int option
  -> (t, string) result
  = "rust_data_frame_sample_n"

let sample_n ?seed t ~n ~with_replacement ~shuffle =
  sample_n t ~n ~with_replacement ~shuffle ~seed
;;

let sample_n_exn ?seed t ~n ~with_replacement ~shuffle =
  sample_n ?seed t ~n ~with_replacement ~shuffle
  |> Result.map_error ~f:Error.of_string
  |> Or_error.ok_exn
;;

external sum : t -> t = "rust_data_frame_sum"
external mean : t -> t = "rust_data_frame_mean"
external median : t -> t = "rust_data_frame_median"
external null_count : t -> t = "rust_data_frame_null_count"

external explode
  :  t
  -> columns:string list
  -> (t, string) result
  = "rust_data_frame_explode"

let explode_exn t ~columns =
  explode t ~columns |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
;;

external schema : t -> Schema.t = "rust_data_frame_schema"
external to_string_hum : t -> string = "rust_data_frame_to_string_hum"

let print t = print_endline (to_string_hum t)
