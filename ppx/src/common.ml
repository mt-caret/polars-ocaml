open! Base
open  Ppxlib
open  Ast_builder.Default

let module_name ~type_name =
  let suffix =
    match type_name with
    | "t" -> ""
    | s   -> "_" ^ s
  in
  "Data_frame" ^ suffix
;;

let data_frame_qualified_ident loc longident ~type_name ~value =
  pexp_ident ~loc (Loc.make ~loc (Ldot (Ldot (longident, module_name ~type_name), value)))
;;

let raise_unsupported loc ~why =
  Location.raise_errorf ~loc "Unsupported use of `polars' (%s)." why
;;

let raise_only_non_core_primitives loc =
  raise_unsupported
    loc
    ~why:
      "you can only use the [@polars.nested] attribute on non-core-primitive type \
       constructors (with no type parameters)"
;;

let if_non_core_primitive_with_no_params_for_nested_fields loc core_type ~f =
  match core_type.ptyp_desc with
  | Ptyp_constr (longident_loc, []) ->
    (match longident_loc.txt with
     | Ldot (longident, type_name) -> f longident ~type_name
     | _ -> raise_only_non_core_primitives loc)
  | _ -> raise_only_non_core_primitives loc
;;
