open! Core
include Common_intf

module Make_numeric (T : Numeric_basic) = struct
  include T

  let ( + ) = add
  let ( - ) = sub
  let ( * ) = mul
  let ( / ) = div
end

module Make_logic (T : Logic_basic) = struct
  include T

  let ( ! ) = not
  let ( && ) = and_
  let ( || ) = or_
  let ( lxor ) = xor
end

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

  let of_naive_date naive_date =
    of_naive_date naive_date |> Option.value_exn ~here:[%here]
  ;;

  let of_date date = Naive_date.of_date date |> of_naive_date
end

module For_testing = struct
  external panic : string -> unit = "rust_test_panic"
  external raise_exception : string -> unit = "rust_test_exception"

  external install_panic_hook
    :  suppress_backtrace:bool
    -> unit
    = "rust_install_panic_hook"
end
