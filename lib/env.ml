open! Core

(* There are many environment variables that effect Polars behavior, mostly
   related to the display of tables. This module is responsible for setting the
   default values of these variables so that e.g. expect tests will be less
   fragile.

   See https://docs.rs/polars/latest/polars/#config-with-env-vars and
   https://github.com/pola-rs/polars/blob/9c194a24ad6aac33004c8fb0515a90239a376f44/py-polars/polars/config.py
   for more details on which environment variables exist.
*)
type t = { table_width : int } [@@deriving typed_fields]

module Info = struct
  type 'a t =
    { env_var : string
    ; of_string : string -> 'a
    ; to_string : 'a -> string
    ; default : 'a
    }
end

let info : type a. a Typed_field.t -> a Info.t = function
  | Table_width ->
    { env_var = "POLARS_TABLE_WIDTH"
    ; of_string = Int.of_string
    ; to_string = Int.to_string
    ; default = 100
    }
;;

let () =
  List.iter Typed_field.Packed.all ~f:(fun { f = T field } ->
    let { Info.env_var; of_string = _; to_string; default } = info field in
    if Option.is_none (Sys.getenv env_var)
    then Core_unix.putenv ~key:env_var ~data:(to_string default))
;;
