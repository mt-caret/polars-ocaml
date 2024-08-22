open! Base
open  Ppxlib
open  Ast_builder.Default

let constraint_ loc ~derived_on =
  Pwith_typesubst
    ( Loc.make (Lident "derived_on") ~loc
    , type_declaration
        ~loc
        ~name:(Loc.make ~loc "derived_on")
        ~params:[]
        ~cstrs:[]
        ~kind:Ptype_abstract
        ~private_:Public
        ~manifest:(Some (ptyp_constr ~loc (Loc.make (Lident derived_on) ~loc) [])) )
;;

let create loc ~derived_on =
  let sig_name = Ldot (Ldot (Lident "Ppx_polars_runtime", "Polars_data_frame"), "S") in
  let sig_     = pmty_ident ~loc (Loc.make sig_name ~loc)                            in
  pmty_with sig_ [ constraint_ loc ~derived_on ] ~loc
;;
