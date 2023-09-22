open! Core

type t

external create : unit -> t = "rust_sql_context_new"
external register : t -> name:string -> Lazy_frame.t -> unit = "rust_sql_context_register"

external rust_sql_context_execute_with_data_frames'
  :  data_frames:Data_frame.t list
  -> names:string list
  -> query:string
  -> (Data_frame.t, string) result
  = "rust_sql_context_execute_with_data_frames"

let rust_sql_context_execute_with_data_frames ~data_frames_with_names ~query =
  let data_frames, names = List.unzip data_frames_with_names in
  rust_sql_context_execute_with_data_frames' ~data_frames ~names ~query
;;

let rust_sql_context_execute_with_data_frames_exn ~data_frames_with_names ~query =
  rust_sql_context_execute_with_data_frames ~data_frames_with_names ~query
  |> Utils.string_result_ok_exn
;;

let create tables =
  let t = create () in
  List.iter tables ~f:(fun (name, lazy_frame) -> register t ~name lazy_frame);
  t
;;

external get_tables : t -> string list = "rust_sql_context_get_tables"
external unregister : t -> name:string -> unit = "rust_sql_context_unregister"

external execute
  :  t
  -> query:string
  -> (Lazy_frame.t, string) result
  = "rust_sql_context_execute"

let execute_exn t ~query = execute t ~query |> Utils.string_result_ok_exn
