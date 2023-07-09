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

  module Naive_date : sig
    type t

    external create : year:int -> month:int -> day:int -> t option = "rust_naive_date"
    val of_date : Date.t -> t
    val of_string : string -> t
  end

  module Naive_datetime : sig
    type t

    val of_naive_date : ?hour:int -> ?min:int -> ?sec:int -> Naive_date.t -> t
    val of_date : ?hour:int -> ?min:int -> ?sec:int -> Date.t -> t
    val of_string : string -> t
  end

  module For_testing : sig
    external panic : string -> unit = "rust_test_panic"
    external raise_exception : string -> unit = "rust_test_exception"

    external install_panic_hook
      :  suppress_backtrace:bool
      -> unit
      = "rust_install_panic_hook"
  end
end
