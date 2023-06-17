open! Core

type t =
  | Backward of int option
  | Forward of int option
  | Mean
  | Min
  | Max
  | Zero
  | One
  | Max_bound
  | Min_bound
[@@deriving quickcheck, compare, sexp]

module For_testing : sig
  val to_rust_and_back : t -> t
end
