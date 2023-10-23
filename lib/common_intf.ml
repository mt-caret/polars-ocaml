open! Core

module type Compare = sig
  type t

  val ( = ) : t -> t -> t
  val ( <> ) : t -> t -> t
  val ( > ) : t -> t -> t
  val ( >= ) : t -> t -> t
  val ( < ) : t -> t -> t
  val ( <= ) : t -> t -> t
end

module type Numeric_basic = sig
  type t

  val add : t -> t -> t
  val sub : t -> t -> t
  val mul : t -> t -> t
  val div : t -> t -> t
end

module type Numeric = sig
  include Numeric_basic

  val ( + ) : t -> t -> t
  val ( - ) : t -> t -> t
  val ( * ) : t -> t -> t
  val ( / ) : t -> t -> t
end

module type Logic_basic = sig
  type t

  val not : t -> t
  val and_ : t -> t -> t
  val or_ : t -> t -> t
  val xor : t -> t -> t
end

module type Logic = sig
  include Logic_basic

  val ( ! ) : t -> t
  val ( && ) : t -> t -> t
  val ( || ) : t -> t -> t
  val ( lxor ) : t -> t -> t
end

module type Common = sig
  module type Compare = Compare
  module type Numeric = Numeric
  module type Logic = Logic

  module Make_numeric (T : Numeric_basic) : Numeric with type t := T.t
  module Make_logic (T : Logic_basic) : Logic with type t := T.t

  val record_panic_backtraces : unit -> unit

  module For_testing : sig
    val panic : string -> unit

    (** [clear_panic_hook] sets the panic handler to a no-op. We've found that
        the output of the default panic hook does not seem to be stable across
        Rust versions, so this should be used in expect tests where we expect
        panic-driven exceptions. *)
    val clear_panic_hook : unit -> unit
  end
end
