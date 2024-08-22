open! Base
open  Ppxlib
open  Ast_builder.Default

let rec create loc core_type ~field_name =
  match core_type.ptyp_desc with
  | Ptyp_constr (longident_loc, core_types) ->
    let name = Longident.name longident_loc.txt in
    (match core_types with
     | hd :: _ ->
       (match name with
        | "list" -> [%expr Polars.Data_type.Typed.List [%e create loc hd ~field_name]]
        | _      ->
          Common.raise_unsupported
            loc
            ~why:"no field types with arity > 0 allowed unless it's a [_ list]")
     | [] ->
       (match name with
        | "int"    -> [%expr Polars.Data_type.Typed.Int64]
        | "float"  -> [%expr Polars.Data_type.Typed.Float64]
        | "string" -> [%expr Polars.Data_type.Typed.Utf8]
        | "bool"   -> [%expr Polars.Data_type.Typed.Boolean]
        | _        -> type_constr_conv longident_loc ~loc ~f:(fun (_ : label) -> "data_type") []))
  | _ ->
    Common.raise_unsupported
      loc
      ~why:
        ("cannot infer data type for field "
         ^ field_name
         ^ " - please use [@polars.data_type _]")
;;
