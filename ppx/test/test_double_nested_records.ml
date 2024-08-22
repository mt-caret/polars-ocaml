open! Core

module M0 = struct
  type t =
    { f1 : int
    ; f2 : float
    }
  [@@deriving polars, sexp_of]

  let t = { f1 = 1; f2 = 1. }
end

module M1 = struct
  type t = { m0 : M0.t [@polars.nested] } [@@deriving polars, sexp_of]

  let t = { m0 = M0.t }
end

module M2 = struct
  type t = { m1 : M1.t [@polars.nested] } [@@deriving polars, sexp_of]

  let t = { m1 = M1.t }

  let%expect_test "doubly nested get" =
    let t = Data_frame.of_ts [ t ] |> Data_frame.get_exn ~index:0 in
    Core.print_s [%message (t : t)];
    [%expect {| (t ((m1 ((m0 ((f1 1) (f2 1))))))) |}]
  ;;
end
