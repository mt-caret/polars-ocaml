open! Core

module T = struct
  type t =
    | Nanoseconds
    | Microseconds
    | Milliseconds
  [@@deriving compare, sexp, enumerate, quickcheck]
end

include T
include Sexpable.To_stringable (T)
