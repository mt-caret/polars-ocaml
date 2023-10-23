open! Core

type t =
  | Nanoseconds
  | Microseconds
  | Milliseconds
[@@deriving compare, sexp, enumerate, quickcheck]
