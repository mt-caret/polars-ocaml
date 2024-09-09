open! Core

(** An [Expr.t] is the basic building block of how to select columns from a
    dataframe. It is a representation of a lazy computation over a dataframe
    which can be executed via functions such as [Lazy_frame.select]/
    [Data_frame.select] and [Lazy_frame.with_columns]/[Data_frame.with_columns]. *)

type t

(* TODO: Functions here should probably be grouped and reordered into rough
   categories shown in
   https://pola-rs.github.io/polars/py-polars/html/reference/expressions/index.html
*)

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
      - : Data_frame0.t =
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
      - : Data_frame0.t =
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
      - : Data_frame0.t =
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
      - : Data_frame0.t =
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
      - : Data_frame0.t =
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
        in
        Data_frame.select_exn df ~exprs:Expr.[ all () |> sum ]
      - : Data_frame0.t =
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
            ; stringo "ba" [ Some "a"; Some "b"; None ]
            ; floato "cc" [ None; Some 2.5; Some 1.5 ]
            ]
          ;;
      val df : Data_frame0.t =
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
      - : Data_frame0.t =
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
      - : Data_frame0.t =
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

(** [element ()] is an alias for an element being evaluated in an [List.eval]
    expression.

    A horizontal rank computation by taking the elements of a list:

    {@ocaml[
      # let df =
          Data_frame.create_exn
            Series.
              [ int "a" [ 1; 8; 3 ]
              ; int "b" [ 4; 5; 2 ]
              ]
        in
        Data_frame.with_columns_exn df ~exprs:Expr.[
          concat_list [ cols ["a"; "b"] ]
          |> List.eval ~expr:(element () |> rank)
          |> alias ~name:"rank"
        ]
      - : Data_frame0.t =
      shape: (3, 3)
      +-----+-----+-----------+
      | a   | b   | rank      |
      | --- | --- | ---       |
      | i64 | i64 | list[u32] |
      +=======================+
      | 1   | 4   | [1, 2]    |
      | 8   | 5   | [2, 1]    |
      | 3   | 2   | [2, 1]    |
      +-----+-----+-----------+
    ]}

    A mathematical operation on array elements:

    {@ocaml[
      # let df =
          Data_frame.create_exn
            Series.
              [ int "a" [ 1; 8; 3 ]
              ; int "b" [ 4; 5; 2 ]
              ]
        in
        Data_frame.with_columns_exn df ~exprs:Expr.[
          concat_list [ cols ["a"; "b"] ]
          |> List.eval ~expr:(element () * int 2)
          |> alias ~name:"a_b_doubled"
        ]
      - : Data_frame0.t =
      shape: (3, 3)
      +-----+-----+-------------+
      | a   | b   | a_b_doubled |
      | --- | --- | ---         |
      | i64 | i64 | list[i64]   |
      +=========================+
      | 1   | 4   | [2, 8]      |
      | 8   | 5   | [16, 10]    |
      | 3   | 2   | [6, 4]      |
      +-----+-----+-------------+
    ]} *)
val element : unit -> t

(** [cast] casts the values to a different data type. [strict] defaults to true,
    and raises an error if the conversion could not be done, for example, due
    to an overflow.

    {@ocaml[
      # let df =
          Data_frame.create_exn
            Series.
              [ int "a" [ 1; 8; 3 ]
              ; string "b" [ "4"; "5"; "2" ]
              ]
        in
        Data_frame.with_columns_exn df ~exprs:Expr.[
          col "a" |> cast ~to_:Float64
        ; col "b" |> cast ~to_:Int32
        ]
      - : Data_frame0.t =
      shape: (3, 2)
      +-----+-----+
      | a   | b   |
      | --- | --- |
      | f64 | i32 |
      +===========+
      | 1.0 | 4   |
      | 8.0 | 5   |
      | 3.0 | 2   |
      +-----+-----+
    ]} *)
val cast : ?strict:bool -> t -> to_:Data_type.t -> t

