use crate::utils::*;
use chrono::naive::{NaiveDate, NaiveDateTime};
use ocaml_interop::{
    impl_to_ocaml_variant, DynBox, OCaml, OCamlBytes, OCamlFloat, OCamlInt, OCamlInt32, OCamlList,
    OCamlRef, OCamlRuntime, ToOCaml,
};
use polars::prelude::prelude::*;
use polars::prelude::*;
use polars_ocaml_macros::ocaml_interop_export;

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

pub enum TypedList {
    Int(Vec<Option<i64>>),
    Int32(Vec<Option<i32>>),
    Float(Vec<Option<f64>>),
    String(Vec<Option<String>>),
    Bytes(Vec<Option<Vec<u8>>>),
}

impl_to_ocaml_variant! {
    // Optionally, if Rust and OCaml types don't match:
    // RustType => OCamlType { ... }
    TypedList {
        TypedList::Int(l: OCamlList<Option<OCamlInt>>),
        TypedList::Int32(l: OCamlList<Option<OCamlInt32>>),
        TypedList::Float(l: OCamlList<Option<OCamlFloat>>),
        TypedList::String(l: OCamlList<Option<String>>),
        TypedList::Bytes(l: OCamlList<Option<OCamlBytes>>),
    }
}

#[ocaml_interop_export]
fn rust_series_new_int(
    cr: &mut &mut OCamlRuntime,
    name: OCamlRef<String>,
    values: OCamlRef<OCamlList<OCamlInt>>,
) -> OCaml<DynBox<Series>> {
    let name: String = name.to_rust(cr);
    let values: Vec<i64> = values.to_rust(cr);

    OCaml::box_value(cr, Series::new(&name, values))
}

#[ocaml_interop_export]
fn rust_series_new_int_option(
    cr: &mut &mut OCamlRuntime,
    name: OCamlRef<String>,
    values: OCamlRef<OCamlList<Option<OCamlInt>>>,
) -> OCaml<DynBox<Series>> {
    let name: String = name.to_rust(cr);
    let values: Vec<Option<i64>> = values.to_rust(cr);
    OCaml::box_value(cr, Series::new(&name, values))
}

#[ocaml_interop_export]
fn rust_series_new_float(
    cr: &mut &mut OCamlRuntime,
    name: OCamlRef<String>,
    values: OCamlRef<OCamlList<OCamlFloat>>,
) -> OCaml<DynBox<Series>> {
    let name: String = name.to_rust(cr);
    let values: Vec<f64> = values.to_rust(cr);
    OCaml::box_value(cr, Series::new(&name, values))
}

#[ocaml_interop_export]
fn rust_series_new_float_option(
    cr: &mut &mut OCamlRuntime,
    name: OCamlRef<String>,
    values: OCamlRef<OCamlList<Option<OCamlFloat>>>,
) -> OCaml<DynBox<Series>> {
    let name: String = name.to_rust(cr);
    let values: Vec<Option<f64>> = values.to_rust(cr);
    OCaml::box_value(cr, Series::new(&name, values))
}

#[ocaml_interop_export]
fn rust_series_new_string(
    cr: &mut &mut OCamlRuntime,
    name: OCamlRef<String>,
    values: OCamlRef<OCamlList<String>>,
) -> OCaml<DynBox<Series>> {
    let name: String = name.to_rust(cr);
    let values: Vec<String> = values.to_rust(cr);
    OCaml::box_value(cr, Series::new(&name, values))
}

#[ocaml_interop_export]
fn rust_series_new_string_option(
    cr: &mut &mut OCamlRuntime,
    name: OCamlRef<String>,
    values: OCamlRef<OCamlList<Option<String>>>,
) -> OCaml<DynBox<Series>> {
    let name: String = name.to_rust(cr);
    let values: Vec<Option<String>> = values.to_rust(cr);
    OCaml::box_value(cr, Series::new(&name, values))
}

#[ocaml_interop_export]
fn rust_series_new_bool(
    cr: &mut &mut OCamlRuntime,
    name: OCamlRef<String>,
    values: OCamlRef<OCamlList<bool>>,
) -> OCaml<DynBox<Series>> {
    let name: String = name.to_rust(cr);
    let values: Vec<bool> = values.to_rust(cr);
    OCaml::box_value(cr, Series::new(&name, values))
}

#[ocaml_interop_export]
fn rust_series_new_bool_option(
    cr: &mut &mut OCamlRuntime,
    name: OCamlRef<String>,
    values: OCamlRef<OCamlList<Option<bool>>>,
) -> OCaml<DynBox<Series>> {
    let name: String = name.to_rust(cr);
    let values: Vec<Option<bool>> = values.to_rust(cr);
    OCaml::box_value(cr, Series::new(&name, values))
}

#[ocaml_interop_export]
fn rust_series_new_datetime(
    cr: &mut &mut OCamlRuntime,
    name: OCamlRef<String>,
    values: OCamlRef<OCamlList<DynBox<NaiveDateTime>>>,
) -> OCaml<DynBox<Series>> {
    let name: String = name.to_rust(cr);
    let values = unwrap_abstract_vec(values.to_rust(cr));
    OCaml::box_value(cr, Series::new(&name, values))
}

