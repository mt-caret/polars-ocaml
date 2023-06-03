open! Core

type t = Data_frame0.t

external create : Series.t list -> (t, string) result = "rust_data_frame_new"

let create_exn series =
  create series |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
;;

external lazy_ : t -> Lazy_frame.t = "rust_data_frame_lazy"

let select t ~exprs = lazy_ t |> Lazy_frame.select ~exprs |> Lazy_frame.collect
let select_exn t ~exprs = lazy_ t |> Lazy_frame.select ~exprs |> Lazy_frame.collect_exn

external to_string_hum : t -> string = "rust_data_frame_to_string_hum"

let print t = print_endline (to_string_hum t)
