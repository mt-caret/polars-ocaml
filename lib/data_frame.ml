open! Core

type t = Data_frame0.t

external create : Series.t list -> (t, string) result = "rust_data_frame_new"

let create_exn series =
  create series |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
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

let select t ~exprs = lazy_ t |> Lazy_frame.select ~exprs |> Lazy_frame.collect
let select_exn t ~exprs = lazy_ t |> Lazy_frame.select ~exprs |> Lazy_frame.collect_exn

let with_columns t ~exprs =
  lazy_ t |> Lazy_frame.with_columns ~exprs |> Lazy_frame.collect
;;

let with_columns_exn t ~exprs =
  lazy_ t |> Lazy_frame.with_columns ~exprs |> Lazy_frame.collect_exn
;;

let groupby ?is_stable t ~by ~agg =
  lazy_ t |> Lazy_frame.groupby ?is_stable ~by ~agg |> Lazy_frame.collect
;;

let groupby_exn ?is_stable t ~by ~agg =
  lazy_ t |> Lazy_frame.groupby ?is_stable ~by ~agg |> Lazy_frame.collect_exn
;;

external head : t -> length:int option -> t option = "rust_data_frame_head"

let head ?length t = head t ~length |> Option.value_exn ~here:[%here]

external tail : t -> length:int option -> t option = "rust_data_frame_tail"

let tail ?length t = tail t ~length |> Option.value_exn ~here:[%here]

external sample_n
  :  t
  -> n:int
  -> with_replacement:bool
  -> shuffle:bool
  -> seed:int option
  -> (t, string) result option
  = "rust_data_frame_sample_n"

let sample_n ?seed t ~n ~with_replacement ~shuffle =
  sample_n t ~n ~with_replacement ~shuffle ~seed |> Option.value_exn ~here:[%here]
;;

let sample_n_exn ?seed t ~n ~with_replacement ~shuffle =
  sample_n ?seed t ~n ~with_replacement ~shuffle
  |> Result.map_error ~f:Error.of_string
  |> Or_error.ok_exn
;;

external schema : t -> Schema.t = "rust_data_frame_schema"
external to_string_hum : t -> string = "rust_data_frame_to_string_hum"

let print t = print_endline (to_string_hum t)
