open! Core

module type S = sig
  type derived_on
  type t = private Polars.Data_frame.t

  val default_column_names : string list
  val to_series : derived_on array -> Polars.Series.t list
  val of_ts : derived_on list -> t
  val to_ts : t -> derived_on list

  (** [of_polars_data_frame ?remap data_frame] converts a [Polars.Data_frame.t] to a [t],
      ensuring that columns are a superset of fields.

      Column names are validated according to [remap], where [remap]'s keys correspond to
      the names of fields in the record and the values correspond to the name of the
      series to expect.

      If a field is not available in [remap], it will expect a column with the default
      field name. *)
  val of_polars_data_frame : ?remap:string String.Map.t -> Polars.Data_frame.t -> t

  val to_polars_data_frame : t -> Polars.Data_frame.t

  (** [get ?remap t ~index] attempts to find the [derived_on] value at [index].

      The field column names it searches for will be adjusted according to [remap]. See
      [of_polars_data_frame] for the semantics of [remap]. *)
  val get     : ?remap:string String.Map.t -> t -> index:int -> derived_on option

  (** [get_exn] is the same as {!val:get} except raises if the [index] is not in the data
      frame. *)
  val get_exn : ?remap:string String.Map.t -> t -> index:int -> derived_on
end

module type Polars_data_frame = sig
  module type S = S
end
