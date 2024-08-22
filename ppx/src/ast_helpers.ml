open! Base
open  Ppxlib
open  Ast_builder.Default

let rec sequence_exn loc list f =
  match list with
  | [ x ]   -> f x
  | x :: tl -> pexp_sequence ~loc (f x) (sequence_exn loc tl f)
  | []      ->
    invalid_arg "Cannot construct a sequence without expressions - input list is empty."
;;

let rec value_bindings_exn loc list f ~in_ =
  match list with
  | [ x ]   -> pexp_let ~loc Nonrecursive [ f x ] in_
  | x :: tl -> pexp_let ~loc Nonrecursive [ f x ] (value_bindings_exn loc tl f ~in_)
  | []      ->
    invalid_arg
      "Cannot construct a let binding without expressions - input list is empty."
;;

let value_binding_structure loc pat expr =
  pstr_value ~loc Nonrecursive [ value_binding ~loc ~pat ~expr ]
;;

let record_pattern loc fields =
  ppat_record
    ~loc
    (List.map fields ~f:(fun field ->
       Loc.make ~loc (Longident.Lident field.pld_name.txt), ppat_var ~loc field.pld_name))
    Open
;;

let record_expression loc fields =
  pexp_record
    ~loc
    (List.map fields ~f:(fun field ->
       let longident = Loc.make ~loc (Longident.Lident field.pld_name.txt) in
       longident, pexp_ident ~loc longident))
    None
;;

let field_name_pattern loc field = ppat_var ~loc field.pld_name

let field_name_expression loc field =
  pexp_ident ~loc (Loc.make ~loc (Longident.Lident field.pld_name.txt))
;;

let field_name_expression_string loc field =
  pexp_constant ~loc (Pconst_string (field.pld_name.txt, loc, None))
;;