(** [null], [lit], [int], [float], [bool], [string], [naive_date], [naive_datetime],
    and [series] are expressions representing literal values.

    {@ocaml[
      # let df = Data_frame.create_exn Series.[ int "a" [ 1; 2; 3 ] ] in
        Data_frame.select_exn df ~exprs:Expr.[ col "a" + int 1 ]
      - : Data_frame0.t =
      shape: (3, 1)
      +-----+
      | a   |
      | --- |
      | i64 |
      +=====+
      | 2   |
      | 3   |
      | 4   |
      +-----+

      # let df = Data_frame.create_exn Series.[ string "a" [ "1"; "2"; "3" ] ] in
        Data_frame.select_exn df ~exprs:Expr.[ col "a" + string "1" ]
      - : Data_frame0.t =
      shape: (3, 1)
      +-----+
      | a   |
      | --- |
      | str |
      +=====+
      | 11  |
      | 21  |
      | 31  |
      +-----+

      # let df = Data_frame.create_exn Series.[ int "a" [ 1; 2; 3 ] ] in
        Data_frame.select_exn df ~exprs:Expr.[ col "a" + lit Int32 1 ]
      - : Data_frame0.t =
      shape: (3, 1)
      +-----+
      | a   |
      | --- |
      | i64 |
      +=====+
      | 2   |
      | 3   |
      | 4   |
      +-----+

      # let df = Data_frame.create_exn Series.[ int "a" [ 1; 2; 3 ] ] in
        Data_frame.select_exn df ~exprs:Expr.[ col "a" + series (Series.int "b" [ 1; 1; 1 ]) ]
      - : Data_frame0.t =
      shape: (3, 1)
      +-----+
      | a   |
      | --- |
      | i64 |
      +=====+
      | 2   |
      | 3   |
      | 4   |
      +-----+
    ]} *)
val null : unit -> t

val lit : 'a Data_type.Typed.t -> 'a -> t
val int : int -> t
val float : float -> t
val bool : bool -> t
val string : string -> t
val naive_date : Naive_date.t -> t
val naive_datetime : Naive_datetime.t -> t
val time : Time_ns.t -> t
val series : Series.t -> t

(** [sort] sorts this column. When used in a projection/selection context, the
    whole column is sorted. When used in a groupby context, the groups are
    sorted.

    {@ocaml[
      # let df =
          Data_frame.create_exn
            Series.[ into "a" [ Some 1; None; Some 3; Some 2 ] ]
      ...

      # Data_frame.with_columns_exn df ~exprs:Expr.[ col "a" |> sort ]
      - : Data_frame0.t =
      shape: (4, 1)
      +------+
      | a    |
      | ---  |
      | i64  |
      +======+
      | null |
      | 1    |
      | 2    |
      | 3    |
      +------+

      # Data_frame.with_columns_exn df ~exprs:Expr.[ col "a" |> sort ~descending:true ]
      - : Data_frame0.t =
      shape: (4, 1)
      +------+
      | a    |
      | ---  |
      | i64  |
      +======+
      | null |
      | 3    |
      | 2    |
      | 1    |
      +------+

      # let df =
          Data_frame.create_exn
            Series.
              [ string "group" [ "one"; "one"; "one"; "two"; "two"; "two" ]
              ; int "value" [ 1; 98; 2; 3; 99; 4 ]
              ]
      val df : Data_frame0.t =
        shape: (6, 2)
      +-------+-------+
      | group | value |
      | ---   | ---   |
      | str   | i64   |
      +===============+
      | one   | 1     |
      | one   | 98    |
      | one   | 2     |
      | two   | 3     |
      | two   | 99    |
      | two   | 4     |
      +-------+-------+

      # Data_frame.groupby
          df
          ~by:Expr.[ col "group" ]
          ~agg:Expr.[ col "value" |> sort ]
      - : (Data_frame0.t, string) result =
      Core.Ok
       shape: (2, 2)
      +-------+------------+
      | group | value      |
      | ---   | ---        |
      | str   | list[i64]  |
      +====================+
      | one   | [1, 2, 98] |
      | two   | [3, 4, 99] |
      +-------+------------+
    ]} *)
val sort : ?descending:bool -> t -> t

