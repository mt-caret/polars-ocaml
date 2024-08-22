open! Core

type t

external create : unit -> t = "rust_sql_context_new"
external register : t -> name:string -> Lazy_frame.t -> unit = "rust_sql_context_register"

external execute_with_data_frames
  :  names_and_data_frames:(string * Data_frame.t) list
  -> query:string
  -> (Data_frame.t, string) result
  = "rust_sql_context_execute_with_data_frames"

let execute_with_data_frames_exn ~names_and_data_frames ~query =
  execute_with_data_frames ~names_and_data_frames ~query |> Utils.string_result_ok_exn
;;

external vstack_and_collect
  :  names_and_data_frames:(string * Data_frame.t list) list
  -> query:string
  -> (Data_frame.t, string) result
  = "rust_sql_context_vstack_and_collect"

external vstack_and_execute
  :  names_and_data_frames:(string * Data_frame.t list) list
  -> query:string
  -> (Lazy_frame.t, string) result
  = "rust_sql_context_vstack_and_execute"


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
