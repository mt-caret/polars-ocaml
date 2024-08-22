open! Base
open  Ppxlib

val module_name : type_name:label -> label

(** [data_frame_qualified_ident] returns [longident.(module_name ~type_name).value]. *)
val data_frame_qualified_ident
  :  location
  -> longident
  -> type_name:label
  -> value:label
  -> expression

(** [if_non_core_primitive_with_no_params_for_nested_fields] will raise if [core_type] is
    a core primitive or has type parameters. *)
val if_non_core_primitive_with_no_params_for_nested_fields
  :  location
  -> core_type
  -> f:(longident -> type_name:label -> expression)
  -> expression

val raise_unsupported : location -> why:label -> _
