open! Core

(** An [Expr.t] is the basic building block of how to select columns from a
    dataframe. It is a representation of a lazy computation over a dataframe
    which can be executed via functions such as [Lazy_frame.select]/
    [Data_frame.select] and [Lazy_frame.with_columns]/[Data_frame.with_columns]. *)

type t

(** [col] return column(s) in a dataframe:
    {@ocaml[
      # let df =
          Data_frame.create_exn
            Series.
              [ int "ham" [ 1; 2; 3 ]
              ; int "hamburger" [ 11; 22; 33 ]
              ; int "foo" [ 3; 2; 1 ]
              ; string "bar" [ "a"; "b"; "c" ]
              ]
            ;;
      ...

      # Data_frame.select_exn df ~exprs:Expr.[ col "foo" ]
      - : Data_frame.t =
      shape: (3, 1)
      +-----+
      | foo |
      | --- |
      | i64 |
      +=====+
      | 3   |
      | 2   |
      | 1   |
      +-----+
    ]}

    Use the wildcard [*] to represent all columns:
    {@ocaml[
      # Data_frame.select_exn df ~exprs:Expr.[ col "*" ]
      - : Data_frame.t =
      shape: (3, 4)
      +-----+-----------+-----+-----+
      | ham | hamburger | foo | bar |
      | --- | ---       | --- | --- |
      | i64 | i64       | i64 | str |
      +=============================+
      | 1   | 11        | 3   | a   |
      | 2   | 22        | 2   | b   |
      | 3   | 33        | 1   | c   |
      +-----+-----------+-----+-----+

      # Data_frame.select_exn df ~exprs:Expr.[ col "*" |> exclude ~names:[ "ham" ] ]
      - : Data_frame.t =
      shape: (3, 3)
      +-----------+-----+-----+
      | hamburger | foo | bar |
      | ---       | --- | --- |
      | i64       | i64 | str |
      +=======================+
      | 11        | 3   | a   |
      | 22        | 2   | b   |
      | 33        | 1   | c   |
      +-----------+-----+-----+
    ]}

    Regular expressions are also supported:
    {@ocaml[
      # Data_frame.select_exn df ~exprs:Expr.[ col "^ham.*$" ]
      - : Data_frame.t =
      shape: (3, 2)
      +-----+-----------+
      | ham | hamburger |
      | --- | ---       |
      | i64 | i64       |
      +=================+
      | 1   | 11        |
      | 2   | 22        |
      | 3   | 33        |
      +-----+-----------+
    ]} *)
val col : string -> t

(** Use [cols] to specify multiple columns at once:
    {@ocaml[
      # Data_frame.select_exn df ~exprs:Expr.[ cols [ "hamburger"; "foo" ] ]
      - : Data_frame.t =
      shape: (3, 2)
      +-----------+-----+
      | hamburger | foo |
      | ---       | --- |
      | i64       | i64 |
      +=================+
      | 11        | 3   |
      | 22        | 2   |
      | 33        | 1   |
      +-----------+-----+
    ]} *)
val cols : string list -> t

(** [all] selects all columns in the dataframe:

    {@ocaml[
      # let df =
          Data_frame.create_exn
            Series.
              [ bool "a" [ true; false; true ]
              ; bool "b" [ false; false; false ]
              ]
            ;;
      ...

      # Data_frame.select_exn df ~exprs:Expr.[ all () |> sum ]
      - : Data_frame.t =
      shape: (1, 2)
      +-----+-----+
      | a   | b   |
      | --- | --- |
      | u32 | u32 |
      +===========+
      | 2   | 0   |
      +-----+-----+
    ]} *)
val all : unit -> t

(** [exclude] excludes columns from a multi-column expression:

    {@ocaml[
      # let df =
        Data_frame.create_exn
          Series.
            [ int "aa" [ 1; 2; 3 ]
            ; string_option "ba" [ Some "a"; Some "b"; None ]
            ; float_option "cc" [ None; Some 2.5; Some 1.5 ]
            ]
          ;;
      val df : Data_frame.t =
        shape: (3, 3)
      +-----+------+------+
      | aa  | ba   | cc   |
      | --- | ---  | ---  |
      | i64 | str  | f64  |
      +===================+
      | 1   | a    | null |
      | 2   | b    | 2.5  |
      | 3   | null | 1.5  |
      +-----+------+------+

      # Data_frame.select_exn df ~exprs:Expr.[ all () |> exclude ~names:[ "ba" ] ]
      - : Data_frame.t =
      shape: (3, 2)
      +-----+------+
      | aa  | cc   |
      | --- | ---  |
      | i64 | f64  |
      +============+
      | 1   | null |
      | 2   | 2.5  |
      | 3   | 1.5  |
      +-----+------+
    ]}

    Regular expressions are also supported:

    {@ocaml[
      # Data_frame.select_exn df ~exprs:Expr.[ all () |> exclude ~names:[ "^.*a$" ] ]
      - : Data_frame.t =
      shape: (3, 1)
      +------+
      | cc   |
      | ---  |
      | f64  |
      +======+
      | null |
      | 2.5  |
      | 1.5  |
      +------+
    ]} *)
