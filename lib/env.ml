open! Core

module Table_formatting = struct
  type t =
    | Ascii_full
    | Ascii_full_condensed
    | Ascii_no_borders
    | Ascii_borders_only
    | Ascii_borders_only_condensed
    | Ascii_horizontal_only
    | Ascii_markdown
    | Utf8_full
    | Utf8_full_condensed
    | Utf8_no_borders
    | Utf8_borders_only
    | Utf8_horizontal_only
    | Nothing
  [@@deriving variants, enumerate]

  let to_string t = Variants.to_name t |> String.uppercase

  let of_string =
    let variants = List.map all ~f:(fun t -> to_string t, t) in
    fun s -> List.find_exn variants ~f:(fun (name, _) -> String.equal name s) |> snd
  ;;
end

type t =
  { fmt_table_formatting : Table_formatting.t
  ; table_width : int
  }
[@@deriving typed_fields]

module Info = struct
  type 'a t =
    { env_var : string
    ; of_string : string -> 'a
    ; to_string : 'a -> string
    ; default : 'a
    }
  [@@deriving fields]
end

let info : type a. a Typed_field.t -> a Info.t = function
  | Fmt_table_formatting ->
    { env_var = "POLARS_FMT_TABLE_FORMATTING"
    ; of_string = Table_formatting.of_string
    ; to_string = Table_formatting.to_string
    ; default = Utf8_full_condensed
    }
  | Table_width ->
    { env_var = "POLARS_TABLE_WIDTH"
    ; of_string = Int.of_string
    ; to_string = Int.to_string
    ; default = 100
    }
;;

let set : type a. ?value:a -> a Typed_field.t -> unit =
  fun ?value field ->
  let { Info.env_var; of_string = _; to_string; default } = info field in
  Core_unix.putenv ~key:env_var ~data:(to_string (Option.value value ~default))
;;

let () =
  List.iter Typed_field.Packed.all ~f:(fun { f = T field } ->
    let env_var = info field |> Info.env_var in
    if Option.is_none (Sys.getenv env_var) then set field)
;;
