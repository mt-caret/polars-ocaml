open! Core

type t

let quickcheck_shrinker = Base_quickcheck.Shrinker.atomic

let quickcheck_observer =
  Base_quickcheck.Observer.of_hash_fold (fun hash_state _t -> hash_state)
;;

external get_categories : t -> string list = "rust_rev_mapping_get_categories"

let sexp_of_t t =
  get_categories t
  |> (* It's difficult to get tests to be deterministic wrt the ordering of
        categories, so we force them to be sorted here as a hacky workaround. *)
  List.sort ~compare:String.compare
  |> [%sexp_of: string list]
;;
