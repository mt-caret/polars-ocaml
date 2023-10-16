open! Core

module Time_unit : sig
  type t =
    | Nanoseconds
    | Microseconds
    | Milliseconds
  [@@deriving compare, sexp, enumerate, quickcheck]
end

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
  | Datetime of Time_unit.t * string option
  | Duration of Time_unit.t
  | Time
  | List of t
  | Null
  | Struct of (string * t) list
  | Unknown
[@@deriving compare, sexp, quickcheck]

include Stringable.S with type t := t

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
      | Date : Common.Naive_date.t t
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
  end
  with type untyped := t
