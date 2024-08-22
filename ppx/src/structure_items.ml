open! Base
open  Ppxlib
open  Ast_builder.Default

type t =
  { type_                : structure_item
  ; default_column_names : structure_item
  ; to_series            : structure_item
  ; of_ts                : structure_item
  ; to_ts                : structure_item
  ; of_polars_data_frame : structure_item
  ; to_polars_data_frame : structure_item
  ; get                  : structure_item
  ; get_exn              : structure_item
  }

let type_ loc =
  let manifest = [%type: Polars.Data_frame.t] in
  let type_declarations =
    [ type_declaration
        ~loc
        ~name:(Loc.make ~loc "t")
        ~params:[]
        ~cstrs:[]
        ~kind:Ptype_abstract
        ~private_:Public
        ~manifest:(Some manifest)
    ]
  in
  pstr_type ~loc Recursive type_declarations
;;

let column_names_for_nested_fields loc core_type field =
  let f longident ~type_name =
    let module_qualified_default_column_names =
      Common.data_frame_qualified_ident
        loc
        longident
        ~type_name
        ~value:"default_column_names"
    in
    (* TODO: perhaps we want to define a type like:
       {[
         type t =
           | Direct of { name: string }
           | Nested of { name: string; children: t list }
       ]}

       to make this encoding more explicit in the types. *)
    let field_qualified_name =
      [%expr
        [%e Ast_helpers.field_name_expression_string loc field]
        ^ "."
        ^ name_type_params_in_td]
    in
    [%expr
      Core.List.map
        [%e module_qualified_default_column_names]
        ~f:(fun name_type_params_in_td -> [%e field_qualified_name])]
  in
  Common.if_non_core_primitive_with_no_params_for_nested_fields loc core_type ~f
;;

let default_column_names_internal loc ~fields =
  let column_names_for_one_field field =
    let unqualified_field_name =
      [%expr [ [%e Ast_helpers.field_name_expression_string loc field] ]]
    in
    match Attributes.create field with
    | Nested            -> column_names_for_nested_fields loc field.pld_type field
    | Default_data_type -> unqualified_field_name
    | Custom_data_type (_ : expression) -> unqualified_field_name
  in
  let series = List.map fields ~f:column_names_for_one_field |> List.rev in
  match series with
  | x :: tl ->
    let list =
      List.fold
        tl
        ~init:[%expr [ [%e x] ]]
        ~f:(fun acc next -> [%expr [%e next] :: [%e acc]])
    in
    [%expr Core.List.concat [%e list]]
  | _ ->
    invalid_arg
      "Fields cannot be empty because it is parsed from a record. Was this a cosmic or \
       gamma ray?"
;;

let default_column_names loc ~fields =
  let pat  = [%pat? default_column_names]              in
  let expr = default_column_names_internal loc ~fields in
  Ast_helpers.value_binding_structure loc pat expr
;;

let make_field_name_plural field =
  { field with pld_name = { field.pld_name with txt = field.pld_name.txt ^ "s" } }
;;

let nested_series loc core_type data field =
  let f longident ~type_name =
    let module_qualified_to_series =
      Common.data_frame_qualified_ident loc longident ~type_name ~value:"to_series"
    in
    let field_qualified_name =
      (* TODO: nested series have to be appropriately renamed in order to
         guarantee no name clashes; it would be cool if the ppx would expose some more
         flexibility over this (i.e. ability to specify what each series name should
         be, the prefix for nested records, etc.) *)
      [%expr
        [%e Ast_helpers.field_name_expression_string loc field]
        ^ "."
        ^ name_type_params_in_td]
    in
    [%expr
      Core.List.map
        ([%e module_qualified_to_series] [%e data])
        ~f:(fun series ->
          let name_type_params_in_td = Polars.Series.name series in
          Polars.Series.rename series ~name:[%e field_qualified_name];
          series)]
  in
  Common.if_non_core_primitive_with_no_params_for_nested_fields loc core_type ~f
;;

