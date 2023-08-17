open! Core
open Async
include Polars.Lazy_frame

let collect ?(streaming = false) t = In_thread.run (fun () -> collect t ~streaming)

let collect_exn ?streaming t =
  collect ?streaming t >>| Result.map_error ~f:Error.of_string >>| Or_error.ok_exn
;;

let collect_all t = In_thread.run (fun () -> collect_all t)

let collect_all_exn ts =
  collect_all ts >>| Result.map_error ~f:Error.of_string >>| Or_error.ok_exn
;;

let profile t = In_thread.run (fun () -> profile t)
let profile_exn t = profile t >>| Result.map_error ~f:Error.of_string >>| Or_error.ok_exn
let fetch t ~n_rows = In_thread.run (fun () -> fetch t ~n_rows)

let fetch_exn t ~n_rows =
  fetch t ~n_rows >>| Result.map_error ~f:Error.of_string >>| Or_error.ok_exn
;;
