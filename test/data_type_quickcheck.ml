open Core
open Polars

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
  | Float32 ->
    Generator.float
    |> Generator.map ~f:(fun x ->
      (* By converting to a 32bit float and back, we generate doubles which do
         not change values even when converted to a 32bit float. *)
      Int32.bits_of_float x |> Int32.float_of_bits)
  | Float64 -> Generator.float
  | Utf8 ->
    Generator.string
    |> (* Core.String doesn't have a [is_valid_utf_8] function :( *)
    Generator.filter ~f:Stdlib.String.is_valid_utf_8
  | Binary -> Generator.string
  | Date -> Date.quickcheck_generator |> Generator.map ~f:Naive_date.of_date
  | Datetime (time_unit, _time_zone) ->
    Time_ns.quickcheck_generator
    |> Generator.map ~f:(fun time_ns ->
      Naive_datetime.of_time_ns_exn time_ns
      |> Naive_datetime.For_testing.round_to_time_unit ~time_unit)
  | Duration time_unit ->
    Time_ns.Span.quickcheck_generator
    |> Generator.map ~f:(fun span ->
      Duration.of_span span |> Duration.For_testing.round_to_time_unit ~time_unit)
  | Time ->
    Time_ns.Ofday.quickcheck_generator |> Generator.filter_map ~f:Naive_time.of_ofday
  | List t -> value_generator t |> Generator.list
  | Custom { data_type; f; f_inverse = _ } ->
    value_generator data_type |> Generator.map ~f
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
  | Date ->
    Date.quickcheck_shrinker
    |> Shrinker.map ~f:Naive_date.of_date ~f_inverse:Naive_date.to_date_exn
  | Datetime (_time_unit, _time_zone) ->
    Time_ns.quickcheck_shrinker
    |> Shrinker.map ~f:Naive_datetime.of_time_ns_exn ~f_inverse:Naive_datetime.to_time_ns
  | Duration _time_unit ->
    Time_ns.Span.quickcheck_shrinker
    |> Shrinker.map ~f:Duration.of_span ~f_inverse:Duration.to_span
  | Time ->
    Time_ns.Ofday.quickcheck_shrinker
    |> Shrinker.map ~f:Naive_time.of_ofday_exn ~f_inverse:Naive_time.to_ofday
  | List t ->
    value_shrinker t |> Shrinker.list |> Shrinker.filter ~f:(Fn.non List.is_empty)
  | Custom { data_type; f; f_inverse } ->
    value_shrinker data_type |> Shrinker.map ~f ~f_inverse
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
  | Date -> [%sexp_of: Date.t] (Naive_date.to_date_exn a)
  | Datetime (time_unit, time_zone) ->
    [%sexp_of: Time_ns.Alternate_sexp.t * Time_unit.t * Tz.t option]
      (Naive_datetime.to_time_ns a, time_unit, time_zone)
  | Time -> [%sexp_of: Time_ns.Ofday.t] (Naive_time.to_ofday a)
  | Duration time_unit ->
    [%sexp_of: Time_ns.Span.t * Time_unit.t] (Duration.to_span a, time_unit)
  | List t ->
    let sexp_of_value = value_to_sexp t in
    [%sexp_of: value list] a
  | Custom { data_type; f = _; f_inverse } -> value_to_sexp data_type (f_inverse a)
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
  | Date -> Comparable.lift [%compare: Date.t] ~f:Naive_date.to_date_exn a b
  | Datetime (_time_unit, _time_zone) ->
    Comparable.lift [%compare: Time_ns.t] ~f:Naive_datetime.to_time_ns a b
  | Duration _time_unit ->
    Comparable.lift [%compare: Time_ns.Span.t] ~f:Duration.to_span a b
  | Time -> Comparable.lift [%compare: Time_ns.Ofday.t] ~f:Naive_time.to_ofday a b
  | List t -> List.compare (value_compare t) a b
  | Custom { data_type; f = _; f_inverse } ->
    Comparable.lift (value_compare data_type) ~f:f_inverse a b
;;

module type Value_module = sig
  type t [@@deriving sexp_of, compare, quickcheck]
end

let data_type_value (type a) (data_type : a Data_type.Typed.t)
  : (module Value_module with type t = a)
  =
  (module struct
    type t = a

    let quickcheck_generator = value_generator data_type
    let quickcheck_shrinker = value_shrinker data_type
    let quickcheck_observer = Base_quickcheck.Observer.opaque
    let sexp_of_t = value_to_sexp data_type
    let compare = value_compare data_type
  end)
;;
