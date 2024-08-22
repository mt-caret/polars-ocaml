open! Core

type t

val create : (string * Lazy_frame.t) list -> t
val get_tables : t -> string list
val register : t -> name:string -> Lazy_frame.t -> unit

val execute_with_data_frames
  :  names_and_data_frames:(string * Data_frame.t) list
  -> query:string
  -> (Data_frame.t, string) result

val execute_with_data_frames_exn
  :  names_and_data_frames:(string * Data_frame.t) list
  -> query:string
  -> Data_frame.t

(** Execute a query over [names_and_data_frames].
    Unlike [execute_with_data_frames], each data source is a list of
    data frames that can be concatenated together to produce the final result.

    As an example,
    [vstack_and_collect ~names_and_data_frames:["data1", [df1; df2; df3]]]
    roughly translates to
    {v
      let df = vstack [ df1; df2; df3 ] in
      execute_with_data_frames ~names_and_data_frames:[ "data1", df ]
    v} *)
val vstack_and_collect
  :  names_and_data_frames:(string * Data_frame.t list) list
  -> query:string
  -> (Data_frame.t, string) result

(** Like [vstack_and_collect] except it returns the query plan that would
    be executed instead of actually computing the result *)
val vstack_and_execute
  : names_and_data_frames:(string * Data_frame.t list) list
  -> query:string
  -> (Lazy_frame.t, string) result

val unregister : t -> name:string -> unit
val execute : t -> query:string -> (Lazy_frame.t, string) result
val execute_exn : t -> query:string -> Lazy_frame.t
