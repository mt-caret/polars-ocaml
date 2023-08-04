open! Core
open! Polars

(* Examples from https://pola-rs.github.io/polars-book/user-guide/sql/intro/ *)
let%expect_test "Introduction" =
  let create_lazy_frame columns = Data_frame.create_exn columns |> Data_frame.lazy_ in
  let a = create_lazy_frame Series.[ int "a" [ 1; 2; 3 ] ] in
  let b = create_lazy_frame Series.[ int "b" [ 4; 5; 6 ] ] in
  let ctx = Sql_context.create [ "a", a; "b", b ] in
  Sql_context.get_tables ctx |> [%sexp_of: string list] |> print_s;
  [%expect {|
    (a b) |}];
  let pokemon = Lazy_frame.scan_csv_exn "./data/pokemon.csv" in
  let ctx = Sql_context.create [ "pokemon", pokemon ] in
  Sql_context.execute_exn ctx ~query:"SELECT * from pokemon LIMIT 5"
  |> Lazy_frame.collect_exn
  |> Data_frame.print;
  [%expect
    {|
    shape: (5, 13)
    ┌─────┬───────────────────────┬────────┬────────┬───┬─────────┬───────┬────────────┬───────────┐
    │ #   ┆ Name                  ┆ Type 1 ┆ Type 2 ┆ … ┆ Sp. Def ┆ Speed ┆ Generation ┆ Legendary │
    │ --- ┆ ---                   ┆ ---    ┆ ---    ┆   ┆ ---     ┆ ---   ┆ ---        ┆ ---       │
    │ i64 ┆ str                   ┆ str    ┆ str    ┆   ┆ i64     ┆ i64   ┆ i64        ┆ bool      │
    ╞═════╪═══════════════════════╪════════╪════════╪═══╪═════════╪═══════╪════════════╪═══════════╡
    │ 1   ┆ Bulbasaur             ┆ Grass  ┆ Poison ┆ … ┆ 65      ┆ 45    ┆ 1          ┆ false     │
    │ 2   ┆ Ivysaur               ┆ Grass  ┆ Poison ┆ … ┆ 80      ┆ 60    ┆ 1          ┆ false     │
    │ 3   ┆ Venusaur              ┆ Grass  ┆ Poison ┆ … ┆ 100     ┆ 80    ┆ 1          ┆ false     │
    │ 3   ┆ VenusaurMega Venusaur ┆ Grass  ┆ Poison ┆ … ┆ 120     ┆ 80    ┆ 1          ┆ false     │
    │ 4   ┆ Charmander            ┆ Fire   ┆ null   ┆ … ┆ 50      ┆ 65    ┆ 1          ┆ false     │
    └─────┴───────────────────────┴────────┴────────┴───┴─────────┴───────┴────────────┴───────────┘ |}];
  let ctx =
    Sql_context.create
      [ ( "products_masterdata"
        , create_lazy_frame
            Series.
              [ int "product_id" [ 1; 2; 3; 4; 5 ]
              ; string
                  "product_name"
                  [ "Product A"; "Product B"; "Product C"; "Product D"; "Product E" ]
              ] )
      ; ( "products_categories"
        , create_lazy_frame
            Series.
              [ int "product_id" [ 1; 2; 3; 4; 5 ]
              ; string
                  "category"
                  [ "Category 1"; "Category 1"; "Category 2"; "Category 2"; "Category 3" ]
              ] )
      ; ( "sales_data"
        , create_lazy_frame
            Series.
              [ int "product_id" [ 1; 2; 3; 4; 5 ]
              ; int "sales" [ 100; 200; 150; 250; 300 ]
              ] )
      ]
  in
  Sql_context.execute_exn
    ctx
    ~query:
      {|
  SELECT 
    product_id,
    product_name,
    category,
    sales
  FROM 
      products_masterdata
  LEFT JOIN products_categories USING (product_id)
  LEFT JOIN sales_data USING (product_id)|}
  |> Lazy_frame.collect_exn
  |> Data_frame.print;
  [%expect
    {|
    shape: (5, 4)
    ┌────────────┬──────────────┬────────────┬───────┐
    │ product_id ┆ product_name ┆ category   ┆ sales │
    │ ---        ┆ ---          ┆ ---        ┆ ---   │
    │ i64        ┆ str          ┆ str        ┆ i64   │
    ╞════════════╪══════════════╪════════════╪═══════╡
    │ 1          ┆ Product A    ┆ Category 1 ┆ 100   │
    │ 2          ┆ Product B    ┆ Category 1 ┆ 200   │
    │ 3          ┆ Product C    ┆ Category 2 ┆ 150   │
    │ 4          ┆ Product D    ┆ Category 2 ┆ 250   │
    │ 5          ┆ Product E    ┆ Category 3 ┆ 300   │
    └────────────┴──────────────┴────────────┴───────┘ |}]
;;

(* Examples from https://pola-rs.github.io/polars-book/user-guide/sql/show/ *)
let%expect_test "SHOW TABLES" =
  let create_lazy_frame columns = Data_frame.create_exn columns |> Data_frame.lazy_ in
  let df1 =
    create_lazy_frame
      Series.
        [ string "name" [ "Alice"; "Bob"; "Charlie"; "David" ]
        ; int "age" [ 25; 30; 35; 40 ]
        ]
  in
  let df2 =
    create_lazy_frame
      Series.
        [ string "name" [ "Ellen"; "Frank"; "Gina"; "Henry" ]
        ; int "age" [ 45; 50; 55; 60 ]
        ]
  in
  let ctx = Sql_context.create [ "mytable1", df1; "mytable2", df2 ] in
  Sql_context.execute_exn ctx ~query:"SHOW TABLES"
  |> Lazy_frame.collect_exn
  |> Data_frame.print;
  [%expect
    {|
    shape: (2, 1)
    ┌──────────┐
    │ name     │
    │ ---      │
    │ str      │
    ╞══════════╡
    │ mytable1 │
    │ mytable2 │
    └──────────┘
    |}]
;;

(* Examples from https://pola-rs.github.io/polars-book/user-guide/sql/select/ *)
let%expect_test "SELECT" =
  let create_lazy_frame columns = Data_frame.create_exn columns |> Data_frame.lazy_ in
  let df =
    create_lazy_frame
      Series.
        [ string
            "city"
            [ "New York"; "Los Angeles"; "Chicago"; "Houston"; "Phoenix"; "Amsterdam" ]
        ; string "country" [ "USA"; "USA"; "USA"; "USA"; "USA"; "Netherlands" ]
        ; int "population" [ 8399000; 3997000; 2705000; 2320000; 1680000; 900000 ]
        ]
  in
  let ctx = Sql_context.create [ "population", df ] in
  Sql_context.execute_exn ctx ~query:"SELECT * FROM population"
  |> Lazy_frame.collect_exn
  |> Data_frame.print;
  [%expect
    {|
    shape: (6, 3)
    ┌─────────────┬─────────────┬────────────┐
    │ city        ┆ country     ┆ population │
    │ ---         ┆ ---         ┆ ---        │
    │ str         ┆ str         ┆ i64        │
    ╞═════════════╪═════════════╪════════════╡
    │ New York    ┆ USA         ┆ 8399000    │
    │ Los Angeles ┆ USA         ┆ 3997000    │
    │ Chicago     ┆ USA         ┆ 2705000    │
    │ Houston     ┆ USA         ┆ 2320000    │
    │ Phoenix     ┆ USA         ┆ 1680000    │
    │ Amsterdam   ┆ Netherlands ┆ 900000     │
    └─────────────┴─────────────┴────────────┘ |}];
  Sql_context.execute_exn
    ctx
    ~query:
      {|
  SELECT country, AVG(population) as avg_population
  FROM population
  GROUP BY country|}
  |> Lazy_frame.collect_exn
  |> Data_frame.print;
  [%expect
    {|
    shape: (2, 2)
    ┌─────────────┬────────────────┐
    │ country     ┆ avg_population │
    │ ---         ┆ ---            │
    │ str         ┆ f64            │
    ╞═════════════╪════════════════╡
    │ USA         ┆ 3.8202e6       │
    │ Netherlands ┆ 900000.0       │
    └─────────────┴────────────────┘ |}];
  Sql_context.execute_exn
    ctx
    ~query:{|
  SELECT city, population
  FROM population
  ORDER BY population |}
  |> Lazy_frame.collect_exn
  |> Data_frame.print;
  [%expect
    {|
    shape: (6, 2)
    ┌─────────────┬────────────┐
    │ city        ┆ population │
    │ ---         ┆ ---        │
    │ str         ┆ i64        │
    ╞═════════════╪════════════╡
    │ Amsterdam   ┆ 900000     │
    │ Phoenix     ┆ 1680000    │
    │ Houston     ┆ 2320000    │
    │ Chicago     ┆ 2705000    │
    │ Los Angeles ┆ 3997000    │
    │ New York    ┆ 8399000    │
    └─────────────┴────────────┘ |}];
  let income =
    create_lazy_frame
      Series.
        [ string
            "city"
            [ "New York"
            ; "Los Angeles"
            ; "Chicago"
            ; "Houston"
            ; "Amsterdam"
            ; "Rotterdam"
            ; "Utrecht"
            ]
        ; string
            "country"
            [ "USA"; "USA"; "USA"; "USA"; "Netherlands"; "Netherlands"; "Netherlands" ]
        ; int "income" [ 55000; 62000; 48000; 52000; 42000; 38000; 41000 ]
        ]
  in
  let ctx = Sql_context.create [ "population", df; "income", income ] in
  Sql_context.execute_exn
    ctx
    ~query:
      {|
  SELECT country, city, income, population
  FROM population
  LEFT JOIN income on population.city = income.city|}
  |> Lazy_frame.collect_exn
  |> Data_frame.print;
  [%expect
    {|
    shape: (6, 4)
    ┌─────────────┬─────────────┬────────┬────────────┐
    │ country     ┆ city        ┆ income ┆ population │
    │ ---         ┆ ---         ┆ ---    ┆ ---        │
    │ str         ┆ str         ┆ i64    ┆ i64        │
    ╞═════════════╪═════════════╪════════╪════════════╡
    │ USA         ┆ New York    ┆ 55000  ┆ 8399000    │
    │ USA         ┆ Los Angeles ┆ 62000  ┆ 3997000    │
    │ USA         ┆ Chicago     ┆ 48000  ┆ 2705000    │
    │ USA         ┆ Houston     ┆ 52000  ┆ 2320000    │
    │ USA         ┆ Phoenix     ┆ null   ┆ 1680000    │
    │ Netherlands ┆ Amsterdam   ┆ 42000  ┆ 900000     │
    └─────────────┴─────────────┴────────┴────────────┘ |}];
  Sql_context.execute_exn
    ctx
    ~query:
      {|
  SELECT city, population
  FROM population
  WHERE STARTS_WITH(country,'U')|}
  |> Lazy_frame.collect_exn
  |> Data_frame.print;
  [%expect
    {|
    shape: (5, 2)
    ┌─────────────┬────────────┐
    │ city        ┆ population │
    │ ---         ┆ ---        │
    │ str         ┆ i64        │
    ╞═════════════╪════════════╡
    │ New York    ┆ 8399000    │
    │ Los Angeles ┆ 3997000    │
    │ Chicago     ┆ 2705000    │
    │ Houston     ┆ 2320000    │
    │ Phoenix     ┆ 1680000    │
    └─────────────┴────────────┘ |}];
  Sql_context.execute_exn ctx ~query:{|
  SELECT *
  FROM read_csv('./data/iris.csv')|}
  |> Lazy_frame.collect_exn
  |> Data_frame.print;
  [%expect
    {|
    shape: (150, 5)
    ┌──────────────┬─────────────┬──────────────┬─────────────┬────────────────┐
    │ sepal_length ┆ sepal_width ┆ petal_length ┆ petal_width ┆ species        │
    │ ---          ┆ ---         ┆ ---          ┆ ---         ┆ ---            │
    │ f64          ┆ f64         ┆ f64          ┆ f64         ┆ str            │
    ╞══════════════╪═════════════╪══════════════╪═════════════╪════════════════╡
    │ 5.1          ┆ 3.5         ┆ 1.4          ┆ 0.2         ┆ Iris-setosa    │
    │ 4.9          ┆ 3.0         ┆ 1.4          ┆ 0.2         ┆ Iris-setosa    │
    │ 4.7          ┆ 3.2         ┆ 1.3          ┆ 0.2         ┆ Iris-setosa    │
    │ 4.6          ┆ 3.1         ┆ 1.5          ┆ 0.2         ┆ Iris-setosa    │
    │ …            ┆ …           ┆ …            ┆ …           ┆ …              │
    │ 6.3          ┆ 2.5         ┆ 5.0          ┆ 1.9         ┆ Iris-virginica │
    │ 6.5          ┆ 3.0         ┆ 5.2          ┆ 2.0         ┆ Iris-virginica │
    │ 6.2          ┆ 3.4         ┆ 5.4          ┆ 2.3         ┆ Iris-virginica │
    │ 5.9          ┆ 3.0         ┆ 5.1          ┆ 1.8         ┆ Iris-virginica │
    └──────────────┴─────────────┴──────────────┴─────────────┴────────────────┘ |}]
;;

(* Examples from https://pola-rs.github.io/polars-book/user-guide/sql/create/ *)
let%expect_test "CREATE" =
  let create_lazy_frame columns = Data_frame.create_exn columns |> Data_frame.lazy_ in
  let df =
    create_lazy_frame
      Series.
        [ string "name" [ "Alice"; "Bob"; "Charlie"; "David" ]
        ; int "age" [ 25; 30; 35; 40 ]
        ]
  in
  let ctx = Sql_context.create [ "my_table", df ] in
  Sql_context.execute_exn
    ctx
    ~query:{|
  CREATE TABLE older_people
  AS
  SELECT * FROM my_table WHERE age > 30|}
  |> Lazy_frame.collect_exn
  |> Data_frame.print;
  [%expect
    {|
    shape: (1, 1)
    ┌──────────────┐
    │ Response     │
    │ ---          │
    │ str          │
    ╞══════════════╡
    │ Create Table │
    └──────────────┘ |}];
  Sql_context.execute_exn ctx ~query:"SELECT * FROM older_people"
  |> Lazy_frame.collect_exn
  |> Data_frame.print;
  [%expect
    {|
    shape: (2, 2)
    ┌─────────┬─────┐
    │ name    ┆ age │
    │ ---     ┆ --- │
    │ str     ┆ i64 │
    ╞═════════╪═════╡
    │ Charlie ┆ 35  │
    │ David   ┆ 40  │
    └─────────┴─────┘ |}]
;;

(* Examples from https://pola-rs.github.io/polars-book/user-guide/sql/cte/ *)
let%expect_test "Common Table Expressions" =
  let create_lazy_frame columns = Data_frame.create_exn columns |> Data_frame.lazy_ in
  let df =
    create_lazy_frame
      Series.
        [ string "name" [ "Alice"; "Bob"; "Charlie"; "David" ]
        ; int "age" [ 25; 30; 35; 40 ]
        ]
  in
  let ctx = Sql_context.create [ "my_table", df ] in
  Sql_context.execute_exn
    ctx
    ~query:
      {|
  WITH older_people AS (
      SELECT * FROM my_table WHERE age > 30
  )
  SELECT * FROM older_people WHERE STARTS_WITH(name,'C')|}
  |> Lazy_frame.collect_exn
  |> Data_frame.print;
  [%expect
    {|
    shape: (1, 2)
    ┌─────────┬─────┐
    │ name    ┆ age │
    │ ---     ┆ --- │
    │ str     ┆ i64 │
    ╞═════════╪═════╡
    │ Charlie ┆ 35  │
    └─────────┴─────┘ |}]
;;
