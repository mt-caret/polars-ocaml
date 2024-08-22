open! Core

module M0 : sig
  type t =
    { f1 : int
    ; f2 : float
    ; f3 : string
    ; f4 : bool
    }
  [@@deriving_inline polars]

  include sig
    [@@@ocaml.warning "-32-60"]

    module Data_frame : Ppx_polars_runtime.Polars_data_frame.S with type derived_on := t
  end
  [@@ocaml.doc "@inline"]

  [@@@end]
end

module M0' : sig
  type t = M0.t =
    { f1 : int
    ; f2 : float
    ; f3 : string
    ; f4 : bool
    }
  [@@deriving_inline polars]

  include sig
    [@@@ocaml.warning "-32-60"]

    module Data_frame : Ppx_polars_runtime.Polars_data_frame.S with type derived_on := t
  end
  [@@ocaml.doc "@inline"]

  [@@@end]
end

module M1 : sig
  type t [@@deriving sexp]

  val custom_data_type : t Polars.Data_type.Typed.t
end

module M2 : sig
  type t =
    | Polar
    | Beats
    | Bears
  [@@deriving sexp]
end

module M3 : sig
  type s =
    { f1 : int
    ; f2 : M0.t
    ; f3 : M0.t
    ; f4 : M1.t
    ; f5 : M2.t
    }
  [@@deriving_inline polars]

  include sig
    [@@@ocaml.warning "-32-60"]

    module Data_frame_s : Ppx_polars_runtime.Polars_data_frame.S with type derived_on := s
  end
  [@@ocaml.doc "@inline"]

  [@@@end]
end

module M4 : sig
  type t =
    { f1 : int  list
    ; f2 : M0.t list
    }
  [@@deriving_inline polars]

  include sig
    [@@@ocaml.warning "-32-60"]

    module Data_frame : Ppx_polars_runtime.Polars_data_frame.S with type derived_on := t
  end
  [@@ocaml.doc "@inline"]

  [@@@end]
end
