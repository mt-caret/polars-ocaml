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
