open! Core
open Async
open Polars
include module type of Sql_context

val execute_with_data_frames
  :  names_and_data_frames:(string * Data_frame.t) list
  -> query:string
  -> (Data_frame.t, string) result Deferred.t

val execute_with_data_frames_exn
  :  names_and_data_frames:(string * Data_frame.t) list
  -> query:string
  -> Data_frame.t Deferred.t

(** [vstack_and_collect] returns a [Data_frame.t] deferred instead of returning a
    [Lazy_frame.t] that you can [Lazy_frame.collect] on. The difference here is that a
    [Lazy_frame.t] may contain dangling references to the input data
    [names_and_data_frames] in its computation graph, whereas [Data_frame.t] generally
    does not, and generally we want to clean up any dangling references before mutating
    [names_and_data_frames] in the future. *)
val vstack_and_collect
  :  names_and_data_frames:(string * Data_frame.t list) list
  -> query:string
  -> (Data_frame.t, string) result Deferred.t

val execute : t -> query:string -> (Lazy_frame.t, string) result Deferred.t
val execute_exn : t -> query:string -> Lazy_frame.t Deferred.t
