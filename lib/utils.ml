open! Core

module Naive_date = struct
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
end

module Naive_datetime = struct
  type t

  external of_naive_date : Naive_date.t -> t option = "rust_naive_date_to_naive_datetime"
end
