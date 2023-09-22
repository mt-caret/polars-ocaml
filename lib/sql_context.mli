open! Core

type t

val create : (string * Lazy_frame.t) list -> t
val get_tables : t -> string list
val register : t -> name:string -> Lazy_frame.t -> unit

val rust_sql_context_execute_with_data_frames
  :  data_frames_with_names:(Data_frame.t * string) list
  -> query:string
  -> (Data_frame.t, string) result

val rust_sql_context_execute_with_data_frames_exn
  :  data_frames_with_names:(Data_frame.t * string) list
  -> query:string
  -> Data_frame.t

val unregister : t -> name:string -> unit
val execute : t -> query:string -> (Lazy_frame.t, string) result
val execute_exn : t -> query:string -> Lazy_frame.t
