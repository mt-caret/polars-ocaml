open Core

let string_result_ok_exn string_result =
  Result.map_error string_result ~f:Error.of_string |> Or_error.ok_exn
;;
