open! Core

module Time_unit = struct
  module T = struct
    type t =
      | Nanoseconds
      | Microseconds
      | Milliseconds
    [@@deriving sexp, enumerate]
  end

  include T
  include Sexpable.To_stringable (T)
end

module T = struct
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
end

include T
include Sexpable.To_stringable (T)

module Typed = struct
  type untyped = t

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

  type packed = T : _ t -> packed

  let rec to_untyped : type a. a t -> untyped = function
    | Boolean -> Boolean
    | UInt8 -> UInt8
    | UInt16 -> UInt16
    | UInt32 -> UInt32
    | UInt64 -> UInt64
    | Int8 -> Int8
    | Int16 -> Int16
    | Int32 -> Int32
    | Int64 -> Int64
    | Float32 -> Float32
    | Float64 -> Float64
    | Utf8 -> Utf8
    | Binary -> Binary
    | List t -> List (to_untyped t)
  ;;

  let rec of_untyped : untyped -> packed option = function
    | Boolean -> Some (T Boolean)
    | UInt8 -> Some (T UInt8)
    | UInt16 -> Some (T UInt16)
    | UInt32 -> Some (T UInt32)
    | UInt64 -> Some (T UInt64)
    | Int8 -> Some (T Int8)
    | Int16 -> Some (T Int16)
    | Int32 -> Some (T Int32)
    | Int64 -> Some (T Int64)
    | Float32 -> Some (T Float32)
    | Float64 -> Some (T Float64)
    | Utf8 -> Some (T Utf8)
    | Binary -> Some (T Binary)
    | List t -> of_untyped t |> Option.map ~f:(fun (T t) -> T (List t))
    | Date | Datetime _ | Duration _ | Time | Null | Struct _ | Unknown -> None
  ;;
end
