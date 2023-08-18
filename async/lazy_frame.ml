open! Core
open Async
include Polars.Lazy_frame

let collect ?streaming t = In_thread.run (fun () -> collect ?streaming t)
let collect_exn ?streaming t = In_thread.run (fun () -> collect_exn ?streaming t)
let collect_all t = In_thread.run (fun () -> collect_all t)
let collect_all_exn ts = In_thread.run (fun () -> collect_all_exn ts)
let profile t = In_thread.run (fun () -> profile t)
let profile_exn t = In_thread.run (fun () -> profile_exn t)
let fetch t ~n_rows = In_thread.run (fun () -> fetch t ~n_rows)
let fetch_exn t ~n_rows = In_thread.run (fun () -> fetch_exn t ~n_rows)
