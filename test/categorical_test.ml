open Core
open Polars

let%expect_test "Rev_mapping.get_categories returns categories in order they are \
                 encountered"
  =
  let s =
    Series.stringo "" [ Some "foo"; None; Some "bar"; Some "ham" ]
    |> Series.cast ~to_:(Categorical None)
  in
  let rev_mapping =
    match Series.dtype s with
    | Categorical (Some rev_mapping) -> rev_mapping
    | _ -> failwith "unexpected"
  in
  Rev_mapping.get_categories rev_mapping |> [%sexp_of: string list] |> print_s;
  [%expect {| (foo bar ham) |}]
;;

let%expect_test "Test csv parsing with schema including categorical types" =
  let schema =
    Schema.create
      [ "gender", Categorical None
      ; "type", Categorical None
      ; "state", Categorical None
      ; "party", Categorical None
      ; "birthday", Date
      ]
  in
  let dataset =
    Data_frame.read_csv_exn
      ~schema
      ~try_parse_dates:true
      "../guide/data/legislators-historical.csv"
  in
  Data_frame.print dataset;
  [%expect
    {|
    shape: (12_136, 36)
    ┌────────────┬────────────┬────────────┬────────┬───┬───────────┬───────────┬──────────┬───────────┐
    │ last_name  ┆ first_name ┆ middle_nam ┆ suffix ┆ … ┆ ballotped ┆ washingto ┆ icpsr_id ┆ wikipedia │
    │ ---        ┆ ---        ┆ e          ┆ ---    ┆   ┆ ia_id     ┆ n_post_id ┆ ---      ┆ _id       │
    │ str        ┆ str        ┆ ---        ┆ str    ┆   ┆ ---       ┆ ---       ┆ i64      ┆ ---       │
    │            ┆            ┆ str        ┆        ┆   ┆ str       ┆ str       ┆          ┆ str       │
    ╞════════════╪════════════╪════════════╪════════╪═══╪═══════════╪═══════════╪══════════╪═══════════╡
    │ Bassett    ┆ Richard    ┆ null       ┆ null   ┆ … ┆ null      ┆ null      ┆ 507      ┆ Richard   │
    │            ┆            ┆            ┆        ┆   ┆           ┆           ┆          ┆ Bassett   │
    │            ┆            ┆            ┆        ┆   ┆           ┆           ┆          ┆ (Delaware │
    │            ┆            ┆            ┆        ┆   ┆           ┆           ┆          ┆ politi…   │
    │ Bland      ┆ Theodorick ┆ null       ┆ null   ┆ … ┆ null      ┆ null      ┆ 786      ┆ Theodoric │
    │            ┆            ┆            ┆        ┆   ┆           ┆           ┆          ┆ k Bland   │
    │            ┆            ┆            ┆        ┆   ┆           ┆           ┆          ┆ (congress │
    │            ┆            ┆            ┆        ┆   ┆           ┆           ┆          ┆ man)      │
    │ Burke      ┆ Aedanus    ┆ null       ┆ null   ┆ … ┆ null      ┆ null      ┆ 1260     ┆ Aedanus   │
    │            ┆            ┆            ┆        ┆   ┆           ┆           ┆          ┆ Burke     │
    │ Carroll    ┆ Daniel     ┆ null       ┆ null   ┆ … ┆ null      ┆ null      ┆ 1538     ┆ Daniel    │
    │            ┆            ┆            ┆        ┆   ┆           ┆           ┆          ┆ Carroll   │
    │ …          ┆ …          ┆ …          ┆ …      ┆ … ┆ …         ┆ …         ┆ …        ┆ …         │
    │ Flores     ┆ Mayra      ┆ null       ┆ null   ┆ … ┆ Mayra     ┆ null      ┆ null     ┆ Mayra     │
    │            ┆            ┆            ┆        ┆   ┆ Flores    ┆           ┆          ┆ Flores    │
    │ Sempolinsk ┆ Joseph     ┆ null       ┆ null   ┆ … ┆ Joe Sempo ┆ null      ┆ null     ┆ Joe Sempo │
    │ i          ┆            ┆            ┆        ┆   ┆ linski    ┆           ┆          ┆ linski    │
    │ Inhofe     ┆ James      ┆ M.         ┆ null   ┆ … ┆ Jim       ┆ null      ┆ 15424    ┆ Jim       │
    │            ┆            ┆            ┆        ┆   ┆ Inhofe    ┆           ┆          ┆ Inhofe    │
    │ Sasse      ┆ Benjamin   ┆ Eric       ┆ null   ┆ … ┆ Ben Sasse ┆ null      ┆ 41503    ┆ Ben Sasse │
    └────────────┴────────────┴────────────┴────────┴───┴───────────┴───────────┴──────────┴───────────┘ |}];
  Data_frame.schema dataset |> [%sexp_of: Schema.t] |> print_s;
  [%expect
    {|
    ((last_name Utf8) (first_name Utf8) (middle_name Utf8) (suffix Utf8)
     (nickname Utf8) (full_name Utf8) (birthday Date)
     (gender (Categorical ((F M)))) (type (Categorical ((rep sen))))
     (state
      (Categorical
       ((AK AL AR AS AZ CA CO CT DC DE DK FL GA GU HI IA ID IL IN KS KY LA MA MD
         ME MI MN MO MS MT NC ND NE NH NJ NM NV NY OH OK OL OR PA PI PR RI SC SD
         TN TX UT VA VI VT WA WI WV WY))))
     (district Int64) (senate_class Int64)
     (party
      (Categorical
       ((Adams "Adams Democrat" American "American Labor" "Anti Jackson"
         "Anti Jacksonian" "Anti Masonic" Anti-Administration Anti-Jacksonian
         "Anti-Lecompton Democrat" Coalitionist Conservative
         "Conservative Republican" "Constitutional Unionist"
         "Crawford Republican" Democrat Democrat-Liberal "Democratic Republican"
         Farmer-Labor Federalist "Free Silver" "Free Soil" "Ind. Democrat"
         "Ind. Republican" "Ind. Republican-Democrat" "Ind. Whig" Independent
         "Independent Democrat" Jackson "Jackson Republican" Jacksonian
         "Law and Order" "Liberal Republican" Libertarian Liberty
         "National Greenbacker" "New Progressive" Nonpartisan Nullifier
         "Popular Democrat" Populist Pro-Administration Progressive
         "Progressive Republican" Prohibitionist Readjuster "Readjuster Democrat"
         Republican Republican-Conservative "Silver Republican" Socialist
         "States Rights" "Unconditional Unionist" Union "Union Democrat"
         "Union Labor" Unionist Unknown Whig))))
     (url Utf8) (address Utf8) (phone Utf8) (contact_form Utf8) (rss_url Utf8)
     (twitter Utf8) (twitter_id Utf8) (facebook Utf8) (youtube Utf8)
     (youtube_id Utf8) (mastodon Utf8) (bioguide_id Utf8) (thomas_id Utf8)
     (opensecrets_id Utf8) (lis_id Utf8) (fec_ids Utf8) (cspan_id Utf8)
     (govtrack_id Int64) (votesmart_id Utf8) (ballotpedia_id Utf8)
     (washington_post_id Utf8) (icpsr_id Int64) (wikipedia_id Utf8)) |}]
;;
