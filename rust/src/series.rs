use crate::utils::*;
use chrono::naive::{NaiveDate, NaiveDateTime};
use ocaml_interop::{
    ocaml_export, DynBox, OCaml, OCamlFloat, OCamlInt, OCamlList, OCamlRef, OCamlRuntime, ToOCaml,
};
use polars::prelude::prelude::*;
use polars::prelude::*;

fn series_binary_op<'a>(
    cr: &'a mut &'a mut OCamlRuntime,
    series: OCamlRef<'a, DynBox<Series>>,
    other: OCamlRef<'a, DynBox<Series>>,
    f: impl Fn(Series, Series) -> Series,
) -> OCaml<'a, DynBox<Series>> {
    let Abstract(series) = series.to_rust(cr);
    let Abstract(other) = other.to_rust(cr);

    OCaml::box_value(cr, f(series, other))
}

fn series_binary_op_result<'a>(
    cr: &'a mut &'a mut OCamlRuntime,
    series: OCamlRef<'a, DynBox<Series>>,
    other: OCamlRef<'a, DynBox<Series>>,
    f: impl Fn(Series, Series) -> Result<Series, PolarsError>,
) -> OCaml<'a, Result<DynBox<Series>, String>> {
    let Abstract(series) = series.to_rust(cr);
    let Abstract(other) = other.to_rust(cr);

    let series = f(series, other)
        .map(Abstract)
        .map_err(|err| err.to_string());

    series.to_ocaml(cr)
}

