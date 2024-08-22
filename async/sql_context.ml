open! Core
open Async
include Polars.Sql_context

let execute_with_data_frames ~names_and_data_frames ~query =
  In_thread.run (fun () -> execute_with_data_frames ~names_and_data_frames ~query)
;;

let execute_with_data_frames_exn ~names_and_data_frames ~query =
  In_thread.run (fun () -> execute_with_data_frames_exn ~names_and_data_frames ~query)
;;

let vstack_and_collect ~names_and_data_frames ~query =
  In_thread.run (fun () -> vstack_and_collect ~names_and_data_frames ~query)
;;

let execute t ~query = In_thread.run (fun () -> execute t ~query)
let execute_exn t ~query = In_thread.run (fun () -> execute_exn t ~query)
