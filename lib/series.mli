open! Core

type t

val create : 'a Data_type.Typed.t -> string -> 'a list -> t
val createo : 'a Data_type.Typed.t -> string -> 'a option list -> t
val create' : 'a Data_type.Typed.t -> string -> 'a array -> t
val createo' : 'a Data_type.Typed.t -> string -> 'a option array -> t
val int : string -> int list -> t
val into : string -> int option list -> t
val float : string -> float list -> t
val floato : string -> float option list -> t
val float' : string -> floatarray -> t
val bool : string -> bool list -> t
val boolo : string -> bool option list -> t
val string : string -> string list -> t
val stringo : string -> string option list -> t
val date : string -> Naive_date.t list -> t
val dateo : string -> Naive_date.t option list -> t
val date' : string -> Date.t list -> t
val dateo' : string -> Date.t option list -> t
val datetime : string -> Naive_datetime.t list -> t
val datetimeo : string -> Naive_datetime.t option list -> t
val datetime' : string -> Time_ns.t list -> t
val datetimeo' : string -> Time_ns.t option list -> t
val duration : string -> Duration.t list -> t
val durationo : string -> Duration.t option list -> t
val duration' : string -> Time_ns.Span.t list -> t
val durationo' : string -> Time_ns.Span.t option list -> t
val time : string -> Naive_time.t list -> t
val timeo : string -> Naive_time.t option list -> t
val time' : string -> Time_ns.Ofday.t list -> t
val timeo' : string -> Time_ns.Ofday.t option list -> t

val date_range
  :  ?every:string
  -> string
  -> start:Date.t
  -> stop:Date.t
  -> (t, string) result

val date_range_exn : ?every:string -> string -> start:Date.t -> stop:Date.t -> t

val datetime_range
  :  ?every:string
  -> string
  -> start:Naive_datetime.t
  -> stop:Naive_datetime.t
  -> (t, string) result

val datetime_range_exn
  :  ?every:string
  -> string
  -> start:Naive_datetime.t
  -> stop:Naive_datetime.t
  -> t

val datetime_range'
  :  ?every:string
  -> string
  -> start:Date.t
  -> stop:Date.t
  -> (t, string) result

val datetime_range_exn' : ?every:string -> string -> start:Date.t -> stop:Date.t -> t
val to_list : 'a Data_type.Typed.t -> t -> 'a list
val to_option_list : 'a Data_type.Typed.t -> t -> 'a option list
val get : 'a Data_type.Typed.t -> t -> int -> 'a option
val get_exn : 'a Data_type.Typed.t -> t -> int -> 'a

val map
  :  'a Data_type.Typed.t
  -> 'b Data_type.Typed.t
  -> t
  -> f:('a option -> 'b option)
  -> t

val cast : ?strict:bool -> t -> to_:Data_type.t -> t
val name : t -> string
val rename : t -> name:string -> unit
val dtype : t -> Data_type.t
val to_data_frame : t -> Data_frame0.t
val sort : ?descending:bool -> t -> t
val head : ?length:int -> t -> t
val tail : ?length:int -> t -> t
val estimated_size : t -> int

val sample_n
  :  ?seed:int
  -> t
  -> n:int
  -> with_replacement:bool
  -> shuffle:bool
  -> (t, string) result

val sample_n_exn : ?seed:int -> t -> n:int -> with_replacement:bool -> shuffle:bool -> t
val fill_null : t -> strategy:Fill_null_strategy.t -> (t, string) result
val fill_null_exn : t -> strategy:Fill_null_strategy.t -> t
val interpolate : t -> method_:[ `Linear | `Nearest ] -> (t, string) result
val interpolate_exn : t -> method_:[ `Linear | `Nearest ] -> t

(** Set this series to the empty series. This does not mutate any of the underlying
    chunks:
    If you constructed a dataframe using this series as an input or created a slice of
    the series, clearing this series will not clear the dataframe or clear the slice.

    Clearing the series will cause the series to lose access to its previous underlying
    chunks, which will decrement any reference counts to those chunks. *)
val clear : t -> unit

val to_string_hum : t -> string
val print : t -> unit
val pp : Format.formatter -> t -> unit [@@ocaml.toplevel_printer]

include Common.Compare with type t := t
include Common.Numeric with type t := t

module Expert : sig
  (** Edit the values of a chunk in a non-nullable series.

      This function will result in undesired behavior when applied to a series containing
      any null values -- use [modify_optional_series_at_chunk_index] to get proper null
      handling. *)
  val modify_at_chunk_index
    :  t
    -> dtype:'a Data_type.Typed.t
    -> chunk_index:int (** The index of the chunk to modify, 0-indexed. *)
    -> indices_and_values:(int * 'a) list
         (** A list of (index, value) tuples to set within the chunk. The index is 0-indexed
             and refers to an index within the chunk, not the entire series. Therefore, index
             should satisfy 0 <= index < chunk_length. *)
    -> (unit, string) result

  (** Edit the values of a chunk in a nullable series. This does not automatically update
      the null counts for the series; call [compute_null_count] after updating chunks. *)
  val modify_optional_at_chunk_index
    :  t
    -> dtype:'a Data_type.Typed.t
    -> chunk_index:int
    -> indices_and_values:(int * 'a option) list
    -> (unit, string) result

  (** Recompute the null counts for the series, if the cached null counts are stale. *)
  val compute_null_count : t -> int
end
