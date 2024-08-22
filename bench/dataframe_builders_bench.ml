open! Core
open Polars

let create_basic_df () =
  let integer = Series.int "integer" [ 1 ] in
  let float = Series.float "float" [ 4. ] in
  let df = Data_frame.create_exn [ integer; float ] in
  Series.clear integer;
  Series.clear float;
  df
;;

(* Building a dataframe using ocaml-interop can be tricky, and there are ways to do it
   that result in O(n^2) runtime due to the way that dataframe refcounting works.
   The following function is intended to be close to linear runtime: vstack
   dataframes one by one, and then call as_single_chunk_par once at the end.
*)
let build_a_dataframe_usage n =
  let df = Data_frame.create_exn Series.[ int "integer" []; float "float" [] ] in
  let single_row = create_basic_df () in
  List.init n ~f:Fn.id
  |> List.iter ~f:(fun i ->
    Data_frame.vstack_exn df ~other:single_row;
    if i % 100 = 0
    then (
      let (_ : Data_frame.t) =
        Sql_context.execute_with_data_frames_exn
          ~names_and_data_frames:[ "data", df ]
          ~query:"select * from data limit 1"
      in
      ()));
  Data_frame.as_single_chunk_par df;
  let count =
    Sql_context.execute_with_data_frames_exn
      ~names_and_data_frames:[ "data", df ]
      ~query:"select count(*), max(integer) from data"
  in
  ignore (count : Data_frame.t)
;;

let%bench "build and periodically query a dataframe -- 100_000 iters" =
  build_a_dataframe_usage 100_000
;;

let%bench "build and periodically query a dataframe -- 1_000_000 iters" =
  build_a_dataframe_usage 1_000_000
;;

let%bench "build and periodically query a dataframe -- 10_000_000 iters" =
  build_a_dataframe_usage 10_000_000
;;

(* The following function is also intended to be close to linear runtime: use a helper
   dataframe to batch rows into chunks of size 1000, and vstack each chunk one at a
   time to [df]
*)
let one_chunk_at_a_time_usage n =
  let df = Data_frame.create_exn Series.[ int "integer" []; float "float" [] ] in
  let df2 = ref (Data_frame.create_exn Series.[ int "integer" []; float "float" [] ]) in
  let single_row = create_basic_df () in
  List.init n ~f:Fn.id
  |> List.iter ~f:(fun i ->
    Data_frame.vstack_exn !df2 ~other:single_row;
    if i % 1000 = 0
    then (
      Data_frame.as_single_chunk_par !df2;
      Data_frame.vstack_exn df ~other:!df2;
      df2 := Data_frame.clear !df2));
  let count =
    Sql_context.execute_with_data_frames_exn
      ~names_and_data_frames:[ "data", df ]
      ~query:"select count(*), max(integer) from data"
  in
  ignore (count : Data_frame.t)
;;

let%bench "one chunk at a time -- 100_000 iters" = build_a_dataframe_usage 100_000
let%bench "one chunk at a time -- 1_000_000 iters" = build_a_dataframe_usage 1_000_000
let%bench "one chunk at a time -- 10_000_000 iters" = build_a_dataframe_usage 10_000_000

let create_data_frame () =
  let series = Series.int "integer" (List.init 1000 ~f:Fn.id) in
  let df = Data_frame.create_exn [ series ] in
  Series.clear series;
  df
;;

let modify_one_value_at_a_time n =
  let df = create_data_frame () in
  fun () ->
    List.init n ~f:Fn.id
    |> List.iter ~f:(fun i ->
      let len = Data_frame.height df in
      Data_frame.Expert.modify_series_at_chunk_index
        df
        ~dtype:Data_type.Typed.Int64
        ~series_index:0
        ~chunk_index:0
        ~indices_and_values:[ i % len, 1000 + i ]
      |> Result.ok_or_failwith);
    let count =
      Sql_context.execute_with_data_frames_exn
        ~names_and_data_frames:[ "data", df ]
        ~query:"select count(*), max(integer) from data"
    in
    ignore (count : Data_frame.t)
;;

let%bench_fun "modify one value at a time -- 100_000 iters" =
  modify_one_value_at_a_time 100_000
;;

let%bench_fun "modify one value at a time -- 1_000_000 iters" =
  modify_one_value_at_a_time 1_000_000
;;

let create_data_frame_optional () =
  let series = Series.into "integer" (List.init 1000 ~f:(fun x -> Some x)) in
  let df = Data_frame.create_exn [ series ] in
  Series.clear series;
  df
;;

