open! Core

module Expr = struct
  type t

  external col : string -> t = "rust_expr_col"
  external int : int -> t = "rust_expr_int"
  external float : float -> t = "rust_expr_float"
  external sort : t -> descending:bool -> t = "rust_expr_sort"
  external head : t -> length:int option -> t = "rust_expr_head"
  external filter : t -> predicate:t -> t = "rust_expr_filter"
  external sum : t -> t = "rust_expr_sum"
  external alias : t -> name:string -> t = "rust_expr_alias"
  external equal : t -> t -> t = "rust_expr_eq"

  let ( = ) = equal

  external ( <> ) : t -> t -> t = "rust_expr_neq"
  external ( > ) : t -> t -> t = "rust_expr_gt"
  external ( >= ) : t -> t -> t = "rust_expr_gt_eq"
  external ( < ) : t -> t -> t = "rust_expr_lt"
  external ( <= ) : t -> t -> t = "rust_expr_lt_eq"
  external not : t -> t = "rust_expr_not"

  let ( ! ) = not

  external and_ : t -> t -> t = "rust_expr_and"
  external or_ : t -> t -> t = "rust_expr_or"
  external xor : t -> t -> t = "rust_expr_xor"

  let ( && ) = and_
  let ( || ) = or_
  let ( lxor ) = xor

  external add : t -> t -> t = "rust_expr_add"
  external sub : t -> t -> t = "rust_expr_sub"
  external mul : t -> t -> t = "rust_expr_mul"
  external div : t -> t -> t = "rust_expr_div"

  let ( + ) = add
  let ( - ) = sub
  let ( * ) = mul
  let ( / ) = div
end

module Series = struct
  type t

  external int : string -> int list -> t = "rust_series_new_int"
  external int_option : string -> int option list -> t = "rust_series_new_int_option"
  external float : string -> float list -> t = "rust_series_new_float"

  external float_option
    :  string
    -> float option list
    -> t
    = "rust_series_new_float_option"

  external string : string -> string list -> t = "rust_series_new_string"

  external string_option
    :  string
    -> string option list
    -> t
    = "rust_series_new_string_option"

  external to_string_hum : t -> string = "rust_series_to_string_hum"

  let print t = print_endline (to_string_hum t)
end

module Data_frame = struct
  type t

  external create : Series.t list -> (t, string) result = "rust_data_frame_new"

  let create_exn series =
    create series |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn
  ;;

  external to_string_hum : t -> string = "rust_data_frame_to_string_hum"

  let print t = print_endline (to_string_hum t)
end

module Lazy_frame = struct
  type t

  external scan_parquet : string -> (t, string) result = "rust_lazy_frame_scan_parquet"
  external of_data_frame : Data_frame.t -> t = "rust_data_frame_lazy"
  external to_dot : t -> (string, string) result = "rust_lazy_frame_to_dot"
  external collect : t -> (Data_frame.t, string) result = "rust_lazy_frame_collect"

  let collect_exn t = collect t |> Result.map_error ~f:Error.of_string |> Or_error.ok_exn

  external select : t -> exprs:Expr.t list -> t = "rust_lazy_frame_select"
end

