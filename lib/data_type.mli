open! Core

module Time_unit : sig
  type t =
    | Nanoseconds
    | Microseconds
    | Milliseconds
  [@@deriving sexp, enumerate]
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
[@@deriving sexp]

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
      | List : 'a t -> 'a list t

    type packed = T : 'a t -> packed

    val to_untyped : 'a t -> untyped
    val of_untyped : untyped -> packed option
  end
  with type untyped := t
