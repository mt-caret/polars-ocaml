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

let quickcheck_generator =
  Quickcheck.Generator.filter quickcheck_generator ~f:(function
    | Backward (Some n) | Forward (Some n) ->
      (* n is u32 under the hood *)
      n >= 0 && n < Int.pow 2 32
    | Backward None | Forward None | Mean | Min | Max | Zero | One | Max_bound | Min_bound
      -> true)
;;

module For_testing = struct
  external to_rust_and_back : t -> t = "rust_test_fill_null_strategy"
end
