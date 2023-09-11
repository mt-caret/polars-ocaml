open! Core

(** There are many environment variables that effect Polars behavior, mostly
    related to the display of tables. This module is responsible for setting the
    default values of these variables so that e.g. expect tests will be less
    fragile.

    See https://docs.rs/polars/latest/polars/#config-with-env-vars and
    https://github.com/pola-rs/polars/blob/9c194a24ad6aac33004c8fb0515a90239a376f44/py-polars/polars/config.py
    for more details on which environment variables exist and what they mean. *)

module Table_formatting : sig
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
end

type t =
  { fmt_table_formatting : Table_formatting.t
  ; table_width : int
  }
[@@deriving typed_fields]

module Info : sig
  type 'a t =
    { env_var : string
    ; of_string : string -> 'a
    ; to_string : 'a -> string
    ; default : 'a
    }
  [@@deriving fields]
end

val info : 'a Typed_field.t -> 'a Info.t
val set : ?value:'a -> 'a Typed_field.t -> unit