val exclude : t -> names:string list -> t

val element : unit -> t
val cast : ?strict:bool -> t -> to_:Data_type.t -> t
val null : unit -> t
val int : int -> t
val float : float -> t
val bool : bool -> t
val string : string -> t
val naive_date : Common.Naive_date.t -> t
val naive_datetime : Common.Naive_datetime.t -> t
val series : Series.t -> t
val sort : ?descending:bool -> t -> t
val sort_by : ?descending:bool -> t -> by:t list -> t
val set_sorted_flag : t -> sorted:[ `Ascending | `Descending | `Not ] -> t
val first : t -> t
val last : t -> t
val reverse : t -> t
val head : ?length:int -> t -> t
val tail : ?length:int -> t -> t
val take : t -> idx:t -> t

val sample_n
  :  ?seed:int
  -> ?fixed_seed:bool
  -> t
  -> n:int
  -> with_replacement:bool
  -> shuffle:bool
  -> t

val filter : t -> predicate:t -> t
val ceil : t -> t
val floor : t -> t
val clip_min_float : t -> min:float -> t
val clip_max_float : t -> max:float -> t
val clip_min_int : t -> min:int -> t
val clip_max_int : t -> max:int -> t
val sum : t -> t
val mean : t -> t
val median : t -> t
val max : t -> t
val min : t -> t
val arg_max : t -> t
val arg_min : t -> t
val count : t -> t
val count_ : unit -> t
val n_unique : t -> t
val approx_unique : t -> t
val explode : t -> t

val over
  :  ?mapping_strategy:[ `Groups_to_rows | `Explode | `Join ]
  -> t
  -> partition_by:t list
  -> t

val concat_list : t Nonempty_list.t -> t
val null_count : t -> t
val is_null : t -> t
val is_not_null : t -> t
val is_nan : t -> t
val is_not_nan : t -> t
val fill_null : t -> with_:t -> t
val fill_null' : t -> strategy:Fill_null_strategy.t -> t
val interpolate : ?method_:[ `Linear | `Nearest ] -> t -> t
val fill_nan : t -> with_:t -> t

val rank
  :  ?method_:[ `Average | `Dense | `Max | `Min | `Ordinal | `Random ]
  -> ?descending:bool
  -> ?seed:int
  -> t
  -> t

val when_ : (t * t) list -> otherwise:t -> t
val shift : ?fill_value:t -> t -> periods:int -> t
val cum_count : ?reverse:bool -> t -> t
val cum_sum : ?reverse:bool -> t -> t
val cum_prod : ?reverse:bool -> t -> t
val cum_min : ?reverse:bool -> t -> t
val cum_max : ?reverse:bool -> t -> t
val alias : t -> name:string -> t
val prefix : t -> prefix:string -> t
val suffix : t -> suffix:string -> t
val round : t -> decimals:int -> t
val equal : t -> t -> t

include Common.Compare with type t := t
include Common.Logic with type t := t
include Common.Numeric with type t := t

(* TODO: apparently this doesn't exist for series, which is surprising! *)
val floor_div : t -> t -> t
val ( // ) : t -> t -> t

module Dt : sig
  val strftime : t -> format:string -> t
  val convert_time_zone : t -> to_:string -> t
  val replace_time_zone : ?use_earliest:bool -> t -> to_:string option -> t
  val year : t -> t
  val month : t -> t
  val day : t -> t
  val days : t -> t
  val hours : t -> t
  val minutes : t -> t
  val seconds : t -> t
  val milliseconds : t -> t
  val microseconds : t -> t
  val nanoseconds : t -> t
end

module Str : sig
  val split : ?inclusive:bool -> t -> by:string -> t
  val strptime : t -> type_:Data_type.t -> format:string -> t
  val lengths : t -> t
  val n_chars : t -> t
  val contains : ?literal:bool -> t -> pat:string -> t
  val starts_with : t -> prefix:string -> t
  val ends_with : t -> suffix:string -> t
  val extract : t -> pat:string -> group:int -> t
  val extract_all : t -> pat:string -> t
  val replace : ?literal:bool -> t -> pat:string -> with_:string -> t
  val replace_all : ?literal:bool -> t -> pat:string -> with_:string -> t
  val strip : ?matches:string -> t -> t
  val lstrip : ?matches:string -> t -> t
  val rstrip : ?matches:string -> t -> t
  val to_lowercase : t -> t
  val to_uppercase : t -> t
  val slice : t -> start:int -> length:int option -> t
end

module List : sig
  val lengths : t -> t
  val slice : t -> offset:t -> length:t -> t
  val head : t -> n:t -> t
  val tail : t -> n:t -> t
  val sum : t -> t
  val eval : ?parallel:bool -> t -> expr:t -> t
end
