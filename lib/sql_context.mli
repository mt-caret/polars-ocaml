open! Core

type t

val create : (string * Lazy_frame.t) list -> t
val get_tables : t -> string list
val register : t -> name:string -> Lazy_frame.t -> unit
val unregister : t -> name:string -> unit
val execute : t -> query:string -> (Lazy_frame.t, string) result
val execute_exn : t -> query:string -> Lazy_frame.t
