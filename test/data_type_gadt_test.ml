open Core
open Polars

(* TODO: perhaps these things should be bundled in a module and there should be a single function like:
   {[
     val data_type_value
       :  'a Data_type.Typed.t
       -> (module Data_type_value with type t = 'a)
   ]}
*)
let rec value_generator : type a. a Data_type.Typed.t -> a Quickcheck.Generator.t =
  fun (type a) (t : a Data_type.Typed.t) : a Quickcheck.Generator.t ->
  let open Base_quickcheck in
  let uint bits = Generator.int_inclusive 0 (Int.(2 ** bits) - 1) in
  let int bits =
    Generator.int_inclusive (-Int.(2 ** (bits - 1))) (Int.(2 ** (bits - 1)) - 1)
  in
  match t with
  | Boolean -> Generator.bool
  | UInt8 -> uint 8
  | UInt16 -> uint 16
  | UInt32 -> uint 32
  | UInt64 -> Generator.int |> Generator.filter ~f:Int.is_non_negative
  | Int8 -> int 8
  | Int16 -> int 16
  | Int32 -> int 32
  | Int64 -> Generator.int
  | Float32 -> Generator.float
  | Float64 -> Generator.float
  | Utf8 ->
    Generator.string
    |> (* Core.String doesn't have a [is_valid_utf_8] function :( *)
    Generator.filter ~f:Stdlib.String.is_valid_utf_8
  | Binary -> Generator.string
  | List t ->
    value_generator t
    |> (* Polars currently doesn't support passing empty Vec<Series> to Series::new.
          See test in rust/polars-ocaml/src/lib.rs. *)
    Generator.list_non_empty
;;

let rec value_shrinker : type a. a Data_type.Typed.t -> a Quickcheck.Shrinker.t =
  fun (type a) (t : a Data_type.Typed.t) : a Quickcheck.Shrinker.t ->
  let open Base_quickcheck in
  match t with
  | Boolean -> Shrinker.bool
  | UInt8 -> Shrinker.int |> Shrinker.filter ~f:Int.is_non_negative
  | UInt16 -> Shrinker.int |> Shrinker.filter ~f:Int.is_non_negative
  | UInt32 -> Shrinker.int |> Shrinker.filter ~f:Int.is_non_negative
  | UInt64 -> Shrinker.int |> Shrinker.filter ~f:Int.is_non_negative
  | Int8 -> Shrinker.int
  | Int16 -> Shrinker.int
  | Int32 -> Shrinker.int
  | Int64 -> Shrinker.int
  | Float32 -> Shrinker.float
  | Float64 -> Shrinker.float
  | Utf8 -> Shrinker.string
  | Binary -> Shrinker.string
  | List t ->
    value_shrinker t |> Shrinker.list |> Shrinker.filter ~f:(Fn.non List.is_empty)
;;

let rec value_to_sexp : type a. a Data_type.Typed.t -> a -> Sexp.t =
  fun (type a) (t : a Data_type.Typed.t) (a : a) ->
  match t with
  | Boolean -> [%sexp_of: bool] a
  | UInt8 -> [%sexp_of: int] a
  | UInt16 -> [%sexp_of: int] a
  | UInt32 -> [%sexp_of: int] a
  | UInt64 -> [%sexp_of: int] a
  | Int8 -> [%sexp_of: int] a
  | Int16 -> [%sexp_of: int] a
  | Int32 -> [%sexp_of: int] a
  | Int64 -> [%sexp_of: int] a
  | Float32 -> [%sexp_of: float] a
  | Float64 -> [%sexp_of: float] a
  | Utf8 -> [%sexp_of: string] a
  | Binary -> [%sexp_of: string] a
  | List t ->
    let sexp_of_value = value_to_sexp t in
    [%sexp_of: value list] a
;;

let rec value_compare : type a. a Data_type.Typed.t -> a -> a -> int =
  fun (type a) (t : a Data_type.Typed.t) (a : a) (b : a) ->
  match t with
  | Boolean -> [%compare: bool] a b
  | UInt8 -> [%compare: int] a b
  | UInt16 -> [%compare: int] a b
  | UInt32 -> [%compare: int] a b
  | UInt64 -> [%compare: int] a b
  | Int8 -> [%compare: int] a b
  | Int16 -> [%compare: int] a b
  | Int32 -> [%compare: int] a b
  | Int64 -> [%compare: int] a b
  | Float32 -> [%compare: float] a b
  | Float64 -> [%compare: float] a b
  | Utf8 -> [%compare: string] a b
  | Binary -> [%compare: string] a b
  | List t -> List.compare (value_compare t) a b
;;

module Series_create = struct
  type t = Args : 'a Data_type.Typed.t * 'a list -> t

  let sexp_of_t (Args (data_type, values)) =
    let sexp_of_value = value_to_sexp data_type in
    [%sexp_of: Data_type.Typed.packed * value list] (Data_type.Typed.T data_type, values)
  ;;

  let quickcheck_generator =
    let open Quickcheck.Generator.Let_syntax in
    let%bind (T data_type) = Data_type.Typed.quickcheck_generator_packed in
    let%map values = Quickcheck.Generator.list_non_empty (value_generator data_type) in
    Args (data_type, values)
  ;;

  let quickcheck_shrinker =
    Quickcheck.Shrinker.create (fun (Args (data_type, values)) ->
      let value_shrinker = Base_quickcheck.Shrinker.list (value_shrinker data_type) in
      Quickcheck.Shrinker.shrink value_shrinker values
      |> Sequence.map ~f:(fun values -> Args (data_type, values)))
  ;;

  let quickcheck_observer =
    Quickcheck.Observer.unmap
      Data_type.Typed.quickcheck_observer_packed
      ~f:(fun (Args (data_type, _values)) -> T data_type)
  ;;
end

let%expect_test "Series.create doesn't raise" =
  Base_quickcheck.Test.run_exn
    (module Series_create)
    ~f:(fun (Series_create.Args (data_type, values)) ->
      let series = Series.create data_type "series_name" values in
      match data_type with
      | List _ | Float32 -> ()
      | _ ->
        let values' = Series.to_list data_type series |> List.filter_opt in
        let value_compare = value_compare data_type in
        assert (List.equal (Comparable.equal value_compare) values values'));
  [%expect {||}]
;;