let series loc ~fields =
  let create_one_series field =
    let data = Ast_helpers.field_name_expression loc (make_field_name_plural field) in
    let create' data_type =
      [%expr
        [ Polars.Series.create'
            [%e data_type]
            [%e Ast_helpers.field_name_expression_string loc field]
            [%e data]
        ]]
    in
    match Attributes.create field with
    | Nested            -> nested_series loc field.pld_type data field
    | Default_data_type ->
      let data_type =
        Data_type.create loc field.pld_type ~field_name:field.pld_name.txt
      in
      create' data_type
    | Custom_data_type expression -> create' expression
  in
  let series = List.map fields ~f:create_one_series |> List.rev in
  match series with
  | x :: tl ->
    let list =
      List.fold
        tl
        ~init:[%expr [ [%e x] ]]
        ~f:(fun acc next -> [%expr [%e next] :: [%e acc]])
    in
    [%expr Core.List.concat [%e list]]
  | _ ->
    invalid_arg
      "Fields cannot be empty because it is parsed from a record. Was this a cosmic or \
       gamma ray?"
;;

let create_arrays loc ~fields =
  Ast_helpers.value_bindings_exn loc fields (fun field ->
    let pat            = Ast_helpers.field_name_pattern loc (make_field_name_plural field) in
    let record_pattern = Ast_helpers.record_pattern loc [ field ]           in
    let field_name_expression = Ast_helpers.field_name_expression loc field in
    let expr =
      [%expr Core.Array.map ts ~f:(fun [%p record_pattern] -> [%e field_name_expression])]
    in
    value_binding ~loc ~pat ~expr)
;;

let to_series loc ~fields =
  let pat = [%pat? to_series] in
  let expr =
    let create_series = series        loc ~fields in
    let create_arrays = create_arrays loc ~fields in
    [%expr fun ts -> [%e create_arrays ~in_:create_series]]
  in
  Ast_helpers.value_binding_structure loc pat expr
;;

let of_ts loc =
  let pat = [%pat? of_ts] in
  let expr =
    [%expr
      fun ts ->
        let series = Core.Array.of_list ts |> to_series in
        Polars.Data_frame.create_exn series]
  in
  Ast_helpers.value_binding_structure loc pat expr
;;

let to_ts loc =
  let pat = [%pat? to_ts] in
  let expr =
    (* TODO: This is kind of tricky; it'll be nice if we can expose some way
       of using get_exn that assumes that nothing is null (and also be able to null-aware
       versions that give you [t option list] that is guaranteed to be as long as the
       dataframe has rows) *)
    [%expr
      fun t ->
        let length = Polars.Data_frame.height t in
        Core.List.init length ~f:(fun index -> get t ~index) |> Core.List.filter_opt]
  in
  Ast_helpers.value_binding_structure loc pat expr
;;

let of_polars_data_frame loc =
  let pat = [%pat? of_polars_data_frame] in
  let expr =
    (* TODO: Currently we only validate the series names, but we probably
       can also validate the data types of each series as well with [Series.dtype]. *)
    [%expr
      fun ?(remap = Core.String.Map.empty) df ->
        let expected_series_names =
          Core.List.map default_column_names ~f:(fun column_name ->
            Core.Map.find remap column_name |> Core.Option.value ~default:column_name)
          |> Core.String.Set.of_list
        in
        let series_names =
          Polars.Data_frame.get_column_names df |> Core.String.Set.of_list
        in
        let fields_not_found = Core.Set.diff expected_series_names series_names in
        (match Core.Set.is_empty fields_not_found with
         | true  -> ()
         | false ->
           let fields_not_found =
             Core.Set.to_list fields_not_found |> Core.String.concat ~sep:", "
           in
           invalid_arg
             ("Field(s) not found in series or in [remap] parameter: " ^ fields_not_found));
        df]
  in
  Ast_helpers.value_binding_structure loc pat expr
;;

let to_polars_data_frame loc =
  let pat  = [%pat? to_polars_data_frame] in
  let expr = [%expr fun t -> t]           in
  Ast_helpers.value_binding_structure loc pat expr
;;

let get_from_series loc data_type ~name =
  [%expr
    let name = Core.Map.find remap [%e name] |> Core.Option.value ~default:[%e name] in
    Polars.Series.get [%e data_type] (Polars.Data_frame.column_exn t ~name) index]
;;

(* TODO: Consider changing the semantics of [remap] to the following:

   [remap] for {[

     type t = {
       m0 : M0.t [@polars.nested]
     }
     [@@deriving polars]

   ]}

   should be [m0 -> some_other_name] rather than [m0.field_of_m0 -> some_other_name]. The
   latter is strictly more expressive, but forces users to think about field names in the
   nested data frame. *)