(** [sort_by] sorts this column based on the ordering on expressions passed to
    [by]. When used in a projection/selection context, the whole column is
    sorted. When used in a groupby context, the groups are sorted.

    Pass a single column name ot sort by that column:

    {@ocaml[
      # let df =
        Data_frame.create_exn
          Series.
            [ string "group" [ "a"; "a"; "b"; "b" ]
            ; int "value1" [ 1; 3; 4; 2 ]
            ; int "value2" [ 8; 7; 6; 5 ]
            ]
      ...

      # Data_frame.select_exn
          df
          ~exprs:Expr.[ col "group" |> sort_by ~by:[ col "value1" ] ]
      - : Data_frame0.t =
      shape: (4, 1)
      +-------+
      | group |
      | ---   |
      | str   |
      +=======+
      | a     |
      | b     |
      | a     |
      | b     |
      +-------+
    ]}

    Sorting by expressions is also supported:

    {@ocaml[
      # Data_frame.select_exn
          df
          ~exprs:Expr.[ col "group" |> sort_by ~by:[ col "value1" + col "value2" ] ]
      - : Data_frame0.t =
      shape: (4, 1)
      +-------+
      | group |
      | ---   |
      | str   |
      +=======+
      | b     |
      | a     |
      | a     |
      | b     |
      +-------+
    ]}

    Sort by multiple columns by passing a list of columns:

    {@ocaml[
      # Data_frame.select_exn
          df
          ~exprs:Expr.[ col "group" |> sort_by ~by:[ col "value1"; col "value2" ] ~descending:true ]
      - : Data_frame0.t =
      shape: (4, 1)
      +-------+
      | group |
      | ---   |
      | str   |
      +=======+
      | b     |
      | a     |
      | b     |
      | a     |
      +-------+
    ]}

    When sorting in a groupby context, the groups are sorted:

    {@ocaml[
      # Data_frame.groupby
          df
          ~by:Expr.[ col "group" ]
          ~agg:Expr.[ col "value1" |> sort_by ~by:[ col "value2" ] ]
      - : (Data_frame0.t, string) result =
      Core.Ok
       shape: (2, 2)
      +-------+-----------+
      | group | value1    |
      | ---   | ---       |
      | str   | list[i64] |
      +===================+
      | a     | [3, 1]    |
      | b     | [2, 4]    |
      +-------+-----------+
    ]}

    Take a single row from each group where a column attains its minimal value
    within that group:

    {@ocaml[
      # Data_frame.groupby_exn
          df
          ~by:Expr.[ col "group" ] ~agg:Expr.[ all () |> sort_by ~by:[ col "value2" ] ]
      - : Data_frame0.t =
      shape: (2, 3)
      +-------+-----------+-----------+
      | group | value1    | value2    |
      | ---   | ---       | ---       |
      | str   | list[i64] | list[i64] |
      +===============================+
      | a     | [3, 1]    | [7, 8]    |
      | b     | [2, 4]    | [5, 6]    |
      +-------+-----------+-----------+
    ]} *)
val sort_by : ?descending:bool -> t -> by:t list -> t

