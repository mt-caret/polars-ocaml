use chrono::naive::{NaiveDate, NaiveDateTime};
use ocaml_interop::{
    ocaml_export, ocaml_unpack_variant, BoxRoot, DynBox, FromOCaml, OCaml, OCamlBytes, OCamlFloat,
    OCamlInt, OCamlList, OCamlRef, OCamlRuntime, ToOCaml,
};
use polars::prelude::prelude::*;
use polars::prelude::*;
use std::fmt::Display;
use std::{borrow::Borrow, path::Path};

struct PolarsTimeUnit(TimeUnit);

unsafe impl FromOCaml<TimeUnit> for PolarsTimeUnit {
    fn from_ocaml(v: OCaml<TimeUnit>) -> Self {
        let result = ocaml_unpack_variant! {
            v => {
                TimeUnit::Nanoseconds,
                TimeUnit::Microseconds,
                TimeUnit::Milliseconds,
            }
        };
        PolarsTimeUnit(result.expect("Failure when unpacking an OCaml<TimeUnit> variant into PolarsTimeUnit (unexpected tag value"))
    }
}

struct PolarsDataType(DataType);

unsafe impl FromOCaml<DataType> for PolarsDataType {
    fn from_ocaml(v: OCaml<DataType>) -> Self {
        let result = ocaml_unpack_variant! {
            v => {
                DataType::Boolean,
                DataType::UInt8,
                DataType::UInt16,
                DataType::UInt32,
                DataType::UInt64,
                DataType::Int8,
                DataType::Int16,
                DataType::Int32,
                DataType::Int64,
                DataType::Float32,
                DataType::Float64,
                DataType::Utf8,
                DataType::Binary,
                DataType::Date,
                DataType::Datetime(timeunit: TimeUnit) => {
                    let PolarsTimeUnit(timeunit) = timeunit;
                    DataType::Datetime(timeunit, None)},
                DataType::Time,
                DataType::Null,
                DataType::Unknown,
            }
        };
        PolarsDataType(result.expect("Failure when unpacking an OCaml<DataType> variant into PolarsDataType (unexpected tag value"))
    }
}

fn ocaml_list_to_vec<T: 'static + Clone>(mut list: OCaml<OCamlList<DynBox<T>>>) -> Vec<T> {
    let mut ret = Vec::new();

    while let Some((head, tail)) = list.uncons() {
        ret.push((*Borrow::<T>::borrow(&head)).clone());
        list = tail;
    }

    ret
}

fn box_result<'a, T: 'static, E: Display>(
    cr: &'a mut &'a mut OCamlRuntime,
    result: Result<T, E>,
) -> OCaml<'a, Result<DynBox<T>, String>> {
    match result {
        Err(err) => Err::<BoxRoot<DynBox<_>>, _>(err.to_string()).to_ocaml(cr),
        Ok(v) => {
            let v: BoxRoot<DynBox<T>> = OCaml::box_value(cr, v).root();
            Ok::<_, String>(v).to_ocaml(cr)
        }
    }
}

fn expr_unary_op<'a>(
    cr: &'a mut &'a mut OCamlRuntime,
    expr: OCamlRef<'a, DynBox<Expr>>,
    f: impl Fn(Expr) -> Expr,
) -> OCaml<'a, DynBox<Expr>> {
    let expr: Expr = Borrow::<Expr>::borrow(&expr.to_ocaml(cr)).clone();
    OCaml::box_value(cr, f(expr))
}

fn expr_binary_op<'a>(
    cr: &'a mut &'a mut OCamlRuntime,
    expr: OCamlRef<'a, DynBox<Expr>>,
    other: OCamlRef<'a, DynBox<Expr>>,
    f: impl Fn(Expr, Expr) -> Expr,
) -> OCaml<'a, DynBox<Expr>> {
    let expr: Expr = Borrow::<Expr>::borrow(&expr.to_ocaml(cr)).clone();
    let other: Expr = Borrow::<Expr>::borrow(&other.to_ocaml(cr)).clone();
    OCaml::box_value(cr, f(expr, other))
}

enum WhenThenClause {
    Empty,
    WhenThen(WhenThen),
    WhenThenThen(WhenThenThen),
}

