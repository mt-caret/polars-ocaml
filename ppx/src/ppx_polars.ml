open! Base
open  Ppxlib
open  Ast_builder.Default

let get_exactly_one_type loc = function
  | [ typedef ] -> typedef
  | _ -> Common.raise_unsupported loc ~why:"you cannot use it on more than one type"
;;

let get_exactly_one_type_without_params loc typedefs =
  match get_exactly_one_type loc typedefs with
  | { ptype_params = []; _ } as typedef -> typedef
  | { ptype_params = _ :: _; _ }        ->
    Common.raise_unsupported loc ~why:"you cannot use it with type parameters"
;;

let data_frame_module_sig loc typedef =
  let derived_on = typedef.ptype_name.txt                   in
  let name       = Common.module_name ~type_name:derived_on in
  let type_ =
    match typedef.ptype_kind with
    | Ptype_record _ | Ptype_abstract -> Signature_items.create loc ~derived_on
    | _ ->
      Common.raise_unsupported loc ~why:"you can only use it on records or abstract types"
  in
  psig_module (module_declaration ~loc ~name:(Loc.make ~loc (Some name)) ~type_) ~loc
;;

let sig_generator =
  let generate ~loc ~path:(_ : label) ((_ : rec_flag), typedefs) =
    let typedef = get_exactly_one_type_without_params loc typedefs in
    [ data_frame_module_sig loc typedef ]
  in
  Deriving.Generator.make_noarg generate ~attributes:[]
;;

let data_frame_module_str loc typedef =
  let derived_on = typedef.ptype_name.txt in
  let expr =
    match typedef.ptype_kind with
    | Ptype_record fields ->
      let expr =
        Structure_items.create loc ~fields
        |> Structure_items.to_list
        |> pmod_structure ~loc
      in
      let type_ = Signature_items.create loc ~derived_on in
      pmod_constraint ~loc expr type_
    | _ -> Common.raise_unsupported loc ~why:"you can only use it on records"
  in
  let name = Common.module_name ~type_name:typedef.ptype_name.txt in
  pstr_module ~loc (module_binding ~loc ~name:(Loc.make ~loc (Some name)) ~expr)
;;

let struct_generator =
  let generate ~loc ~path:(_ : label) ((_ : rec_flag), typedefs) =
    let typedef = get_exactly_one_type_without_params loc typedefs in
    [ data_frame_module_str loc typedef ]
  in
  Deriving.Generator.make_noarg generate ~attributes:Attributes.all
;;

let polars =
  Deriving.add "polars" ~sig_type_decl:sig_generator ~str_type_decl:struct_generator
;;
