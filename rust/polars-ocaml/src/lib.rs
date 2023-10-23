mod data_frame;
mod expr;
mod lazy_frame;
mod misc;
mod series;
mod sql_context;
mod utils;

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use expect_test::{expect, Expect};
    use polars::prelude::*;
    use std::fmt::Debug;

    fn check<T: Debug>(actual: T, expect: Expect) {
        let actual = format!("{:?}", actual);
        expect.assert_eq(&actual);
    }

    // https://github.com/pola-rs/polars/issues/11806
    #[test]
    fn date_lit_dtype_silent_conversion_bug() {
        let date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();

        let df = DataFrame::new::<Series>(vec![])
            .unwrap()
            .lazy()
            .select([lit(date)])
            .collect()
            .unwrap();

        check(
            df.clone(),
            expect![[r#"
                shape: (1, 1)
                ┌─────────────────────┐
                │ literal             │
                │ ---                 │
                │ datetime[ns]        │
                ╞═════════════════════╡
                │ 2023-01-01 00:00:00 │
                └─────────────────────┘"#]],
        );

        check(
            Series::new("date", [date]),
            expect![[r#"
                shape: (1,)
                Series: 'date' [date]
                [
                	2023-01-01
                ]"#]],
        );
    }

    #[test]
    fn list_date_series_creation_dtype_confusion() {
        let empty_date_series = Series::new_empty("test", &DataType::Date);
        let series = Series::new("date", vec![empty_date_series]);

        check(
            series.clone(),
            expect![[r#"
                shape: (1,)
                Series: 'date' [list[date]]
                [
                	[]
                ]"#]],
        );

        check(
            series.get(0).unwrap(),
            expect![[r#"
                List(shape: (0,)
                Series: '' [date]
                [
                ])"#]],
        );

        check(
            series.list().unwrap().get(0).unwrap(),
            expect![[r#"
                shape: (0,)
                Series: 'date' [i32]
                [
                ]"#]],
        );
    }

    #[test]
    #[should_panic]
    fn list_date_series_creation_panic() {
        let empty_series = Series::new_empty("test", &DataType::List(Box::new(DataType::Date)));
        let _series = Series::new("test", vec![empty_series]);
    }

    #[test]
    fn check_div() {
        std::env::set_var("POLARS_TABLE_WIDTH", "100");

        let weather_by_day = DataFrame::new(vec![
            Series::new(
                "station",
                (1..11)
                    .map(|i| format!("Station_{}", i))
                    .collect::<Vec<_>>(),
            ),
            Series::new("day_1", [17, 11, 8, 22, 9, 21, 20, 8, 8, 17]),
            Series::new("day_2", [15, 11, 10, 8, 7, 14, 18, 21, 15, 13]),
            Series::new("day_3", [16, 15, 24, 24, 8, 23, 19, 23, 16, 10]),
        ])
        .unwrap();

        check(
            weather_by_day.clone(),
            expect![[r#"
                shape: (10, 4)
                ┌────────────┬───────┬───────┬───────┐
                │ station    ┆ day_1 ┆ day_2 ┆ day_3 │
                │ ---        ┆ ---   ┆ ---   ┆ ---   │
                │ str        ┆ i32   ┆ i32   ┆ i32   │
                ╞════════════╪═══════╪═══════╪═══════╡
                │ Station_1  ┆ 17    ┆ 15    ┆ 16    │
                │ Station_2  ┆ 11    ┆ 11    ┆ 15    │
                │ Station_3  ┆ 8     ┆ 10    ┆ 24    │
                │ Station_4  ┆ 22    ┆ 8     ┆ 24    │
                │ …          ┆ …     ┆ …     ┆ …     │
                │ Station_7  ┆ 20    ┆ 18    ┆ 19    │
                │ Station_8  ┆ 8     ┆ 21    ┆ 23    │
                │ Station_9  ┆ 8     ┆ 15    ┆ 16    │
                │ Station_10 ┆ 17    ┆ 13    ┆ 10    │
                └────────────┴───────┴───────┴───────┘"#]],
        );

        // Interestingly, the Rust div function is translated into
        // Operator::Divide (which doesn't convert output to float) while
        // __div__ in Python gets translated into Operator::TrueDivide
        // (which converts output to float)

        // let rank_pct = (col("")
        //     .rank(
        //         RankOptions {
        //             descending: true,
        //             ..Default::default()
        //         },
        //         None,
        //     )
        //     .cast(DataType::Float64)
        //     / col("*").count())
        // .round(2);

        let rank_pct = (Expr::BinaryExpr {
            left: Box::new(col("").rank(
                RankOptions {
                    descending: true,
                    ..Default::default()
                },
                None,
            )),
            op: Operator::TrueDivide,
            right: Box::new(col("*").count()),
        })
        .round(2);

        let out = weather_by_day
            .lazy()
            .with_column(
                concat_list([all().exclude(["station"])])
                    .unwrap()
                    .alias("all_temps"),
            )
            .collect()
            .unwrap()
            .lazy()
            .select([
                all().exclude(["all_temps"]),
                col("all_temps")
                    .list()
                    .eval(rank_pct, false)
                    .alias("temps_rank"),
            ])
            .collect()
            .unwrap();

        check(
            out,
            expect![[r#"
                shape: (10, 5)
                ┌────────────┬───────┬───────┬───────┬────────────────────┐
                │ station    ┆ day_1 ┆ day_2 ┆ day_3 ┆ temps_rank         │
                │ ---        ┆ ---   ┆ ---   ┆ ---   ┆ ---                │
                │ str        ┆ i32   ┆ i32   ┆ i32   ┆ list[f64]          │
                ╞════════════╪═══════╪═══════╪═══════╪════════════════════╡
                │ Station_1  ┆ 17    ┆ 15    ┆ 16    ┆ [0.33, 1.0, 0.67]  │
                │ Station_2  ┆ 11    ┆ 11    ┆ 15    ┆ [0.67, 0.67, 0.33] │
                │ Station_3  ┆ 8     ┆ 10    ┆ 24    ┆ [1.0, 0.67, 0.33]  │
                │ Station_4  ┆ 22    ┆ 8     ┆ 24    ┆ [0.67, 1.0, 0.33]  │
                │ …          ┆ …     ┆ …     ┆ …     ┆ …                  │
                │ Station_7  ┆ 20    ┆ 18    ┆ 19    ┆ [0.33, 1.0, 0.67]  │
                │ Station_8  ┆ 8     ┆ 21    ┆ 23    ┆ [1.0, 0.67, 0.33]  │
                │ Station_9  ┆ 8     ┆ 15    ┆ 16    ┆ [1.0, 0.67, 0.33]  │
                │ Station_10 ┆ 17    ┆ 13    ┆ 10    ┆ [0.33, 0.67, 1.0]  │
                └────────────┴───────┴───────┴───────┴────────────────────┘"#]],
        );
    }

    // I don't really understand what's going on here, but below tests correctly
    // demonstrates a panic when running on a box on EC2[1] but not on my development machine at home[2]
    //
    // [1]:
    // $ cat /proc/cpuinfo | grep 'model name' | uniq
    // model name      : AMD EPYC 7571
    //
    // [2]:
    // $ cat /proc/cpuinfo | grep 'model name' | uniq
    // model name      : 13th Gen Intel(R) Core(TM) i9-13900K

    // https://github.com/pola-rs/polars/issues/9916
    // #[test]
    // #[should_panic]
    // fn lazy_sort_instability() {
    //     let dataset = CsvReader::from_path("../test/data/legislators-historical.csv")
    //         .unwrap()
    //         .finish()
    //         .unwrap();

    //     let mut prev: Option<DataFrame> = None;

    //     for _ in 0..10000 {
    //         let df = dataset
    //             .clone()
    //             .lazy()
    //             .groupby_stable(vec![col("state")])
    //             .agg(vec![
    //                 (col("party").eq(lit("Anti-Administration")))
    //                     .mean()
    //                     .alias("anti"),
    //                 (col("party").eq(lit("Pro-Administration")))
    //                     .mean()
    //                     .alias("pro"),
    //             ])
    //             .sort(
    //                 "pro",
    //                 SortOptions {
    //                     multithreaded: false,
    //                     maintain_order: true,
    //                     ..Default::default()
    //                 },
    //             )
    //             .limit(5)
    //             .collect()
    //             .unwrap();

    //         if let Some(prev) = prev {
    //             assert_eq!(df, prev);
    //         }
    //         prev = Some(df);
    //     }
    // }
}
