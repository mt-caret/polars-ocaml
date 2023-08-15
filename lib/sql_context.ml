open! Core

type t

external create : unit -> t = "rust_sql_context_new"
external register : t -> name:string -> Lazy_frame.t -> unit = "rust_sql_context_register"

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

let execute_exn t ~query =
  execute t ~query |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
;;
