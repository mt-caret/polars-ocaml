(** [Structure_items] contains the structure items in the [Data_frame] module. *)

open! Base
open  Ppxlib

(* TODO: Support arbitrary queries with [get]. *)

type t = private
  { type_                : structure_item (** [type t = private Data_frame.t] *)
  ; default_column_names : structure_item (** [string list] *)
  ; to_series            : structure_item (** [derived_on array -> Series.t list] *)
  ; of_ts                : structure_item (** [derived_on list -> t] *)
  ; to_ts                : structure_item (** [t -> derived_on list] *)
  ; of_polars_data_frame : structure_item (** [Polars.Data_frame.t -> t] *)
  ; to_polars_data_frame : structure_item (** [t -> Polars.Data_frame.t] *)
  ; get                  : structure_item (** [t -> index:int -> derived_on option] *)
  ; get_exn              : structure_item (** [t -> index:int -> derived_on ] *)
  }

val create : location -> fields:label_declaration list -> t
val to_list : t -> structure_item list
