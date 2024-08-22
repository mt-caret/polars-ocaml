open! Base
open  Ppxlib

type t =
  | Nested
  | Default_data_type
  | Custom_data_type of expression

let nested =
  Attribute.declare
    "polars.nested"
    Attribute.Context.label_declaration
    Ast_pattern.(pstr nil)
    ()
;;

let data_type =
  Attribute.declare
    "polars.data_type"
    Attribute.Context.label_declaration
    Ast_pattern.(pstr (pstr_eval __ nil ^:: nil))
    Fn.id
;;

let create label_declaration =
  let get attribute    = Attribute.get attribute label_declaration in
  let nested           = get nested                                in
  let custom_data_type = get data_type                             in
  match nested, custom_data_type with
  | None   , None            -> Default_data_type
  | Some (), None            -> Nested
  | None   , Some expression -> Custom_data_type expression
  | Some _ , Some _          ->
    Common.raise_unsupported
      label_declaration.pld_loc
      ~why:
        "You cannot use the [@polars.nested] and [@polars.data_type] attributes in the \
         same field."
;;

let all = [ Attribute.T nested; Attribute.T data_type ]