(** [set_sorted_flag] allows manipulation of the flag used for toggling the fast
    path for sorted arrays. Please not that this would result in incorrect
    results if the underlying data is not sorted.

    {@ocaml[
      # let df = Data_frame.create_exn Series.[ int "values" [ 1; 2; 3 ] ] in
        Data_frame.select_exn
          df
          ~exprs:Expr.[ col "values" |> set_sorted_flag ~sorted:`Ascending |> max ]
      - : Data_frame0.t =
      shape: (1, 1)
      +--------+
      | values |
      | ---    |
      | i64    |
      +========+
      | 3      |
      +--------+
    ]} *)
val set_sorted_flag : t -> sorted:[ `Ascending | `Descending | `Not ] -> t

(** Get the first value:

    {@ocaml[
      # let df = Data_frame.create_exn Series.[ int "a" [ 1; 1; 2 ] ] in
        Data_frame.select_exn df ~exprs:Expr.[ col "a" |> first ]
      - : Data_frame0.t =
      shape: (1, 1)
      +-----+
      | a   |
      | --- |
      | i64 |
      +=====+
      | 1   |
      +-----+
    ]} *)
val first : t -> t

(** Get the first value:

    {@ocaml[
      # let df = Data_frame.create_exn Series.[ int "a" [ 1; 1; 2 ] ] in
        Data_frame.select_exn df ~exprs:Expr.[ col "a" |> last ]
      - : Data_frame0.t =
      shape: (1, 1)
      +-----+
      | a   |
      | --- |
      | i64 |
      +=====+
      | 2   |
      +-----+
    ]} *)
val last : t -> t

(** [reverse] reverses the selection:

    {@ocaml[
      # let df =
          Data_frame.create_exn
            Series.
              [ int "A" [ 1; 2; 3; 4; 5 ]
              ; string "fruits" [ "banana"; "banana"; "apple"; "apple"; "banana" ]
              ; int "B" [ 5; 4; 3; 2; 1 ]
              ; string "cars" [ "beetle"; "audi"; "beetle"; "beetle"; "beetle" ]
              ]
        in
        Data_frame.select_exn df ~exprs:Expr.[ all (); all () |> reverse |> suffix ~suffix:"_reverse" ]
      - : Data_frame0.t =
      shape: (5, 8)
      +-----+--------+-----+--------+-----------+----------------+-----------+--------------+
      | A   | fruits | B   | cars   | A_reverse | fruits_reverse | B_reverse | cars_reverse |
      | --- | ---    | --- | ---    | ---       | ---            | ---       | ---          |
      | i64 | str    | i64 | str    | i64       | str            | i64       | str          |
      +=====================================================================================+
      | 1   | banana | 5   | beetle | 5         | banana         | 1         | beetle       |
      | 2   | banana | 4   | audi   | 4         | apple          | 2         | beetle       |
      | 3   | apple  | 3   | beetle | 3         | apple          | 3         | beetle       |
      | 4   | apple  | 2   | beetle | 2         | banana         | 4         | audi         |
      | 5   | banana | 1   | beetle | 1         | banana         | 5         | beetle       |
      +-----+--------+-----+--------+-----------+----------------+-----------+--------------+
    ]} *)
val reverse : t -> t

(** [head] returns the first n rows, defaulting to 10:

    {@ocaml[
      # let df = Data_frame.create_exn Series.[ int "foo" [ 1; 2; 3; 4; 5; 6; 7 ] ] in
        Data_frame.select_exn df ~exprs:Expr.[ col "foo" |> head ~length:3 ]
      - : Data_frame0.t =
      shape: (3, 1)
      +-----+
      | foo |
      | --- |
      | i64 |
      +=====+
      | 1   |
      | 2   |
      | 3   |
      +-----+
    ]} *)
val head : ?length:int -> t -> t

(** [tail] returns the last n rows, defaulting to 10:

    {@ocaml[
      # let df = Data_frame.create_exn Series.[ int "foo" [ 1; 2; 3; 4; 5; 6; 7 ] ] in
        Data_frame.select_exn df ~exprs:Expr.[ col "foo" |> tail ~length:3 ]
      - : Data_frame0.t =
      shape: (3, 1)
      +-----+
      | foo |
      | --- |
      | i64 |
      +=====+
      | 5   |
      | 6   |
      | 7   |
      +-----+
    ]} *)
val tail : ?length:int -> t -> t

(** [take] returns the value based on index:

    {@ocaml[
      # let df =
          Data_frame.create_exn
            Series.
              [ string "group" [ "one"; "one"; "one"; "two"; "two"; "two" ]
              ; int "value" [ 1; 98; 2; 3; 99; 4 ] ]
        in
        Data_frame.groupby_exn
          df
          ~by:Expr.[ col "group" ]
          ~agg:Expr.[ col "value" |> take ~idx:(int 1) ]
      - : Data_frame0.t =
      shape: (2, 2)
      +-------+-------+
      | group | value |
      | ---   | ---   |
      | str   | i64   |
      +===============+
      | one   | 98    |
      | two   | 99    |
      +-------+-------+
    ]} *)
val take : t -> idx:t -> t

(** [sample_n] samples n times from expression:

    {@ocaml[
      # let df =Data_frame.create_exn Series.[ int "a" [ 1; 2; 3 ] ] in
        Data_frame.select_exn
          df
          ~exprs:
            Expr.
              [ col "a" |> sample_n ~seed:0 ~fixed_seed:true ~n:2 ~with_replacement:true ~shuffle:true ]
      - : Data_frame0.t =
      shape: (2, 1)
      +-----+
      | a   |
      | --- |
      | i64 |
      +=====+
      | 2   |
      | 2   |
      +-----+
    ]}

    @param seed
      The seed for the random number generator; if not passed will to use the
      built-in random number generator in Rust.

    @param fixed_seed
      if true, the seed will not be incremented between draws, making output
      predictable when executing concurrently (defaults to true). *)
val sample_n
  :  ?seed:int
  -> ?fixed_seed:bool
  -> t
  -> n:int
  -> with_replacement:bool
  -> shuffle:bool
  -> t

(** [filter] filters a single column according to a predicate:

    {@ocaml[
      # let df =
          Data_frame.create_exn
            Series.
              [ string "group_col" [ "g1"; "g1"; "g2" ]
              ; int "b" [ 1; 2; 3 ]
              ]
        in
        Data_frame.groupby_exn
          df
          ~by:Expr.[ col "group_col" ]
          ~agg:
            Expr.
              [ col "b" |> filter ~predicate:(col "b" < int 2) |> sum |> alias ~name:"lt"
              ; col "b" |> filter ~predicate:(col "b" >= int 2) |> sum |> alias ~name:"gte"
              ]
        |> Data_frame.sort_exn ~by_column:[ "group_col" ]
      - : Data_frame0.t =
      shape: (2, 3)
      +-----------+-----+-----+
      | group_col | lt  | gte |
      | ---       | --- | --- |
      | str       | i64 | i64 |
      +=======================+
      | g1        | 1   | 2   |
      | g2        | 0   | 3   |
      +-----------+-----+-----+
    ]} *)
val filter : t -> predicate:t -> t

val is_in : t -> other:t -> t

(** [ceil] and [floor] rounds up and down to the nearest integer value,
    respectively. This function only works for floating point values.

    {@ocaml[
      # let df = Data_frame.create_exn Series.[ float "a" [ 0.3; 0.5; 1.0; 1.1 ] ] in
        Data_frame.select_exn
          df
          ~exprs:
            Expr.
              [ col "a" |> ceil |> alias ~name:"a_ceil"
              ; col "a" |> ceil |> alias ~name:"a_floor"
              ]
      - : Data_frame0.t =
      shape: (4, 2)
      +--------+---------+
      | a_ceil | a_floor |
      | ---    | ---     |
      | f64    | f64     |
      +==================+
      | 1.0    | 1.0     |
      | 1.0    | 1.0     |
      | 1.0    | 1.0     |
      | 2.0    | 2.0     |
      +--------+---------+
    ]} *)
val ceil : t -> t

val floor : t -> t

(** [clip_min_float], [clip_max_float], [clip_min_int], and [clip_max_int] clip
    (limit) the values to a min/max boundary. This function only works for
    numerical types.

    {@ocaml[
      # let df = Data_frame.create_exn Series.[ floato "foo" [ Some (-50.); Some (5.); None; Some (50.) ] ] in
        Data_frame.select_exn
          df
          ~exprs:
            Expr.
              [ col "foo" |> clip_min_float ~min:(-10.) |> alias ~name:"clipped_min"
              ; col "foo" |> clip_max_float ~max:10. |> alias ~name:"clipped_max"
              ; col "foo" |> clip_min_int ~min:(-10) |> alias ~name:"clipped_min_int"
              ; col "foo" |> clip_max_int ~max:10 |> alias ~name:"clipped_max_int"
              ]
      - : Data_frame0.t =
      shape: (4, 4)
      +-------------+-------------+-----------------+-----------------+
      | clipped_min | clipped_max | clipped_min_int | clipped_max_int |
      | ---         | ---         | ---             | ---             |
      | f64         | f64         | f64             | f64             |
      +===============================================================+
      | -10.0       | -50.0       | -10.0           | -50.0           |
      | 5.0         | 5.0         | 5.0             | 5.0             |
      | null        | null        | null            | null            |
      | 50.0        | 10.0        | 50.0            | 10.0            |
      +-------------+-------------+-----------------+-----------------+
    ]} *)
val clip_min_float : t -> min:float -> t

val clip_max_float : t -> max:float -> t
val clip_min_int : t -> min:int -> t
val clip_max_int : t -> max:int -> t
val pow : t -> t -> t

(** [sum], [mean], [median], and [mode] calculate the sum, mean, median, and
    mode respectively.

    Show examples where each aggregation differs when calculated for a single series:
    {@ocaml[
      # let df = Data_frame.create_exn Series.[ int "a" [ 1; 2; 2; 2; 3; 4; 5; 6; 7 ] ] in
        Data_frame.select_exn
          df
          ~exprs:
            Expr.
              [ col "a" |> sum |> alias ~name:"sum"
              ; col "a" |> mean |> alias ~name:"mean"
              ; col "a" |> median |> alias ~name:"median"
              ; col "a" |> mode |> alias ~name:"mode"
              ]
      - : Data_frame0.t =
      shape: (1, 4)
      +-----+----------+--------+------+
      | sum | mean     | median | mode |
      | --- | ---      | ---    | ---  |
      | i64 | f64      | f64    | i64  |
      +================================+
      | 32  | 3.555556 | 3.0    | 2    |
      +-----+----------+--------+------+
    ]} *)
val sum : t -> t

val mean : t -> t
val median : t -> t

val quantile
  :  ?interpol_option:[ `Nearest | `Lower | `Higher | `Midpoint | `Linear ]
  -> t
  -> quantile_expr:t
  -> t

val mode : t -> t

(** [max], [min], [arg_max], and [arg_min] calculate the max, min, and indices
    associated with the max and min value, respectively.

    {@ocaml[
      # let df = Data_frame.create_exn Series.[ int "a" [ 1; 2; 2; 2; 3; 4; 5; 6; 7 ] ] in
        Data_frame.select_exn
          df
          ~exprs:
            Expr.
              [ col "a" |> max |> alias ~name:"max"
              ; col "a" |> min |> alias ~name:"min"
              ; col "a" |> arg_max |> alias ~name:"arg_max"
              ; col "a" |> arg_min |> alias ~name:"arg_min"
              ]
      - : Data_frame0.t =
      shape: (1, 4)
      +-----+-----+---------+---------+
      | max | min | arg_max | arg_min |
      | --- | --- | ---     | ---     |
      | i64 | i64 | u32     | u32     |
      +===============================+
      | 7   | 1   | 8       | 0       |
      +-----+-----+---------+---------+
    ]} *)
val max : t -> t

val min : t -> t
val arg_max : t -> t
val arg_min : t -> t

(** [count] counts the nuber of values associated with an expression. Please
    note that null values are also included in this count.

    {@ocaml[
      # let df =
          Data_frame.create_exn
            Series.
              [ int "a" [ 1; 2; 3 ]
              ; into "b" [ None; Some 4; Some 4 ]
              ]
        in
        Data_frame.select_exn df ~exprs:Expr.[ all () |> count ]
      - : Data_frame0.t =
      shape: (1, 2)
      +-----+-----+
      | a   | b   |
      | --- | --- |
      | u32 | u32 |
      +===========+
      | 3   | 3   |
      +-----+-----+
    ]} *)
val count : t -> t

(** [count_] counts the number of values in this column/context.

    {@ocaml[
      # let df =
          Data_frame.create_exn
            Series.
              [ int "a" [ 1; 8; 3 ]
              ; int "b" [ 4; 5; 2 ]
              ; string "c" [ "foo"; "bar"; "foo" ]
              ]
      ...

      # Data_frame.select_exn df ~exprs:Expr.[ count_ () ]
      - : Data_frame0.t =
      shape: (1, 1)
      +-------+
      | count |
      | ---   |
      | u32   |
      +=======+
      | 3     |
      +-------+

      # Data_frame.groupby_exn df ~by:Expr.[ col "c" ] ~agg:Expr.[ count_ () ]
      - : Data_frame0.t =
      shape: (2, 2)
      +-----+-------+
      | c   | count |
      | --- | ---   |
      | str | u32   |
      +=============+
      | foo | 2     |
      | bar | 1     |
      +-----+-------+
    ]} *)
val count_ : unit -> t

val n_unique : t -> t
val approx_n_unique : t -> t

(** Get unique values of this expression *)
val unique : t -> t

(** Get unique values of this expression, while maintaining order. This requires more work than [unique] *)
val unique_stable : t -> t

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
val is_finite : t -> t
val is_infinite : t -> t
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
val abs : t -> t
val exp : t -> t
val log : ?base:float -> t -> t
val log1p : t -> t

module Dt : sig
  val strftime : t -> format:string -> t

  (* CR-someday mtakeda: it's pretty weird that we take a string here when there's a
     proper type [Tz.t] that can be passed instead. This is due to the Rust library
     API also taking a string here, and underlying data types also using a
     [Option<String>] to represent timezones:
     https://docs.rs/polars/latest/polars/prelude/dt/struct.DateLikeNameSpace.html#method.convert_time_zone

     Perhaps we could make the OCaml API more typesafe by taking [Tz.t] here and
     converting it to a string on the Rust side. *)
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

  (** Get the (local) date of a Date/Datetime *)
  val date : t -> t

  (** Get the (local) time of a Date/Datetime/Time *)
  val time : t -> t
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
