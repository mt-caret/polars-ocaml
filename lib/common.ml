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

external record_panic_backtraces : unit -> unit = "rust_record_panic_backtraces"

module For_testing = struct
  external panic : string -> unit = "rust_test_panic"
  external clear_panic_hook : unit -> unit = "rust_clear_panic_hook"
end
