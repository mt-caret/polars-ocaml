open! Core

type t =
  | Boolean
  | UInt8
  | UInt16
  | UInt32
  | UInt64
  | Int8
  | Int16
  | Int32
  | Int64
  | Float32
  | Float64
  | Utf8
  | Binary
  | Date
  | Datetime of Time_unit.t * Tz.t option
  | Duration of Time_unit.t
  | Time
  | List of t
  | Null
  | Categorical of Rev_mapping.t option
  | Struct of (string * t) list
  | Unknown
[@@deriving compare, sexp_of, quickcheck]

val to_string : t -> string

module Typed : sig
    type untyped

    type _ t =
      | Boolean : bool t
      | UInt8 : int t
      | UInt16 : int t
      | UInt32 : int t
      | UInt64 : int t
      | Int8 : int t
      | Int16 : int t
      | Int32 : int t
      | Int64 : int t
      | Float32 : float t
      | Float64 : float t
      | Utf8 : string t
      | Binary : string t
      | Date : Naive_date.t t
      | Datetime : Time_unit.t * Tz.t option -> Naive_datetime.t t
      | Duration : Time_unit.t -> Duration.t t
      | Time : Naive_time.t t
      | List : 'a t -> 'a list t
      | Custom :
          { data_type : 'a t
          ; f : 'a -> 'b
          ; f_inverse : 'b -> 'a
          }
          -> 'b t

    (** [strict_type_equal] returns type equality only if the two arguments
        correspond to the same exact branch. *)
    val strict_type_equal : 'a t -> 'b t -> ('a, 'b) Type_equal.t option

    (** [flatten_custom] extracts out any internal instances of the [Custom _]
        variant to the outermost point e.g.
        [List (Custom { data_type = Boolean; ... })] to
        [Custom { data_type = List Boolean; ... }] . *)
    val flatten_custom : 'a t -> 'a t

    type packed = T : 'a t -> packed [@@deriving compare, sexp_of, quickcheck]

    val to_untyped : 'a t -> untyped
    val of_untyped : untyped -> packed option

    module Core : sig
      val date : Date.t t
      val time : Time_ns.t t
      val span : Time_ns.Span.t t
      val ofday : Time_ns.Ofday.t t
    end
  end
  with type untyped := t
