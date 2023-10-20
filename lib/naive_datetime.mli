open Core

type t

val of_naive_date : ?hour:int -> ?min:int -> ?sec:int -> Naive_date.t -> t
val of_date : ?hour:int -> ?min:int -> ?sec:int -> Date.t -> t
val to_string : t -> string
val of_string : string -> t
val of_time_ns : Time_ns.t -> t option
val of_time_ns_exn : Time_ns.t -> t
val to_time_ns : t -> Time_ns.t
