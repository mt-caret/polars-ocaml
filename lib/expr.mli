open! Core

type t

val col : string -> t
val cols : string list -> t
val all : unit -> t
val exclude : string -> t
val element : unit -> t
val cast : ?strict:bool -> t -> to_:Data_type.t -> t
val null : unit -> t
val int : int -> t
val float : float -> t
val bool : bool -> t
val string : string -> t
val naive_date : Common.Naive_date.t -> t
val naive_datetime : Common.Naive_datetime.t -> t
val sort : ?descending:bool -> t -> t
val sort_by : ?descending:bool -> t -> by:t list -> t
val first : t -> t
val last : t -> t
val reverse : t -> t
val head : ?length:int -> t -> t
val tail : ?length:int -> t -> t

val sample_n
  :  ?seed:int
  -> ?fixed_seed:bool
  -> t
  -> n:int
  -> with_replacement:bool
  -> shuffle:bool
  -> t

val filter : t -> predicate:t -> t
val sum : t -> t
val mean : t -> t
val median : t -> t
val max : t -> t
val min : t -> t
val count : t -> t
val count_ : unit -> t
val n_unique : t -> t
val approx_unique : t -> t
val explode : t -> t

val over
  :  ?mapping_strategy:[ `Groups_to_rows | `Explode | `Join ]
  -> t
  -> partition_by:t list
  -> t

val concat_list : t Nonempty_list.t -> t
val null_count : t -> t
val is_null : t -> t
val is_not_null : t -> t
val fill_null : t -> with_:t -> t
val fill_null' : t -> strategy:Fill_null_strategy.t -> t
val interpolate : ?method_:[ `Linear | `Nearest ] -> t -> t
val fill_nan : t -> with_:t -> t

val rank
  :  ?method_:[ `Average | `Dense | `Max | `Min | `Ordinal | `Random ]
  -> ?descending:bool
  -> ?seed:int
  -> t
  -> t

val when_ : (t * t) list -> otherwise:t -> t
val shift : ?fill_value:t -> t -> periods:int -> t
val cum_count : ?reverse:bool -> t -> t
val cum_sum : ?reverse:bool -> t -> t
val cum_prod : ?reverse:bool -> t -> t
val cum_min : ?reverse:bool -> t -> t
val cum_max : ?reverse:bool -> t -> t
val alias : t -> name:string -> t
val prefix : t -> prefix:string -> t
val suffix : t -> suffix:string -> t
val round : t -> decimals:int -> t
val equal : t -> t -> t

include Common.Compare with type t := t
include Common.Logic with type t := t
include Common.Numeric with type t := t

(* TODO: apparently this doesn't exist for series, which is surprising! *)
val floor_div : t -> t -> t
val ( // ) : t -> t -> t

module Dt : sig
  val strftime : t -> format:string -> t
  val convert_time_zone : t -> to_:string -> t
  val replace_time_zone : ?use_earliest:bool -> t -> to_:string option -> t
  val year : t -> t
  val month : t -> t
  val day : t -> t
  val days : t -> t
  val hours : t -> t
  val minutes : t -> t
  val seconds : t -> t
  val milliseconds : t -> t
  val microseconds : t -> t
  val nanoseconds : t -> t
end

module Str : sig
  val split : ?inclusive:bool -> t -> by:string -> t
  val strptime : t -> type_:Data_type.t -> format:string -> t
  val lengths : t -> t
  val n_chars : t -> t
  val contains : ?literal:bool -> t -> pat:string -> t
  val starts_with : t -> prefix:string -> t
  val ends_with : t -> suffix:string -> t
  val extract : t -> pat:string -> group:int -> t
  val extract_all : t -> pat:string -> t
  val replace : ?literal:bool -> t -> pat:string -> with_:string -> t
  val replace_all : ?literal:bool -> t -> pat:string -> with_:string -> t
end

module List : sig
  val lengths : t -> t
  val slice : t -> offset:t -> length:t -> t
  val head : t -> n:t -> t
  val tail : t -> n:t -> t
  val sum : t -> t
  val eval : ?parallel:bool -> t -> expr:t -> t
end
