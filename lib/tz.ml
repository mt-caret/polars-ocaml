open! Core

type t

external all : unit -> t list = "rust_all_tzs"
external to_string : t -> string = "rust_tz_to_string"
external parse : string -> t option = "rust_tz_parse"

let%expect_test "sample" =
  List.take (all ()) 5 |> List.iter ~f:(fun t -> print_endline (to_string t));
  [%expect
    {|
    Africa/Abidjan
    Africa/Accra
    Africa/Addis_Ababa
    Africa/Algiers
    Africa/Asmara |}]
;;

let compare = Comparable.lift [%compare: string] ~f:to_string

include Sexpable.Of_stringable (struct
    type nonrec t = t

    let to_string = to_string
    let of_string str = parse str |> Option.value_exn ~here:[%here] ~message:"Invalid tz"
  end)

let%expect_test "roundtrip" =
  List.iter (all ()) ~f:(fun tz ->
    to_string tz |> parse |> [%test_result: t option] ~expect:(Some tz))
;;

let quickcheck_generator = Quickcheck.Generator.of_list (all ())
let quickcheck_shrinker = Quickcheck.Shrinker.empty ()
let quickcheck_observer = Quickcheck.Observer.of_list (all ()) ~equal:[%compare.equal: t]
let of_time_zone time_zone = Time_ns_unix.Zone.to_string time_zone |> parse
let to_time_zone t = Time_ns_unix.Zone.find (to_string t)
