open! Core

type t =
  | Left
  | Inner
  | Outer
  | As_of of
      { strategy : [ `Backward | `Forward | `Nearest ]
      ; tolerance : string option
      ; left_by : string list option
      ; right_by : string list option
      }
  | Cross
  | Semi
  | Anti
[@@deriving compare, sexp]