ocaml_export! {
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

    fn rust_series_new_datetime(cr, name: OCamlRef<String>, values: OCamlRef<OCamlList<DynBox<NaiveDateTime>>>) -> OCaml<DynBox<Series>> {
        let name: String = name.to_rust(cr);
        let values = unwrap_abstract_vec(values.to_rust(cr));
        OCaml::box_value(cr, Series::new(&name, values))
    }

    fn rust_series_new_date(cr, name: OCamlRef<String>, values: OCamlRef<OCamlList<DynBox<NaiveDate>>>) -> OCaml<DynBox<Series>> {
        let name: String = name.to_rust(cr);
        let values = unwrap_abstract_vec(values.to_rust(cr));
        OCaml::box_value(cr, Series::new(&name, values))
    }

    fn rust_series_date_range(cr, name: OCamlRef<String>, start: OCamlRef<DynBox<NaiveDateTime>>, stop: OCamlRef<DynBox<NaiveDateTime>>, cast_to_date: OCamlRef<bool>) -> OCaml<Result<DynBox<Series>,String>> {
        let name: String = name.to_rust(cr);

        let Abstract(start) = start.to_rust(cr);
        let Abstract(stop) = stop.to_rust(cr);

        let cast_to_date: bool = cast_to_date.to_rust(cr);

        let series =
            date_range(&name, start, stop, Duration::parse("1d"), ClosedWindow::Both, TimeUnit::Milliseconds, None)
            .and_then(|date_range| {
                let series = date_range.into_series();
                if cast_to_date {
                    series.cast(&DataType::Date)
                } else {
                    Ok(series)
                }
            })
            .map(Abstract)
            .map_err(|err| err.to_string());

        series.to_ocaml(cr)
    }

    fn rust_series_sort(cr, series: OCamlRef<DynBox<Series>>, descending: OCamlRef<bool>) -> OCaml<DynBox<Series>> {
        let Abstract(series) = series.to_rust(cr);
        let descending: bool = descending.to_rust(cr);

        OCaml::box_value(cr, series.sort(descending))
    }

    fn rust_series_head(cr, series: OCamlRef<DynBox<Series>>, length: OCamlRef<Option<OCamlInt>>) -> OCaml<Option<DynBox<Series>>> {
        let Abstract(series) = series.to_rust(cr);
        let length: Option<i64> = length.to_rust(cr);

        match length.map(|length| length.try_into().ok()) {
            None => Some(Abstract(series.head(None))),
            Some(None) => None,
            Some(Some(length)) => Some(Abstract(series.head(Some(length)))),
        }.to_ocaml(cr)
    }

    fn rust_series_tail(cr, series: OCamlRef<DynBox<Series>>, length: OCamlRef<Option<OCamlInt>>) -> OCaml<Option<DynBox<Series>>> {
        let Abstract(series) = series.to_rust(cr);
        let length: Option<i64> = length.to_rust(cr);

        match length.map(|length| length.try_into().ok()) {
            None => Some(Abstract(series.tail(None))),
            Some(None) => None,
            Some(Some(length)) => Some(Abstract(series.tail(Some(length)))),
        }.to_ocaml(cr)
    }

    fn rust_series_sample_n(cr, series: OCamlRef<DynBox<Series>>, n: OCamlRef<OCamlInt>, with_replacement: OCamlRef<bool>, shuffle: OCamlRef<bool>, seed: OCamlRef<Option<OCamlInt>>) -> OCaml<Option<Result<DynBox<Series>,String>>> {
        let result: Option<_> = try {
            let Abstract(series) = series.to_rust(cr);
            let n: usize = n.to_rust::<i64>(cr).try_into().ok()?;
            let with_replacement: bool = with_replacement.to_rust(cr);
            let shuffle: bool = shuffle.to_rust(cr);
            let seed: Option<Result<u64,_>> = seed.to_rust::<Option<i64>>(cr).map(|seed| seed.try_into());
            let seed: Option<u64> = seed.map_or(Ok(None), |seed| seed.map(Some)).ok()?;

            series.sample_n(n, with_replacement, shuffle, seed)
            .map(Abstract).map_err(|err| err.to_string())
        };
        result.to_ocaml(cr)
    }

    fn rust_series_eq(cr, series: OCamlRef<DynBox<Series>>, other: OCamlRef<DynBox<Series>>) -> OCaml<Result<DynBox<Series>,String>> {
        series_binary_op_result(cr, series, other, |a, b| a.equal(&b).map(|series| series.into_series()))
    }

    fn rust_series_neq(cr, series: OCamlRef<DynBox<Series>>, other: OCamlRef<DynBox<Series>>) -> OCaml<Result<DynBox<Series>,String>> {
        series_binary_op_result(cr, series, other, |a, b| a.not_equal(&b).map(|series| series.into_series()))
    }

    fn rust_series_gt(cr, series: OCamlRef<DynBox<Series>>, other: OCamlRef<DynBox<Series>>) -> OCaml<Result<DynBox<Series>,String>> {
        series_binary_op_result(cr, series, other, |a, b| a.gt(&b).map(|series| series.into_series()))
    }

    fn rust_series_gt_eq(cr, series: OCamlRef<DynBox<Series>>, other: OCamlRef<DynBox<Series>>) -> OCaml<Result<DynBox<Series>,String>> {
        series_binary_op_result(cr, series, other, |a, b| a.gt_eq(&b).map(|series| series.into_series()))
    }

    fn rust_series_lt(cr, series: OCamlRef<DynBox<Series>>, other: OCamlRef<DynBox<Series>>) -> OCaml<Result<DynBox<Series>,String>> {
        series_binary_op_result(cr, series, other, |a, b| a.lt(&b).map(|series| series.into_series()))
    }

    fn rust_series_lt_eq(cr, series: OCamlRef<DynBox<Series>>, other: OCamlRef<DynBox<Series>>) -> OCaml<Result<DynBox<Series>,String>> {
        series_binary_op_result(cr, series, other, |a, b| a.lt_eq(&b).map(|series| series.into_series()))
    }

    fn rust_series_add(cr, series: OCamlRef<DynBox<Series>>, other: OCamlRef<DynBox<Series>>) -> OCaml<DynBox<Series>> {
        series_binary_op(cr, series, other, |a, b| a + b)
    }

    fn rust_series_sub(cr, series: OCamlRef<DynBox<Series>>, other: OCamlRef<DynBox<Series>>) -> OCaml<DynBox<Series>> {
        series_binary_op(cr, series, other, |a, b| a - b)
    }

    fn rust_series_mul(cr, series: OCamlRef<DynBox<Series>>, other: OCamlRef<DynBox<Series>>) -> OCaml<DynBox<Series>> {
        series_binary_op(cr, series, other, |a, b| a * b)
    }

    fn rust_series_div(cr, series: OCamlRef<DynBox<Series>>, other: OCamlRef<DynBox<Series>>) -> OCaml<DynBox<Series>> {
        series_binary_op(cr, series, other, |a, b| a / b)
    }

    fn rust_series_to_string_hum(cr, series: OCamlRef<DynBox<Series>>) -> OCaml<String> {
        let Abstract(series) = series.to_rust(cr);
        ToString::to_string(&series).to_ocaml(cr)
    }
}
