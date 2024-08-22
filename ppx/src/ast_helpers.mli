open! Base
open  Ppxlib

(** [sequence_exn] raises on an empty list. *)
val sequence_exn : location -> 'a list -> ('a -> expression) -> expression

(** [value_bindings_exn] returns a non-recursive series of value bindings.

    It raises on an empty list. *)
val value_bindings_exn
  :  location
  -> 'a list
  -> ('a -> value_binding)
  -> in_:expression
  -> expression

(** [value_binding_structure] returns a non-recursive value binding as a [structure_item]. *)
val value_binding_structure : location -> pattern -> expression -> structure_item

(** {1 Records} *)

val record_pattern          : location -> label_declaration list -> pattern
val record_expression       : location -> label_declaration list -> expression
val field_name_pattern      : location -> label_declaration -> pattern
val field_name_expression   : location -> label_declaration -> expression
val field_name_expression_string : location -> label_declaration -> expression
