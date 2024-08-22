open! Core

(* TODO: A lot of the tests here can be property tests without a lot of
   effort. *)

module M0 = struct
  type t =
    { f1 : int
    ; f2 : float
    ; f3 : string
    ; f4 : bool
    }
  [@@deriving sexp, stringable_record ~separator:'-'] [@@deriving_inline polars]

  include struct
    [@@@ocaml.warning "-60"]

    let _ = fun (_ : t) -> ()

    module Data_frame : Ppx_polars_runtime.Polars_data_frame.S with type derived_on := t =
    struct
      type t = Polars.Data_frame.t

      let default_column_names =
        Core.List.concat [ [ "f1" ]; [ "f2" ]; [ "f3" ]; [ "f4" ] ]
      ;;

      let _ = default_column_names

      let to_series ts =
        let f1s = Core.Array.map ts ~f:(fun { f1; _ } -> f1) in
        let f2s = Core.Array.map ts ~f:(fun { f2; _ } -> f2) in
        let f3s = Core.Array.map ts ~f:(fun { f3; _ } -> f3) in
        let f4s = Core.Array.map ts ~f:(fun { f4; _ } -> f4) in
        Core.List.concat
          [ [ Polars.Series.create' Polars.Data_type.Typed.Int64   "f1" f1s ]
          ; [ Polars.Series.create' Polars.Data_type.Typed.Float64 "f2" f2s ]
          ; [ Polars.Series.create' Polars.Data_type.Typed.Utf8    "f3" f3s ]
          ; [ Polars.Series.create' Polars.Data_type.Typed.Boolean "f4" f4s ]
          ]
      ;;

      let _ = to_series

      let of_polars_data_frame ?(remap = Core.String.Map.empty) df =
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
        df
      ;;

      let _                      = of_polars_data_frame
      let to_polars_data_frame t = t
      let _                      = to_polars_data_frame

      let get ?(remap = Core.String.Map.empty) t ~index =
        let f1 =
          let name = Core.Map.find remap "f1" |> Core.Option.value ~default:"f1" in
          Polars.Series.get
            Polars.Data_type.Typed.Int64
            (Polars.Data_frame.column_exn t ~name)
            index
        in
        let f2 =
          let name = Core.Map.find remap "f2" |> Core.Option.value ~default:"f2" in
          Polars.Series.get
            Polars.Data_type.Typed.Float64
            (Polars.Data_frame.column_exn t ~name)
            index
        in
        let f3 =
          let name = Core.Map.find remap "f3" |> Core.Option.value ~default:"f3" in
          Polars.Series.get
            Polars.Data_type.Typed.Utf8
            (Polars.Data_frame.column_exn t ~name)
            index
        in
        let f4 =
          let name = Core.Map.find remap "f4" |> Core.Option.value ~default:"f4" in
          Polars.Series.get
            Polars.Data_type.Typed.Boolean
            (Polars.Data_frame.column_exn t ~name)
            index
        in
        match f1, f2, f3, f4 with
        | Some f1, Some f2, Some f3, Some f4 -> Some { f1; f2; f3; f4 }
        | _ -> None
      ;;

      let _ = get

      let get_exn ?remap t ~index =
        Core.Option.value_exn
          (get ?remap t ~index)
          ~here:[%here]
          ~message:"index out of bounds or value is null"
      ;;

      let _ = get_exn

      let of_ts ts =
        let series = Core.Array.of_list ts |> to_series in
        Polars.Data_frame.create_exn series
      ;;

      let _ = of_ts

      let to_ts t =
        let length = Polars.Data_frame.height t in
        Core.List.init length ~f:(fun index -> get t ~index) |> Core.List.filter_opt
      ;;

      let _ = to_ts
    end
  end [@@ocaml.doc "@inline"]

  [@@@end]

  let data_type =
    Polars.Data_type.Typed.Custom
      { data_type = Utf8; f = of_string; f_inverse = to_string }
  ;;

  (* NOTE These are not in a [test_module] because their values are used in other tests.
  *)
  let t  = { f1 = 1; f2 = 1.; f3 = "1"; f4 = true  }
  let t' = { f1 = 2; f2 = 2.; f3 = "2"; f4 = false }

  let%expect_test _ =
    let df = Data_frame.of_ts [ t; t' ] |> Data_frame.to_polars_data_frame in
    Polars.Data_frame.print df;
    [%expect
      {|
      shape: (2, 4)
      ┌─────┬─────┬─────┬───────┐
      │ f1  ┆ f2  ┆ f3  ┆ f4    │
      │ --- ┆ --- ┆ --- ┆ ---   │
      │ i64 ┆ f64 ┆ str ┆ bool  │
      ╞═════╪═════╪═════╪═══════╡
      │ 1   ┆ 1.0 ┆ 1   ┆ true  │
      │ 2   ┆ 2.0 ┆ 2   ┆ false │
      └─────┴─────┴─────┴───────┘
      |}]
  ;;

  let%expect_test "get" =
    let t = Data_frame.of_ts [ t; t' ] |> Data_frame.get_exn ~index:0 in
    print_s [%message (t : t)];
    [%expect {| (t ((f1 1) (f2 1) (f3 1) (f4 true))) |}]
  ;;

  let%expect_test "get with remap" =
    let df = Data_frame.of_ts [ t; t' ] |> Data_frame.to_polars_data_frame in
    let polars_df =
      Polars.Data_frame.columns_exn df ~names:(Polars.Data_frame.get_column_names df)
      |> List.mapi ~f:(fun index series ->
        Polars.Series.rename series ~name:("g" ^ string_of_int (index + 1));
        series)
      |> Polars.Data_frame.create_exn
    in
    let remap =
      Map.of_alist_exn (module String) [ "f1", "g1"; "f2", "g2"; "f3", "g3"; "f4", "g4" ]
    in
    let df = Data_frame.of_polars_data_frame ~remap polars_df in
    let t  = Data_frame.get_exn df           ~remap ~index:0  in
    print_s [%message (t : t)];
    [%expect {| (t ((f1 1) (f2 1) (f3 1) (f4 true))) |}]
  ;;
end

module M0' = struct
  type t = M0.t =
    { f1 : int
    ; f2 : float
    ; f3 : string
    ; f4 : bool
    }
  [@@deriving_inline polars]

  include struct
    [@@@ocaml.warning "-60"]

    let _ = fun (_ : t) -> ()

    module Data_frame : Ppx_polars_runtime.Polars_data_frame.S with type derived_on := t =
    struct
      type t = Polars.Data_frame.t

      let default_column_names =
        Core.List.concat [ [ "f1" ]; [ "f2" ]; [ "f3" ]; [ "f4" ] ]
      ;;

      let _ = default_column_names

      let to_series ts =
        let f1s = Core.Array.map ts ~f:(fun { f1; _ } -> f1) in
        let f2s = Core.Array.map ts ~f:(fun { f2; _ } -> f2) in
        let f3s = Core.Array.map ts ~f:(fun { f3; _ } -> f3) in
        let f4s = Core.Array.map ts ~f:(fun { f4; _ } -> f4) in
        Core.List.concat
          [ [ Polars.Series.create' Polars.Data_type.Typed.Int64   "f1" f1s ]
          ; [ Polars.Series.create' Polars.Data_type.Typed.Float64 "f2" f2s ]
          ; [ Polars.Series.create' Polars.Data_type.Typed.Utf8    "f3" f3s ]
          ; [ Polars.Series.create' Polars.Data_type.Typed.Boolean "f4" f4s ]
          ]
      ;;

      let _ = to_series

      let of_polars_data_frame ?(remap = Core.String.Map.empty) df =
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
        df
      ;;

      let _                      = of_polars_data_frame
      let to_polars_data_frame t = t
      let _                      = to_polars_data_frame

      let get ?(remap = Core.String.Map.empty) t ~index =
        let f1 =
          let name = Core.Map.find remap "f1" |> Core.Option.value ~default:"f1" in
          Polars.Series.get
            Polars.Data_type.Typed.Int64
            (Polars.Data_frame.column_exn t ~name)
            index
        in
        let f2 =
          let name = Core.Map.find remap "f2" |> Core.Option.value ~default:"f2" in
          Polars.Series.get
            Polars.Data_type.Typed.Float64
            (Polars.Data_frame.column_exn t ~name)
            index
        in
        let f3 =
          let name = Core.Map.find remap "f3" |> Core.Option.value ~default:"f3" in
          Polars.Series.get
            Polars.Data_type.Typed.Utf8
            (Polars.Data_frame.column_exn t ~name)
            index
        in
        let f4 =
          let name = Core.Map.find remap "f4" |> Core.Option.value ~default:"f4" in
          Polars.Series.get
            Polars.Data_type.Typed.Boolean
            (Polars.Data_frame.column_exn t ~name)
            index
        in
        match f1, f2, f3, f4 with
        | Some f1, Some f2, Some f3, Some f4 -> Some { f1; f2; f3; f4 }
        | _ -> None
      ;;

      let _ = get

      let get_exn ?remap t ~index =
        Core.Option.value_exn
          (get ?remap t ~index)
          ~here:[%here]
          ~message:"index out of bounds or value is null"
      ;;

      let _ = get_exn

      let of_ts ts =
        let series = Core.Array.of_list ts |> to_series in
        Polars.Data_frame.create_exn series
      ;;

      let _ = of_ts

      let to_ts t =
        let length = Polars.Data_frame.height t in
        Core.List.init length ~f:(fun index -> get t ~index) |> Core.List.filter_opt
      ;;

      let _ = to_ts
    end
  end [@@ocaml.doc "@inline"]

  [@@@end]
end

module M1 : sig
  type t [@@deriving sexp]

  val create : int -> t
  val custom_data_type : t Polars.Data_type.Typed.t
end = struct
  type t = int [@@deriving sexp]

  let create           = Fn.id
  let custom_data_type = Polars.Data_type.Typed.Int64
end

module M2 : sig
  type t =
    | Polar
    | Beats
    | Bears
  [@@deriving sexp]

  val rank    : t -> int
  val of_rank : int -> t
end = struct
  type t =
    | Polar
    | Beats
    | Bears
  [@@deriving sexp]

  let rank = function
    | Polar -> 0
    | Beats -> 1
    | Bears -> 2
  ;;

  let of_rank = function
    | 0 -> Polar
    | 1 -> Beats
    | 2 -> Bears
    | _ -> failwith "rank was not 0, 1, or 2. Was this a cosmic or gamma ray?"
  ;;
end

module M3 = struct
  type s =
    { f1 : int
    ; f2 : M0.t
    ; f3 : M0.t [@polars.nested]
    ; f4 : M1.t [@polars.data_type M1.custom_data_type]
    ; f5 : M2.t
         [@polars.data_type
           Custom { data_type = UInt8; f = M2.of_rank; f_inverse = M2.rank }]
    }
  [@@deriving sexp] [@@deriving_inline polars]

  include struct
    [@@@ocaml.warning "-60"]

    let _ = fun (_ : s) -> ()

    module Data_frame_s :
      Ppx_polars_runtime.Polars_data_frame.S with type derived_on := s = struct
      type t = Polars.Data_frame.t

      let default_column_names =
        Core.List.concat
          [ [ "f1" ]
          ; [ "f2" ]
          ; Core.List.map
              M0.Data_frame.default_column_names
              ~f:(fun name_type_params_in_td -> "f3" ^ "." ^ name_type_params_in_td)
          ; [ "f4" ]
          ; [ "f5" ]
          ]
      ;;

      let _ = default_column_names

      let to_series ts =
        let f1s = Core.Array.map ts ~f:(fun { f1; _ } -> f1) in
        let f2s = Core.Array.map ts ~f:(fun { f2; _ } -> f2) in
        let f3s = Core.Array.map ts ~f:(fun { f3; _ } -> f3) in
        let f4s = Core.Array.map ts ~f:(fun { f4; _ } -> f4) in
        let f5s = Core.Array.map ts ~f:(fun { f5; _ } -> f5) in
        Core.List.concat
          [ [ Polars.Series.create' Polars.Data_type.Typed.Int64 "f1" f1s ]
          ; [ Polars.Series.create' M0.data_type "f2" f2s ]
          ; Core.List.map (M0.Data_frame.to_series f3s) ~f:(fun series ->
              let name_type_params_in_td = Polars.Series.name series in
              Polars.Series.rename series ~name:("f3" ^ "." ^ name_type_params_in_td);
              series)
          ; [ Polars.Series.create' M1.custom_data_type "f4" f4s ]
          ; [ Polars.Series.create'
                (Custom { data_type = UInt8; f = M2.of_rank; f_inverse = M2.rank })
                "f5"
                f5s
            ]
          ]
      ;;

      let _ = to_series

      let of_polars_data_frame ?(remap = Core.String.Map.empty) df =
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
        df
      ;;

      let _                      = of_polars_data_frame
      let to_polars_data_frame t = t
      let _                      = to_polars_data_frame

      let get ?(remap = Core.String.Map.empty) t ~index =
        let f1 =
          let name = Core.Map.find remap "f1" |> Core.Option.value ~default:"f1" in
          Polars.Series.get
            Polars.Data_type.Typed.Int64
            (Polars.Data_frame.column_exn t ~name)
            index
        in
        let f2 =
          let name = Core.Map.find remap "f2" |> Core.Option.value ~default:"f2" in
          Polars.Series.get M0.data_type (Polars.Data_frame.column_exn t ~name) index
        in
        let f3 =
          let column_names = Polars.Data_frame.get_column_names t in
          let columns =
            column_names @ Core.Map.keys remap
            |> Core.List.filter_map ~f:(fun field ->
              match String.lsplit2 field ~on:'.' with
              | Some (field_name, suffix) when Core.String.equal field_name "f3" ->
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
          M0.Data_frame.of_polars_data_frame t ~remap |> M0.Data_frame.get ~remap ~index
        in
        let f4 =
          let name = Core.Map.find remap "f4" |> Core.Option.value ~default:"f4" in
          Polars.Series.get
            M1.custom_data_type
            (Polars.Data_frame.column_exn t ~name)
            index
        in
        let f5 =
          let name = Core.Map.find remap "f5" |> Core.Option.value ~default:"f5" in
          Polars.Series.get
            (Custom { data_type = UInt8; f = M2.of_rank; f_inverse = M2.rank })
            (Polars.Data_frame.column_exn t ~name)
            index
        in
        match f1, f2, f3, f4, f5 with
        | Some f1, Some f2, Some f3, Some f4, Some f5 -> Some { f1; f2; f3; f4; f5 }
        | _ -> None
      ;;

      let _ = get

      let get_exn ?remap t ~index =
        Core.Option.value_exn
          (get ?remap t ~index)
          ~here:[%here]
          ~message:"index out of bounds or value is null"
      ;;

      let _ = get_exn

      let of_ts ts =
        let series = Core.Array.of_list ts |> to_series in
        Polars.Data_frame.create_exn series
      ;;

      let _ = of_ts

      let to_ts t =
        let length = Polars.Data_frame.height t in
        Core.List.init length ~f:(fun index -> get t ~index) |> Core.List.filter_opt
      ;;

      let _ = to_ts
    end
  end [@@ocaml.doc "@inline"]

  [@@@end]

  let%test_module _ =
    (module struct
      let df =
        Data_frame_s.of_ts
          [ { f1 = 1; f2 = M0.t;  f3 = M0.t;  f4 = M1.create 1; f5 = Polar }
          ; { f1 = 1; f2 = M0.t'; f3 = M0.t'; f4 = M1.create 1; f5 = Bears }
          ]
      ;;

      let%expect_test "of_ts" =
        let df = Data_frame_s.to_polars_data_frame df in
        Polars.Data_frame.print df;
        [%expect
          {|
          shape: (2, 8)
          ┌─────┬──────────────┬───────┬───────┬───────┬───────┬─────┬─────┐
          │ f1  ┆ f2           ┆ f3.f1 ┆ f3.f2 ┆ f3.f3 ┆ f3.f4 ┆ f4  ┆ f5  │
          │ --- ┆ ---          ┆ ---   ┆ ---   ┆ ---   ┆ ---   ┆ --- ┆ --- │
          │ i64 ┆ str          ┆ i64   ┆ f64   ┆ str   ┆ bool  ┆ i64 ┆ u8  │
          ╞═════╪══════════════╪═══════╪═══════╪═══════╪═══════╪═════╪═════╡
          │ 1   ┆ 1-1.-1-true  ┆ 1     ┆ 1.0   ┆ 1     ┆ true  ┆ 1   ┆ 0   │
          │ 1   ┆ 2-2.-2-false ┆ 2     ┆ 2.0   ┆ 2     ┆ false ┆ 1   ┆ 2   │
          └─────┴──────────────┴───────┴───────┴───────┴───────┴─────┴─────┘
          |}]
      ;;

      let%expect_test "get" =
        let s = Data_frame_s.get_exn df ~index:1 in
        Core.print_s [%message (s : s)];
        [%expect
          {|
          (s
           ((f1 1) (f2 ((f1 2) (f2 2) (f3 2) (f4 false)))
            (f3 ((f1 2) (f2 2) (f3 2) (f4 false))) (f4 1) (f5 Bears)))
          |}]
      ;;

      let%expect_test "get with remap" =
        let df = Data_frame_s.to_polars_data_frame df in
        let polars_df =
          Polars.Data_frame.columns_exn df ~names:(Polars.Data_frame.get_column_names df)
          |> List.map ~f:(fun series ->
            match Polars.Series.name series with
            | "f3.f1" ->
              Polars.Series.rename series ~name:"Hello123";
              series
            | _ -> series)
          |> Polars.Data_frame.create_exn
        in
        let remap = Map.of_alist_exn (module String) [ "f3.f1", "Hello123" ] in
        let df    = Data_frame_s.of_polars_data_frame ~remap polars_df       in
        let s     = Data_frame_s.get_exn df ~remap ~index:0                  in
        print_s [%message (s : s)];
        [%expect
          {|
          (s
           ((f1 1) (f2 ((f1 1) (f2 1) (f3 1) (f4 true)))
            (f3 ((f1 1) (f2 1) (f3 1) (f4 true))) (f4 1) (f5 Polar)))
          |}]
      ;;
    end)
  ;;
end

module M4 = struct
  type t =
    { f1 : int  list
    ; f2 : M0.t list
    }
  [@@deriving sexp] [@@deriving_inline polars]

  include struct
    [@@@ocaml.warning "-60"]

    let _ = fun (_ : t) -> ()

    module Data_frame : Ppx_polars_runtime.Polars_data_frame.S with type derived_on := t =
    struct
      type t = Polars.Data_frame.t

      let default_column_names = Core.List.concat [ [ "f1" ]; [ "f2" ] ]
      let _                    = default_column_names

      let to_series ts =
        let f1s = Core.Array.map ts ~f:(fun { f1; _ } -> f1) in
        let f2s = Core.Array.map ts ~f:(fun { f2; _ } -> f2) in
        Core.List.concat
          [ [ Polars.Series.create'
                (Polars.Data_type.Typed.List Polars.Data_type.Typed.Int64)
                "f1"
                f1s
            ]
          ; [ Polars.Series.create' (Polars.Data_type.Typed.List M0.data_type) "f2" f2s ]
          ]
      ;;

      let _ = to_series

      let of_polars_data_frame ?(remap = Core.String.Map.empty) df =
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
        df
      ;;

      let _                      = of_polars_data_frame
      let to_polars_data_frame t = t
      let _                      = to_polars_data_frame

      let get ?(remap = Core.String.Map.empty) t ~index =
        let f1 =
          let name = Core.Map.find remap "f1" |> Core.Option.value ~default:"f1" in
          Polars.Series.get
            (Polars.Data_type.Typed.List Polars.Data_type.Typed.Int64)
            (Polars.Data_frame.column_exn t ~name)
            index
        in
        let f2 =
          let name = Core.Map.find remap "f2" |> Core.Option.value ~default:"f2" in
          Polars.Series.get
            (Polars.Data_type.Typed.List  M0.data_type)
            (Polars.Data_frame.column_exn t ~name     )
            index
        in
        match f1, f2 with
        | Some f1, Some f2 -> Some { f1; f2 }
        | _                -> None
      ;;

      let _ = get

      let get_exn ?remap t ~index =
        Core.Option.value_exn
          (get ?remap t ~index)
          ~here:[%here]
          ~message:"index out of bounds or value is null"
      ;;

      let _ = get_exn

      let of_ts ts =
        let series = Core.Array.of_list ts |> to_series in
        Polars.Data_frame.create_exn series
      ;;

      let _ = of_ts

      let to_ts t =
        let length = Polars.Data_frame.height t in
        Core.List.init length ~f:(fun index -> get t ~index) |> Core.List.filter_opt
      ;;

      let _ = to_ts
    end
  end [@@ocaml.doc "@inline"]

  [@@@end]

  let%test_module _ =
    (module struct
      let df = Data_frame.of_ts [ { f1 = [ 1; 2; 3 ]; f2 = [ M0.t; M0.t' ] } ]

      let%expect_test _ =
        let df = Data_frame.to_polars_data_frame df in
        Polars.Data_frame.print df;
        [%expect
          {|
          shape: (1, 2)
          ┌───────────┬─────────────────────────────────┐
          │ f1        ┆ f2                              │
          │ ---       ┆ ---                             │
          │ list[i64] ┆ list[str]                       │
          ╞═══════════╪═════════════════════════════════╡
          │ [1, 2, 3] ┆ ["1-1.-1-true", "2-2.-2-false"] │
          └───────────┴─────────────────────────────────┘
          |}]
      ;;

      let%expect_test "get" =
        let t = Data_frame.get_exn df ~index:0 in
        Core.print_s [%message (t : t)];
        [%expect
          {|
          (t
           ((f1 (1 2 3))
            (f2 (((f1 1) (f2 1) (f3 1) (f4 true)) ((f1 2) (f2 2) (f3 2) (f4 false))))))
          |}]
      ;;
    end)
  ;;
end