let get_from_nested_field loc field_type ~name =
  let module_qualified_data_frame_function_for_nested_fields ~function_name =
    Common.if_non_core_primitive_with_no_params_for_nested_fields
      loc
      field_type
      ~f:(fun longident ~type_name ->
        Common.data_frame_qualified_ident loc longident ~type_name ~value:function_name)
  in
  let get = module_qualified_data_frame_function_for_nested_fields ~function_name:"get" in
  let of_polars_data_frame =
    module_qualified_data_frame_function_for_nested_fields
      ~function_name:"of_polars_data_frame"
  in
  [%expr
    let column_names = Polars.Data_frame.get_column_names t in
    let columns =
      column_names @ Core.Map.keys remap
      |> Core.List.filter_map ~f:(fun field ->
        match String.lsplit2 field ~on:'.' with
        | Some (field_name, suffix) when Core.String.equal field_name [%e name] ->
          Some (field, suffix)
        | _ -> None)
      |> Core.List.dedup_and_sort
           ~compare:(Core.Comparable.lift Core.String.compare ~f:(fun (x, _) -> x))
    in
    let remap =
      Core.List.map columns ~f:(fun (field, suffix) ->
        suffix, Core.Map.find remap field |> Option.value ~default:field)
      |> Core.String.Map.of_alist_exn
    in
    [%e of_polars_data_frame] t ~remap |> [%e get] ~remap ~index]
;;

let get_all_fields_from_data_frame loc fields =
  Ast_helpers.value_bindings_exn loc fields (fun field ->
    let field_name = field.pld_name.txt                                 in
    let field_type = field.pld_type                                     in
    let name       = Ast_helpers.field_name_expression_string loc field in
    let pat        = Ast_helpers.field_name_pattern loc field           in
    let expr =
      match Attributes.create field with
      | Nested -> get_from_nested_field loc field_type ~name
      | Default_data_type ->
        let data_type = Data_type.create loc field_type ~field_name in
        get_from_series loc data_type ~name
      | Custom_data_type expression -> get_from_series loc expression ~name
    in
    value_binding ~loc ~pat ~expr)
;;

let match_on_fields loc fields =
  List.map fields ~f:(Ast_helpers.field_name_expression loc) |> pexp_tuple ~loc
;;

let pattern_of_some_fields loc fields =
  List.map fields ~f:(fun field ->
    let field_name = Ast_helpers.field_name_pattern loc field in
    [%pat? Some [%p field_name]])
  |> ppat_tuple ~loc
;;

let get loc ~fields =
  let pat = [%pat? get] in
  let match_ =
    [%expr
      match [%e match_on_fields loc fields] with
      | [%p pattern_of_some_fields loc fields] ->
        Some [%e Ast_helpers.record_expression loc fields]
      | _ -> None]
  in
  let expr =
    [%expr
      fun ?(remap = Core.String.Map.empty) t ~index ->
        [%e get_all_fields_from_data_frame loc fields ~in_:match_]]
  in
  Ast_helpers.value_binding_structure loc pat expr
;;

let get_exn loc =
  let pat = [%pat? get_exn] in
  let expr =
    [%expr
      fun ?remap t ~index ->
        Core.Option.value_exn
          (get ?remap t ~index)
          ~here:[%here]
          ~message:"index out of bounds or value is null"]
  in
  Ast_helpers.value_binding_structure loc pat expr
;;

let create loc ~fields =
  { type_                = type_                loc
  ; default_column_names = default_column_names loc ~fields
  ; to_series            = to_series            loc ~fields
  ; of_ts                = of_ts                loc
  ; to_ts                = to_ts                loc
  ; of_polars_data_frame = of_polars_data_frame loc
  ; to_polars_data_frame = to_polars_data_frame loc
  ; get                  = get                  loc ~fields
  ; get_exn              = get_exn              loc
  }
;;

let to_list
  { type_
  ; default_column_names
  ; to_series
  ; of_ts
  ; to_ts
  ; of_polars_data_frame
  ; to_polars_data_frame
  ; get
  ; get_exn
  }
  =
  [ type_
  ; default_column_names
  ; to_series
  ; of_polars_data_frame
  ; to_polars_data_frame
  ; get
  ; get_exn
  ; of_ts
  ; to_ts
  ]
;;
