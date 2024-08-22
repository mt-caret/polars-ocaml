open! Core

module M0 : sig
  type t =
    { f1 : int
    ; f2 : float
    }
  [@@deriving polars]
end

module M1 : sig
  type t = { m0 : M0.t } [@@deriving polars]
end

module M2 : sig
  type t = { m1 : M1.t } [@@deriving polars]
end