let modify_one_value_at_a_time_optional n =
  let df = create_data_frame_optional () in
  let len = Data_frame.height df in
  fun () ->
    List.init n ~f:Fn.id
    |> List.iter ~f:(fun i ->
      Data_frame.Expert.modify_optional_series_at_chunk_index
        df
        ~dtype:Data_type.Typed.Int64
        ~series_index:0
        ~chunk_index:0
        ~indices_and_values:[ i % len, Some (1000 + i) ]
      |> Result.ok_or_failwith)
;;

let%bench_fun "modify one value at a time optional -- 100_000 iters" =
  modify_one_value_at_a_time_optional 100_000
;;

let%bench_fun "modify one value at a time optional -- 1_000_000 iters" =
  modify_one_value_at_a_time_optional 1_000_000
;;

let modify_one_chunk_at_a_time n =
  let df = create_data_frame () in
  let len = Data_frame.height df in
  let indices_and_values = List.init n ~f:(fun i -> i % len, 1000 + i) in
  fun () ->
    Data_frame.Expert.modify_series_at_chunk_index
      df
      ~dtype:Data_type.Typed.Int64
      ~series_index:0
      ~chunk_index:0
      ~indices_and_values
    |> Result.ok_or_failwith
;;

let%bench_fun "modify one chunk at a time -- 100_000 iters" =
  modify_one_chunk_at_a_time 100_000
;;

let%bench_fun "modify one chunk at a time -- 1_000_000 iters" =
  modify_one_chunk_at_a_time 1_000_000
;;

let modify_one_chunk_at_a_time_optional n =
  let df = create_data_frame_optional () in
  let len = Data_frame.height df in
  let indices_and_values = List.init n ~f:(fun i -> i % len, Some (1000 + i)) in
  fun () ->
    Data_frame.Expert.modify_optional_series_at_chunk_index
      df
      ~dtype:Data_type.Typed.Int64
      ~series_index:0
      ~chunk_index:0
      ~indices_and_values
    |> Result.ok_or_failwith
;;

let%bench_fun "modify one chunk at a time optional -- 100_000 iters" =
  modify_one_chunk_at_a_time_optional 100_000
;;

let%bench_fun "modify one chunk at a time optional -- 1_000_000 iters" =
  modify_one_chunk_at_a_time_optional 1_000_000
;;

(* Maintain a dataset as two dataframes. df only contains chunks of length 1000 and
   df2 only contains chunks of length 1.

   This test performs multiple mutations and clones interleaved with each other.
   As long as we don't do both at the same time, this should be about O(n) runtime.
*)

let interleave_vstack_and_update n =
  let total_length = ref 2 in
  let df = create_basic_df () in
  let df2 = ref (create_basic_df ()) in
  fun () ->
    List.init n ~f:Fn.id
    |> List.iter ~f:(fun _ ->
      (* Step 1: append a row *)
      let next_row = create_basic_df () in
      Data_frame.vstack_exn !df2 ~other:next_row;
      total_length := !total_length + 1;
      Data_frame.Expert.clear_mut next_row;
      let len = Data_frame.height !df2 in
      if len >= 1000
      then (
        Data_frame.as_single_chunk_par !df2;
        Data_frame.vstack_exn df ~other:!df2;
        Data_frame.Expert.clear_mut !df2;
        total_length := !total_length + 1;
        df2 := create_basic_df ());
      (* Step 2: modify df *)
      Data_frame.Expert.modify_optional_series_at_chunk_index
        df
        ~dtype:Data_type.Typed.Int64
        ~series_index:0
        ~chunk_index:0
        ~indices_and_values:[ 0, Some 1000 ]
      |> Result.ok_or_failwith;
      (* Step 3: modify df2 *)
      Data_frame.Expert.modify_optional_series_at_chunk_index
        !df2
        ~dtype:Data_type.Typed.Int64
        ~series_index:0
        ~chunk_index:0
        ~indices_and_values:[ 0, Some 1000 ]
      |> Result.ok_or_failwith;
      (* Step 4: vstack and collect *)
      if len % 123 = 0
      then (
        let count =
          Sql_context.vstack_and_collect
            ~names_and_data_frames:[ "data", [ df; !df2 ] ]
            ~query:"select count(*) as count from data"
          |> Result.ok_or_failwith
        in
        let count =
          Data_frame.column_exn count ~name:"count"
          |> Series.to_list UInt32
          |> List.hd_exn
        in
        assert (count = !total_length)))
;;

let%bench_fun "vstack_and_update -- 10_000 iters" = interleave_vstack_and_update 10_000
let%bench_fun "vstack_and_update -- 100_000 iters" = interleave_vstack_and_update 100_000
