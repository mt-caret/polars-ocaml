open! Core

type t

external create : year:int -> month:int -> day:int -> t option = "rust_naive_date"

let of_date date =
  create
    ~year:(Date.year date)
    ~month:(Date.month date |> Month.to_int)
    ~day:(Date.day date)
  |> Option.value_exn
       ~here:[%here]
       ~error:(Error.create_s [%message "Unexpected invalid date" (date : Date.t)])
;;

external to_ocaml : t -> int * int * int = "rust_naive_date_to_ocaml"

let to_date_exn t =
  let year, month, day = to_ocaml t in
  Date.create_exn ~y:year ~m:(Month.of_int_exn month) ~d:day
;;

let%expect_test "roundtrip" =
  Quickcheck.test Date.quickcheck_generator ~f:(fun date ->
    of_date date |> to_date_exn |> [%test_result: Date.t] ~expect:date)
;;

let of_string str = Date.of_string str |> of_date