ocaml_export! {
    fn rust_naive_date(cr, year: OCamlRef<OCamlInt>, month: OCamlRef<OCamlInt>, day: OCamlRef<OCamlInt>) -> OCaml<Option<DynBox<NaiveDate>>> {
        let year: i32 = year.to_rust(cr);
        let month: Option<u32> = month.to_rust::<i32>(cr).try_into().ok();
        let day: Option<u32> = day.to_rust::<i32>(cr).try_into().ok();
        match (month, day) {
            (Some(month), Some(day)) => {
                match NaiveDate::from_ymd_opt(year, month, day) {
                    None => OCaml::none(),
                    Some(date) => {
                        let date: BoxRoot<DynBox<NaiveDate>> = OCaml::box_value(cr, date).root();
                        Some(date).to_ocaml(cr)
                    }
                }
            },
            _ => OCaml::none()
        }
    }

    fn rust_naive_date_to_naive_datetime(cr, date: OCamlRef<DynBox<NaiveDate>>) -> OCaml<Option<DynBox<NaiveDateTime>>> {
        let date: NaiveDate = *Borrow::<NaiveDate>::borrow(&date.to_ocaml(cr));

        match date.and_hms_opt(0, 0, 0) {
            None => OCaml::none(),
            Some(datetime) => {
                let datetime: BoxRoot<DynBox<NaiveDateTime>> = OCaml::box_value(cr, datetime).root();
                Some(datetime).to_ocaml(cr)
            }
        }
    }

    fn rust_expr_col(cr, name: OCamlRef<String>) -> OCaml<DynBox<Expr>> {
        let name: String = name.to_rust(cr);
        OCaml::box_value(cr, col(&name))
    }

    fn rust_expr_all(cr, unit: OCamlRef<()>) -> OCaml<DynBox<Expr>> {
        let _: () = unit.to_rust(cr);
        OCaml::box_value(cr, all())
    }

    fn rust_expr_exclude(cr, name: OCamlRef<String>) -> OCaml<DynBox<Expr>> {
        let name: String = name.to_rust(cr);
        OCaml::box_value(cr, all().exclude([name]))
    }

    fn rust_expr_int(cr, value: OCamlRef<OCamlInt>) -> OCaml<DynBox<Expr>> {
        let value: i64 = value.to_rust(cr);
        OCaml::box_value(cr, lit(value))
    }

    fn rust_expr_float(cr, value: OCamlRef<OCamlFloat>) -> OCaml<DynBox<Expr>> {
        let value: f64 = value.to_rust(cr);
        OCaml::box_value(cr, lit(value))
    }

    fn rust_expr_bool(cr, value: OCamlRef<bool>) -> OCaml<DynBox<Expr>> {
        let value: bool = value.to_rust(cr);
        OCaml::box_value(cr, lit(value))
    }

    fn rust_expr_string(cr, value: OCamlRef<String>) -> OCaml<DynBox<Expr>> {
        let value: String = value.to_rust(cr);
        OCaml::box_value(cr, lit(value))
    }

    fn rust_expr_cast(cr, expr: OCamlRef<DynBox<Expr>>, data_type: OCamlRef<DataType>, is_strict: OCamlRef<bool>) -> OCaml<DynBox<Expr>> {
        let PolarsDataType(data_type): PolarsDataType = data_type.to_rust(cr);
        let is_strict: bool = is_strict.to_rust(cr);
        expr_unary_op(cr, expr, |expr|
            if is_strict {
                expr.strict_cast(data_type.clone())
            } else {
                expr.cast(data_type.clone())
            })
    }

    fn rust_expr_sort(cr, expr: OCamlRef<DynBox<Expr>>, descending: OCamlRef<bool>) -> OCaml<DynBox<Expr>> {
        let descending: bool = descending.to_rust(cr);
        expr_unary_op(cr, expr, |expr| expr.sort(descending))
    }

    fn rust_expr_head(cr, expr: OCamlRef<DynBox<Expr>>, length: OCamlRef<Option<OCamlInt>>) -> OCaml<Option<DynBox<Expr>>> {
        let expr: Expr = Borrow::<Expr>::borrow(&expr.to_ocaml(cr)).clone();
        let length: Option<i64> = length.to_rust(cr);

        match length {
            None => {
                let expr: BoxRoot<DynBox<Expr>> = OCaml::box_value(cr, expr.head(None)).root();
                Some(expr).to_ocaml(cr)
            },
            Some(length) => {
                match length.try_into().ok() {
                    // TODO: this should probably be an error instead of none
                    None => OCaml::none(),
                    Some(length) => {
                        let expr: BoxRoot<DynBox<Expr>> = OCaml::box_value(cr, expr.head(Some(length))).root();
                        Some(expr).to_ocaml(cr)
                    },
                }
            }
        }
    }

    fn rust_expr_filter(cr, expr: OCamlRef<DynBox<Expr>>, predicate: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, predicate, |expr, predicate| expr.filter(predicate))
    }

    fn rust_expr_sum(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.sum())
    }

    fn rust_expr_n_unique(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.n_unique())
    }

    fn rust_expr_approx_unique(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.approx_unique())
    }

    fn rust_expr_when_then(cr, when_then_clauses: OCamlRef<OCamlList<(DynBox<Expr>, DynBox<Expr>)>>, otherwise: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        let mut when_then_clauses = when_then_clauses.to_ocaml(cr);
        let when_then_clauses = {
            let mut ret = Vec::new();

            while let Some((head, tail)) = when_then_clauses.uncons() {
                let (when, then): (OCaml<DynBox<Expr>>, OCaml<DynBox<Expr>>) = head.to_tuple();
                let when = Borrow::<Expr>::borrow(&when).clone();
                let then = Borrow::<Expr>::borrow(&then).clone();
                ret.push((when, then));
                when_then_clauses = tail;
            }

            ret
        };

        let otherwise: Expr = Borrow::<Expr>::borrow(&otherwise.to_ocaml(cr)).clone();

        let mut ret = WhenThenClause::Empty;

        for (when_expr, then_expr) in when_then_clauses {
            match ret {
                WhenThenClause::Empty => ret = WhenThenClause::WhenThen(when(when_expr).then(then_expr)),
                WhenThenClause::WhenThen(when_then) => ret = WhenThenClause::WhenThenThen(when_then.when(when_expr).then(then_expr)),
                WhenThenClause::WhenThenThen(when_then_then) => ret = WhenThenClause::WhenThenThen(when_then_then.when(when_expr).then(then_expr))
            }
        }

        match ret {
            WhenThenClause::Empty => OCaml::box_value(cr, otherwise),
            WhenThenClause::WhenThen(when_then) => OCaml::box_value(cr, when_then.otherwise(otherwise)),
            WhenThenClause::WhenThenThen(when_then_then) => OCaml::box_value(cr, when_then_then.otherwise(otherwise))
        }
    }

    fn rust_expr_alias(cr, expr: OCamlRef<DynBox<Expr>>, name: OCamlRef<String>) -> OCaml<DynBox<Expr>> {
        let name: String = name.to_rust(cr);
        expr_unary_op(cr, expr, |expr| expr.alias(&name))
    }

    fn rust_expr_prefix(cr, expr: OCamlRef<DynBox<Expr>>, prefix: OCamlRef<String>) -> OCaml<DynBox<Expr>> {
        let prefix: String = prefix.to_rust(cr);
        expr_unary_op(cr, expr, |expr| expr.prefix(&prefix))
    }

    fn rust_expr_suffix(cr, expr: OCamlRef<DynBox<Expr>>, suffix: OCamlRef<String>) -> OCaml<DynBox<Expr>> {
        let suffix: String = suffix.to_rust(cr);
        expr_unary_op(cr, expr, |expr| expr.suffix(&suffix))
    }

    fn rust_expr_eq(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a.eq(b))
    }

    fn rust_expr_neq(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a.neq(b))
    }

    fn rust_expr_gt(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a.gt(b))
    }

    fn rust_expr_gt_eq(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a.gt_eq(b))
    }

    fn rust_expr_lt(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a.lt(b))
    }

    fn rust_expr_lt_eq(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a.lt_eq(b))
    }

    fn rust_expr_not(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.not())
    }

    fn rust_expr_and(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a.and(b))
    }

    fn rust_expr_or(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a.or(b))
    }

    fn rust_expr_xor(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a.xor(b))
    }

    fn rust_expr_add(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a + b)
    }

    fn rust_expr_sub(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a - b)
    }

    fn rust_expr_mul(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a * b)
    }

    fn rust_expr_div(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a / b)
    }

    fn rust_expr_dt_strftime(cr, expr: OCamlRef<DynBox<Expr>>, format:OCamlRef<String>)-> OCaml<DynBox<Expr>> {
        let format: String = format.to_rust(cr);
        expr_unary_op(cr, expr, |expr| expr.dt().to_string(&format))
    }

    fn rust_expr_str_strptime(cr, expr: OCamlRef<DynBox<Expr>>, data_type: OCamlRef<DataType>, format:OCamlRef<String>)-> OCaml<DynBox<Expr>> {
        let PolarsDataType(data_type): PolarsDataType = data_type.to_rust(cr);
        let format: String = format.to_rust(cr);

        expr_unary_op(cr, expr, |expr| {
            let options = StrptimeOptions {
                format: Some(format.clone()), strict: true, exact:true, cache: false
            };
            expr.str().strptime(data_type.clone(), options)
        })
    }

    fn rust_series_new_int(cr, name: OCamlRef<String>, values: OCamlRef<OCamlList<OCamlInt>>) -> OCaml<DynBox<Series>> {
        let name: String = name.to_rust(cr);
        let values: Vec<i64> = values.to_rust(cr);
        OCaml::box_value(cr, Series::new(&name, values))
    }

    fn rust_series_new_int_option(cr, name: OCamlRef<String>, values: OCamlRef<OCamlList<Option<OCamlInt>>>) -> OCaml<DynBox<Series>> {
        let name: String = name.to_rust(cr);
        let values: Vec<Option<i64>> = values.to_rust(cr);
        OCaml::box_value(cr, Series::new(&name, values))
    }

    fn rust_series_new_float(cr, name: OCamlRef<String>, values: OCamlRef<OCamlList<OCamlFloat>>) -> OCaml<DynBox<Series>> {
        let name: String = name.to_rust(cr);
        let values: Vec<f64> = values.to_rust(cr);
        OCaml::box_value(cr, Series::new(&name, values))
    }

    fn rust_series_new_float_option(cr, name: OCamlRef<String>, values: OCamlRef<OCamlList<Option<OCamlFloat>>>) -> OCaml<DynBox<Series>> {
        let name: String = name.to_rust(cr);
        let values: Vec<Option<f64>> = values.to_rust(cr);
        OCaml::box_value(cr, Series::new(&name, values))
    }

    fn rust_series_new_string(cr, name: OCamlRef<String>, values: OCamlRef<OCamlList<String>>) -> OCaml<DynBox<Series>> {
        let name: String = name.to_rust(cr);
        let values: Vec<String> = values.to_rust(cr);
        OCaml::box_value(cr, Series::new(&name, values))
    }

    fn rust_series_new_string_option(cr, name: OCamlRef<String>, values: OCamlRef<OCamlList<Option<String>>>) -> OCaml<DynBox<Series>> {
        let name: String = name.to_rust(cr);
        let values: Vec<Option<String>> = values.to_rust(cr);
        OCaml::box_value(cr, Series::new(&name, values))
    }

    fn rust_series_new_bool(cr, name: OCamlRef<String>, values: OCamlRef<OCamlList<bool>>) -> OCaml<DynBox<Series>> {
        let name: String = name.to_rust(cr);
        let values: Vec<bool> = values.to_rust(cr);
        OCaml::box_value(cr, Series::new(&name, values))
    }

    fn rust_series_new_bool_option(cr, name: OCamlRef<String>, values: OCamlRef<OCamlList<Option<bool>>>) -> OCaml<DynBox<Series>> {
        let name: String = name.to_rust(cr);
        let values: Vec<Option<bool>> = values.to_rust(cr);
        OCaml::box_value(cr, Series::new(&name, values))
    }

    fn rust_series_date_range(cr, name: OCamlRef<String>, start: OCamlRef<DynBox<NaiveDateTime>>, stop: OCamlRef<DynBox<NaiveDateTime>>, cast_to_date: OCamlRef<bool>) -> OCaml<Result<DynBox<Series>,String>> {
        let name: String = name.to_rust(cr);

        let start: OCaml<DynBox<NaiveDateTime>> = start.to_ocaml(cr);
        let start: NaiveDateTime = *Borrow::<NaiveDateTime>::borrow(&start);

        let stop: OCaml<DynBox<NaiveDateTime>> = stop.to_ocaml(cr);
        let stop: NaiveDateTime = *Borrow::<NaiveDateTime>::borrow(&stop);

        let cast_to_date: bool = cast_to_date.to_rust(cr);

        let series =
            date_range(&name, start, stop, Duration::parse("1d"), ClosedWindow::Both, TimeUnit::Milliseconds, None).and_then(|date_range| {
                let series = date_range.into_series();
                if cast_to_date {
                    series.cast(&DataType::Date)
                } else {
                    Ok(series)
                }
            });

        match series {
            Err(err) => {
                Err::<BoxRoot<DynBox<Series>>, _>(err.to_string()).to_ocaml(cr)
            },
            Ok(series) => {
                let series: BoxRoot<DynBox<Series>> = OCaml::box_value(cr, series).root();
                Ok::<_, String>(series).to_ocaml(cr)
            },
        }
    }

    fn rust_series_to_string_hum(cr, series: OCamlRef<DynBox<Series>>) -> OCaml<String> {
        let series: OCaml<DynBox<Series>> = series.to_ocaml(cr);
        let series: &Series = Borrow::<Series>::borrow(&series);
        ToString::to_string(series).to_ocaml(cr)
    }

    fn rust_data_frame_new(cr, series: OCamlRef<OCamlList<DynBox<Series>>>) -> OCaml<Result<DynBox<DataFrame>,String>> {
        let series = ocaml_list_to_vec(series.to_ocaml(cr));

        match DataFrame::new(series) {
            Err(err) => {
                Err::<BoxRoot<DynBox<DataFrame>>, _>(err.to_string()).to_ocaml(cr)
            },
            Ok(data_frame) => {
                let data_frame: BoxRoot<DynBox<DataFrame>> = OCaml::box_value(cr, data_frame).root();
                Ok::<_, String>(data_frame).to_ocaml(cr)
            }
        }
    }

    fn rust_data_frame_to_string_hum(cr, data_frame: OCamlRef<DynBox<DataFrame>>) -> OCaml<String> {
        let data_frame: OCaml<DynBox<DataFrame>> = data_frame.to_ocaml(cr);
        let data_frame: &DataFrame = Borrow::<DataFrame>::borrow(&data_frame);
        data_frame.to_string().to_ocaml(cr)
    }

    fn rust_data_frame_lazy(cr, data_frame: OCamlRef<DynBox<DataFrame>>) -> OCaml<DynBox<LazyFrame>> {
        let data_frame: OCaml<DynBox<DataFrame>> = data_frame.to_ocaml(cr);
        let data_frame: DataFrame = Borrow::<DataFrame>::borrow(&data_frame).clone();
        OCaml::box_value(cr, data_frame.lazy())
    }

    // TODO: properly return error type instead of a string
    fn rust_lazy_frame_scan_parquet(cr, path: OCamlRef<OCamlBytes>) -> OCaml<Result<DynBox<LazyFrame>, String>>{
        let path:String = path.to_rust(cr);
        let path:&Path = Path::new(&path);

        box_result(cr, LazyFrame::scan_parquet(path, Default::default()))
    }

    fn rust_lazy_frame_to_dot(cr, lazy_frame: OCamlRef<DynBox<LazyFrame>>) -> OCaml<Result<String,String>>{
        let lazy_frame: OCaml<DynBox<LazyFrame>> = lazy_frame.to_ocaml(cr);

        // TODO: make configurable
        match Borrow::<LazyFrame>::borrow(&lazy_frame).to_dot(false) {
            Err(err) => {
                Err::<String, _>(err.to_string()).to_ocaml(cr)
            },
            Ok(dot) => {
                Ok::<_, String>(dot).to_ocaml(cr)
            }
        }
    }

    fn rust_lazy_frame_collect(cr, lazy_frame: OCamlRef<DynBox<LazyFrame>>)-> OCaml<Result<DynBox<DataFrame>, String>> {
        let lazy_frame: OCaml<DynBox<LazyFrame>> = lazy_frame.to_ocaml(cr);
        let lazy_frame = Borrow::<LazyFrame>::borrow(&lazy_frame).clone();

        box_result(cr, lazy_frame.collect())
    }

    fn rust_lazy_frame_select(cr, lazy_frame: OCamlRef<DynBox<LazyFrame>>, exprs: OCamlRef<OCamlList<DynBox<Expr>>>) -> OCaml<DynBox<LazyFrame>> {
        let exprs = ocaml_list_to_vec(exprs.to_ocaml(cr));

        let lazy_frame: OCaml<DynBox<LazyFrame>> = lazy_frame.to_ocaml(cr);
        let lazy_frame: LazyFrame = Borrow::<LazyFrame>::borrow(&lazy_frame).clone();

        OCaml::box_value(cr, lazy_frame.select(&exprs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::{expect, Expect};
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
}