(* TODO: what's an ergonomic way to create series and dataframes easily? *)
let%expect_test "examples" =
  let r = Random.State.make [||] in
  let df =
    Data_frame.create_exn
      [ Series.int_option "nrs" [ Some 1; Some 2; Some 3; None; Some 5 ]
      ; Series.string_option
          "names"
          [ Some "foo"; Some "ham"; Some "spam"; Some "egg"; None ]
      ; Series.float "random" (List.init 5 ~f:(fun _ -> Random.State.float r 5.))
      ; Series.string "groups" [ "A"; "A"; "B"; "C"; "B" ]
      ]
  in
  Data_frame.print df;
  [%expect
    {|
    shape: (5, 4)
    ┌──────┬───────┬──────────┬────────┐
    │ nrs  ┆ names ┆ random   ┆ groups │
    │ ---  ┆ ---   ┆ ---      ┆ ---    │
    │ i64  ┆ str   ┆ f64      ┆ str    │
    ╞══════╪═══════╪══════════╪════════╡
    │ 1    ┆ foo   ┆ 1.848939 ┆ A      │
    │ 2    ┆ ham   ┆ 4.490401 ┆ A      │
    │ 3    ┆ spam  ┆ 3.147566 ┆ B      │
    │ null ┆ egg   ┆ 0.156988 ┆ C      │
    │ 5    ┆ null  ┆ 0.831802 ┆ B      │
    └──────┴───────┴──────────┴────────┘ |}];
  let df_numerical =
    Lazy_frame.of_data_frame df
    |> Lazy_frame.select
         ~exprs:
           Expr.
             [ col "nrs" + int 5 |> alias ~name:"nrs + 5"
             ; col "nrs" - int 5 |> alias ~name:"nrs - 5"
             ; col "nrs" * col "random" |> alias ~name:"nrs * random"
             ; col "nrs" / col "random" |> alias ~name:"nrs / random"
             ]
    |> Lazy_frame.collect_exn
  in
  Data_frame.print df_numerical;
  [%expect
    {|
    shape: (5, 4)
    ┌─────────┬─────────┬──────────────┬──────────────┐
    │ nrs + 5 ┆ nrs - 5 ┆ nrs * random ┆ nrs / random │
    │ ---     ┆ ---     ┆ ---          ┆ ---          │
    │ i64     ┆ i64     ┆ f64          ┆ f64          │
    ╞═════════╪═════════╪══════════════╪══════════════╡
    │ 6       ┆ -4      ┆ 1.848939     ┆ 0.540851     │
    │ 7       ┆ -3      ┆ 8.980802     ┆ 0.445394     │
    │ 8       ┆ -2      ┆ 9.442697     ┆ 0.953117     │
    │ null    ┆ null    ┆ null         ┆ null         │
    │ 10      ┆ 0       ┆ 4.159012     ┆ 6.011044     │
    └─────────┴─────────┴──────────────┴──────────────┘ |}];
  let df_logical =
    Lazy_frame.of_data_frame df
    |> Lazy_frame.select
         ~exprs:
           Expr.
             [ col "nrs" > int 1 |> alias ~name:"nrs > 1"
             ; col "random" <= float 0.5 |> alias ~name:"random <= 0.5"
             ; col "nrs" <> int 1 |> alias ~name:"nrs != 1"
             ; col "nrs" = int 1 |> alias ~name:"nrs == 1"
             ; (col "random" <= float 0.5 && col "nrs" > int 1) |> alias ~name:"and_expr"
             ; (col "random" <= float 0.5 || col "nrs" > int 1) |> alias ~name:"or_expr"
             ]
    |> Lazy_frame.collect_exn
  in
  Data_frame.print df_logical;
  [%expect
    {|
    shape: (5, 6)
    ┌─────────┬───────────────┬──────────┬──────────┬──────────┬─────────┐
    │ nrs > 1 ┆ random <= 0.5 ┆ nrs != 1 ┆ nrs == 1 ┆ and_expr ┆ or_expr │
    │ ---     ┆ ---           ┆ ---      ┆ ---      ┆ ---      ┆ ---     │
    │ bool    ┆ bool          ┆ bool     ┆ bool     ┆ bool     ┆ bool    │
    ╞═════════╪═══════════════╪══════════╪══════════╪══════════╪═════════╡
    │ false   ┆ false         ┆ false    ┆ true     ┆ false    ┆ false   │
    │ true    ┆ false         ┆ true     ┆ false    ┆ false    ┆ true    │
    │ true    ┆ false         ┆ true     ┆ false    ┆ false    ┆ true    │
    │ null    ┆ true          ┆ null     ┆ null     ┆ null     ┆ true    │
    │ true    ┆ false         ┆ true     ┆ false    ┆ false    ┆ true    │
    └─────────┴───────────────┴──────────┴──────────┴──────────┴─────────┘ |}]
;;
