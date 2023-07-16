#![feature(try_blocks)]
mod data_frame;
mod expr;
mod lazy_frame;
mod misc;
mod series;
mod utils;

#[cfg(test)]
mod tests {
    use chrono::naive::NaiveDate;
    use expect_test::{expect, Expect};
    use polars::prelude::prelude::*;
    use polars::prelude::*;
    use std::fmt::Debug;

    fn check<T: Debug>(actual: T, expect: Expect) {
        let actual = format!("{:?}", actual);
        expect.assert_eq(&actual);
    }

    // https://github.com/pola-rs/polars/issues/9409
    #[test]
    fn check_date_range() {
        let start = NaiveDate::from_ymd_opt(2022, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let stop = NaiveDate::from_ymd_opt(2022, 1, 5)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        check(
            date_range(
                "date",
                start,
                stop,
                Duration::parse("1d"),
                ClosedWindow::Both,
                TimeUnit::Nanoseconds,
                None,
            )
            .map(|date_range| date_range.into_series()),
            expect![[r#"
                Ok(shape: (5,)
                Series: 'date' [datetime[ns]]
                [
                	2022-01-01 00:00:00
                	2022-01-02 00:00:00
                	2022-01-03 00:00:00
                	2022-01-04 00:00:00
                	2022-01-05 00:00:00
                ])"#]],
        );
        check(
            date_range(
                "date",
                start,
                stop,
                Duration::parse("1d"),
                ClosedWindow::Both,
                TimeUnit::Microseconds, // TODO: BUG!
                None,
            )
            .map(|date_range| date_range.into_series()),
            expect![[r#"
                Ok(shape: (5,)
                Series: 'date' [datetime[μs]]
                [
                	2022-01-01 00:00:00
                	2022-01-02 00:00:00
                	2022-01-03 00:00:00
                	2022-01-04 00:00:00
                	2022-01-05 00:00:00
                ])"#]],
        );
        check(
            date_range(
                "date",
                start,
                stop,
                Duration::parse("1d"),
                ClosedWindow::Both,
                TimeUnit::Milliseconds,
                None,
            )
            .map(|date_range| date_range.into_series()),
            expect![[r#"
                Ok(shape: (5,)
                Series: 'date' [datetime[ms]]
                [
                	2022-01-01 00:00:00
                	2022-01-02 00:00:00
                	2022-01-03 00:00:00
                	2022-01-04 00:00:00
                	2022-01-05 00:00:00
                ])"#]],
        )
    }

    #[test]
    fn check_div() {
        let weather_by_day = DataFrame::new(vec![
            Series::new(
                "station",
                (1..11)
                    .into_iter()
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
}
