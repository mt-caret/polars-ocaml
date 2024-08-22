open! Base
open  Ppxlib

type t = private
  | Nested
  (** [@polars.nested] denotes whether a particular field type [Foo.t] should use the
      'lifted' [Foo.Data_frame.to_series], as opposed to the default [Foo.data_type]. *)
  | Default_data_type
  | Custom_data_type of expression
  (** [@polars.data_type] allows users to specify a custom [Polars.Data_type.t] for a
      field, as opposed to the default [Foo.data_type]. *)

val create : label_declaration -> t
val all : Attribute.packed list