#[ocaml_interop_export]
fn rust_series_new_date(
    cr: &mut &mut OCamlRuntime,
    name: OCamlRef<String>,
    values: OCamlRef<OCamlList<DynBox<NaiveDate>>>,
) -> OCaml<DynBox<Series>> {
    let name: String = name.to_rust(cr);
    let values = unwrap_abstract_vec(values.to_rust(cr));
    OCaml::box_value(cr, Series::new(&name, values))
}

#[ocaml_interop_export]
fn rust_series_date_range(
    cr: &mut &mut OCamlRuntime,
    name: OCamlRef<String>,
    start: OCamlRef<DynBox<NaiveDateTime>>,
    stop: OCamlRef<DynBox<NaiveDateTime>>,
    every: OCamlRef<Option<String>>,
    cast_to_date: OCamlRef<bool>,
) -> OCaml<Result<DynBox<Series>, String>> {
    let name: String = name.to_rust(cr);

    let Abstract(start) = start.to_rust(cr);
    let Abstract(stop) = stop.to_rust(cr);

    let every: String = every
        .to_rust::<Option<String>>(cr)
        .unwrap_or("1d".to_string());

    let cast_to_date: bool = cast_to_date.to_rust(cr);

    let series = date_range(
        &name,
        start,
        stop,
        Duration::parse(&every),
        ClosedWindow::Both,
        TimeUnit::Milliseconds,
        None,
    )
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

#[ocaml_interop_export]
fn rust_series_name(cr: &mut &mut OCamlRuntime, series: OCamlRef<DynBox<Series>>) -> OCaml<String> {
    let Abstract(series) = series.to_rust(cr);
    series.name().to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_series_rename(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<Series>>,
    name: OCamlRef<String>,
) -> OCaml<DynBox<Series>> {
    let Abstract(mut series) = series.to_rust(cr);
    let name: String = name.to_rust(cr);

    let _ = series.rename(&name);

    OCaml::box_value(cr, series)
}

#[ocaml_interop_export]
fn rust_series_to_data_frame(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<Series>>,
) -> OCaml<DynBox<DataFrame>> {
    let Abstract(series) = series.to_rust(cr);
    OCaml::box_value(cr, series.into_frame())
}

#[ocaml_interop_export]
fn rust_series_sort(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<Series>>,
    descending: OCamlRef<bool>,
) -> OCaml<DynBox<Series>> {
    let Abstract(series) = series.to_rust(cr);
    let descending: bool = descending.to_rust(cr);

    OCaml::box_value(cr, series.sort(descending))
}

#[ocaml_interop_export]
fn rust_series_head(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<Series>>,
    length: OCamlRef<Option<OCamlInt>>,
) -> OCaml<DynBox<Series>> {
    let Abstract(series) = series.to_rust(cr);
    let length = length
        .to_rust::<Coerce<_, Option<i64>, Option<usize>>>(cr)
        .get();

    Abstract(series.head(length)).to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_series_tail(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<Series>>,
    length: OCamlRef<Option<OCamlInt>>,
) -> OCaml<DynBox<Series>> {
    let Abstract(series) = series.to_rust(cr);
    let length = length
        .to_rust::<Coerce<_, Option<i64>, Option<usize>>>(cr)
        .get();

    Abstract(series.tail(length)).to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_series_sample_n(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<Series>>,
    n: OCamlRef<OCamlInt>,
    with_replacement: OCamlRef<bool>,
    shuffle: OCamlRef<bool>,
    seed: OCamlRef<Option<OCamlInt>>,
) -> OCaml<Result<DynBox<Series>, String>> {
    let Abstract(series) = series.to_rust(cr);
    let n = n.to_rust::<Coerce<_, i64, usize>>(cr).get();
    let with_replacement: bool = with_replacement.to_rust(cr);
    let shuffle: bool = shuffle.to_rust(cr);
    let seed = seed
        .to_rust::<Coerce<_, Option<i64>, Option<u64>>>(cr)
        .get();

    series
        .sample_n(n, with_replacement, shuffle, seed)
        .map(Abstract)
        .map_err(|err| err.to_string())
        .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_series_fill_null_with_strategy(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<Series>>,
    strategy: OCamlRef<FillNullStrategy>,
) -> OCaml<Result<DynBox<Series>, String>> {
    let Abstract(series) = series.to_rust(cr);
    let PolarsFillNullStrategy(strategy) = strategy.to_rust(cr);

    series
        .fill_null(strategy)
        .map(Abstract)
        .map_err(|err| err.to_string())
        .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_series_interpolate(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<Series>>,
    method: OCamlRef<InterpolationMethod>,
) -> OCaml<DynBox<Series>> {
    let Abstract(series) = series.to_rust(cr);
    let PolarsInterpolationMethod(method) = method.to_rust(cr);

    Abstract(interpolate(&series, method)).to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_series_eq(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<Series>>,
    other: OCamlRef<DynBox<Series>>,
) -> OCaml<Result<DynBox<Series>, String>> {
    series_binary_op_result(cr, series, other, |a, b| {
        a.equal(&b).map(|series| series.into_series())
    })
}

#[ocaml_interop_export]
fn rust_series_neq(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<Series>>,
    other: OCamlRef<DynBox<Series>>,
) -> OCaml<Result<DynBox<Series>, String>> {
    series_binary_op_result(cr, series, other, |a, b| {
        a.not_equal(&b).map(|series| series.into_series())
    })
}

#[ocaml_interop_export]
fn rust_series_gt(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<Series>>,
    other: OCamlRef<DynBox<Series>>,
) -> OCaml<Result<DynBox<Series>, String>> {
    series_binary_op_result(cr, series, other, |a, b| {
        a.gt(&b).map(|series| series.into_series())
    })
}

#[ocaml_interop_export]
fn rust_series_gt_eq(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<Series>>,
    other: OCamlRef<DynBox<Series>>,
) -> OCaml<Result<DynBox<Series>, String>> {
    series_binary_op_result(cr, series, other, |a, b| {
        a.gt_eq(&b).map(|series| series.into_series())
    })
}

#[ocaml_interop_export]
fn rust_series_lt(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<Series>>,
    other: OCamlRef<DynBox<Series>>,
) -> OCaml<Result<DynBox<Series>, String>> {
    series_binary_op_result(cr, series, other, |a, b| {
        a.lt(&b).map(|series| series.into_series())
    })
}

#[ocaml_interop_export]
fn rust_series_lt_eq(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<Series>>,
    other: OCamlRef<DynBox<Series>>,
) -> OCaml<Result<DynBox<Series>, String>> {
    series_binary_op_result(cr, series, other, |a, b| {
        a.lt_eq(&b).map(|series| series.into_series())
    })
}

#[ocaml_interop_export]
fn rust_series_add(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<Series>>,
    other: OCamlRef<DynBox<Series>>,
) -> OCaml<DynBox<Series>> {
    series_binary_op(cr, series, other, |a, b| a + b)
}

#[ocaml_interop_export]
fn rust_series_sub(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<Series>>,
    other: OCamlRef<DynBox<Series>>,
) -> OCaml<DynBox<Series>> {
    series_binary_op(cr, series, other, |a, b| a - b)
}

#[ocaml_interop_export]
fn rust_series_mul(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<Series>>,
    other: OCamlRef<DynBox<Series>>,
) -> OCaml<DynBox<Series>> {
    series_binary_op(cr, series, other, |a, b| a * b)
}

#[ocaml_interop_export]
fn rust_series_div(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<Series>>,
    other: OCamlRef<DynBox<Series>>,
) -> OCaml<DynBox<Series>> {
    series_binary_op(cr, series, other, |a, b| a / b)
}

#[ocaml_interop_export]
fn rust_series_to_string_hum(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<Series>>,
) -> OCaml<String> {
    let Abstract(series) = series.to_rust(cr);
    ToString::to_string(&series).to_ocaml(cr)
}

// TODO: Consider using Bigarray here instead of OCamlList to keep memory outside the
// OCaml heap and skip a copy.
// TODO: Consider mapping to smaller OCaml values like Int8, Float32, etc instead of
// casting up
#[ocaml_interop_export]
fn rust_series_to_typed_list(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<Series>>,
) -> OCaml<Result<TypedList, String>> {
    let Abstract(series) = series.to_rust(cr);

    // Get and process series based on its data type
    let result: Result<TypedList, _> = match series.dtype() {
        DataType::Int8 | DataType::Int16 => series
            .cast(&DataType::Int32)
            .and_then(|series| {
                series
                    .i32()
                    .map(|elems| elems.into_iter().collect())
                    .map(TypedList::Int32)
            })
            .map_err(|e| e.to_string()),
        DataType::Int32 => series
            .i32()
            .map_err(|e| e.to_string())
            .map(|elems| elems.into_iter().collect())
            .map(TypedList::Int32),
        DataType::Int64 => series
            .i64()
            .map_err(|e| e.to_string())
            .map(|elems| elems.into_iter().collect())
            .map(TypedList::Int),
        DataType::Float32 => series
            .cast(&DataType::Float64)
            .and_then(|series| {
                series
                    .f64()
                    .map(|elems| elems.into_iter().collect())
                    .map(TypedList::Float)
            })
            .map_err(|e| e.to_string()),
        DataType::Float64 => series
            .f64()
            .map_err(|e| e.to_string())
            .map(|elems| elems.into_iter().collect())
            .map(TypedList::Float),
        DataType::Utf8 => series
            .utf8()
            .map_err(|e| e.to_string())
            .map(|elems| {
                elems
                    .into_iter()
                    .map(|s_opt| s_opt.map(|s| s.to_string()))
                    .collect()
            })
            .map(TypedList::String),
        DataType::Binary => series
            .binary()
            .map_err(|e| e.to_string())
            .map(|elems| {
                elems
                    .into_iter()
                    .map(|s_opt| s_opt.map(|b| b.to_vec()))
                    .collect()
            })
            .map(TypedList::Bytes),
        dtype => Result::Err(format!("Unsupported dtype: {:?}", dtype)),
    };
    result.to_ocaml(cr)
}
