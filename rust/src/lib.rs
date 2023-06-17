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
                TimeUnit::Microseconds, // TODO: BUG!
                None,
            )
            .map(|date_range| date_range.into_series()),
            expect![[r#"
                Ok(shape: (1,)
                Series: 'date' [datetime[μs]]
                [
                	1970-01-01 00:27:20.995200
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
    fn lazy_groupby_issue() {
        let dataset = CsvReader::from_path("../test/data/legislators-historical.csv")
            .unwrap()
            .finish()
            .unwrap()
            .head(Some(1000));

        let df = dataset
            .lazy()
            .groupby_stable(vec![col("state")])
            .agg(vec![col("party").eq(lit("some_string")).sum()])
            .collect()
            .unwrap();

        expect![[r#""#]].assert_eq(&df.to_string());
    }
}
