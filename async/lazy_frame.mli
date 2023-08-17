open! Core
open Async
open Polars
include module type of Lazy_frame

val collect : ?streaming:bool -> t -> (Data_frame.t, string) result Deferred.t
val collect_exn : ?streaming:bool -> t -> Data_frame.t Deferred.t
val collect_all : t list -> (Data_frame.t list, string) result Deferred.t
val collect_all_exn : t list -> Data_frame.t list Deferred.t
val profile : t -> (profile_result, string) result Deferred.t
val profile_exn : t -> profile_result Deferred.t
val fetch : t -> n_rows:int -> (Data_frame.t, string) result Deferred.t
val fetch_exn : t -> n_rows:int -> Data_frame.t Deferred.t
