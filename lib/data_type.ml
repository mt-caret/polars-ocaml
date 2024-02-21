open! Core

type t =
  | Boolean
  | UInt8
  | UInt16
  | UInt32
  | UInt64
  | Int8
  | Int16
  | Int32
  | Int64
  | Float32
  | Float64
  | Utf8
  | Binary
  | Date
  | Datetime of Time_unit.t * Tz.t option
  | Duration of Time_unit.t
  | Time
  | List of t
  (* We want this branch to be tested very well, since code dealing with
     this recursive case is usually the most non-trivial portion of the
     logic. *)
  [@quickcheck.weight 10.]
  | Null
  | Categorical of (Rev_mapping.t[@compare.ignore]) option [@quickcheck.do_not_generate]
  | Struct of (string * t) list
  | Unknown
[@@deriving compare, sexp_of, quickcheck]

let to_string t = Sexp.to_string ([%sexp_of: t] t)

module Typed = struct
  type untyped = t [@@deriving compare, sexp_of, quickcheck]

  (* TODO: Consider mapping to smaller OCaml values like Int8, Float32, etc instead of
     casting up *)
  type _ t =
    | Boolean : bool t
    | UInt8 : int t
    | UInt16 : int t
    | UInt32 : int t
    | UInt64 : int t
    | Int8 : int t
    | Int16 : int t
    | Int32 : int t
    | Int64 : int t
    | Float32 : float t
    | Float64 : float t
    | Utf8 : string t
    | Binary : string t
    | Date : Naive_date.t t
    | Datetime : Time_unit.t * Tz.t option -> Naive_datetime.t t
    | Duration : Time_unit.t -> Duration.t t
    | Time : Naive_time.t t
    | List : 'a t -> 'a list t
    | Custom :
        { data_type : 'a t
        ; f : 'a -> 'b
        ; f_inverse : 'b -> 'a
        }
        -> 'b t

  let rec strict_type_equal : type a b. a t -> b t -> (a, b) Type_equal.t option =
    fun t1 t2 ->
    match t1, t2 with
    | Boolean, Boolean -> Some Type_equal.T
    | UInt8, UInt8 -> Some Type_equal.T
    | UInt16, UInt16 -> Some Type_equal.T
    | UInt32, UInt32 -> Some Type_equal.T
    | UInt64, UInt64 -> Some Type_equal.T
    | Int8, Int8 -> Some Type_equal.T
    | Int16, Int16 -> Some Type_equal.T
    | Int32, Int32 -> Some Type_equal.T
    | Int64, Int64 -> Some Type_equal.T
    | Float32, Float32 -> Some Type_equal.T
    | Float64, Float64 -> Some Type_equal.T
    | Utf8, Utf8 -> Some Type_equal.T
    | Binary, Binary -> Some Type_equal.T
    | Date, Date -> Some Type_equal.T
    | Datetime (tu1, tz1), Datetime (tu2, tz2) ->
      if [%compare.equal: Time_unit.t * Tz.t option] (tu1, tz1) (tu2, tz2)
      then Some Type_equal.T
      else None
    | Duration tu1, Duration tu2 ->
      if [%compare.equal: Time_unit.t] tu1 tu2 then Some Type_equal.T else None
    | Time, Time -> Some Type_equal.T
    | List t1, List t2 ->
      (match strict_type_equal t1 t2 with
       | None -> None
       | Some Type_equal.T -> Some Type_equal.T)
    | _, _ -> None
  ;;

  let rec flatten_custom : type a. a t -> a t = function
    | Custom
        { data_type = Custom { data_type; f = f'; f_inverse = f_inverse' }; f; f_inverse }
      ->
      flatten_custom
        (Custom
           { data_type; f = Fn.compose f f'; f_inverse = Fn.compose f_inverse' f_inverse })
    | List t ->
      (match flatten_custom t with
       | Custom { data_type; f; f_inverse } ->
         Custom
           { data_type = List data_type
           ; f = List.map ~f
           ; f_inverse = List.map ~f:f_inverse
           }
       | t -> List t)
    | t -> t
  ;;

  type packed = T : _ t -> packed

  let rec to_untyped : type a. a t -> untyped = function
    | Boolean -> Boolean
    | UInt8 -> UInt8
    | UInt16 -> UInt16
    | UInt32 -> UInt32
    | UInt64 -> UInt64
    | Int8 -> Int8
    | Int16 -> Int16
    | Int32 -> Int32
    | Int64 -> Int64
    | Float32 -> Float32
    | Float64 -> Float64
    | Utf8 -> Utf8
    | Binary -> Binary
    | Date -> Date
    | Datetime (time_unit, time_zone) -> Datetime (time_unit, time_zone)
    | Duration time_unit -> Duration time_unit
    | Time -> Time
    | List t -> List (to_untyped t)
    | Custom { data_type; f = _; f_inverse = _ } -> to_untyped data_type
  ;;

  let rec of_untyped : untyped -> packed option = function
    | Boolean -> Some (T Boolean)
    | UInt8 -> Some (T UInt8)
    | UInt16 -> Some (T UInt16)
    | UInt32 -> Some (T UInt32)
    | UInt64 -> Some (T UInt64)
    | Int8 -> Some (T Int8)
    | Int16 -> Some (T Int16)
    | Int32 -> Some (T Int32)
    | Int64 -> Some (T Int64)
    | Float32 -> Some (T Float32)
    | Float64 -> Some (T Float64)
    | Utf8 -> Some (T Utf8)
    | Binary -> Some (T Binary)
    | Date -> Some (T Date)
    | Datetime (time_unit, time_zone) -> Some (T (Datetime (time_unit, time_zone)))
    | Duration time_unit -> Some (T (Duration time_unit))
    | Time -> Some (T Time)
    | List t -> of_untyped t |> Option.map ~f:(fun (T t) -> T (List t))
    | Null | Categorical _ | Struct _ | Unknown -> None
  ;;

  let rec sexp_of_packed (T t) =
    match t with
    | Custom { data_type; f = _; f_inverse = _ } ->
      let sexp = sexp_of_packed (T data_type) in
      [%message "Custom" ~_:(sexp : Sexp.t)]
    | _ -> [%sexp_of: untyped] (to_untyped t)
  ;;

  let compare_packed (T t1) (T t2) = [%compare: untyped] (to_untyped t1) (to_untyped t2)

  type 'a wrapped =
    | Just of 'a
    | Wrapped of 'a wrapped
  [@@deriving quickcheck]

  let rec unwrap : packed wrapped -> packed = function
    | Just t -> t
    | Wrapped wrapped ->
      let (T data_type) = unwrap wrapped in
      T (Custom { data_type; f = Fn.id; f_inverse = Fn.id })
  ;;

  let quickcheck_generator_packed =
    let generator_without_custom =
      Quickcheck.Generator.filter_map quickcheck_generator ~f:of_untyped
    in
    quickcheck_generator_wrapped generator_without_custom
    |> Quickcheck.Generator.map ~f:unwrap
  ;;

  let quickcheck_shrinker_packed =
    Quickcheck.Shrinker.filter_map
      quickcheck_shrinker
      ~f:of_untyped
      ~f_inverse:(fun (T t) -> to_untyped t)
  ;;

  let quickcheck_observer_packed =
    Quickcheck.Observer.unmap quickcheck_observer ~f:(fun (T t) -> to_untyped t)
  ;;

  module Core = struct
    let date =
      Custom
        { data_type = Date; f = Naive_date.to_date_exn; f_inverse = Naive_date.of_date }
    ;;

    let time =
      Custom
        { data_type = Datetime (Nanoseconds, None)
        ; f = Naive_datetime.to_time_ns
        ; f_inverse = Naive_datetime.of_time_ns_exn
        }
    ;;

    let span =
      Custom
        { data_type = Duration Nanoseconds
        ; f = Duration.to_span
        ; f_inverse = Duration.of_span
        }
    ;;

    let ofday =
      Custom
        { data_type = Time; f = Naive_time.to_ofday; f_inverse = Naive_time.of_ofday_exn }
    ;;
  end
end
