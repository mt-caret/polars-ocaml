use crate::utils::*;
use chrono::naive::{NaiveDate, NaiveDateTime};
use ocaml_interop::{
    BoxRoot, DynBox, OCaml, OCamlBytes, OCamlFloat, OCamlInt, OCamlList, OCamlRef, OCamlRuntime,
    ToOCaml,
};
use polars::prelude::prelude::*;
use polars::prelude::*;
use polars_ocaml_macros::ocaml_interop_export;

// TODO: These helper fuctions should be replaced by a macro.
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

// The rough idea of functions which take GADTDataType is that the argument or
// the return type depends on the value of the GADTDataType, so we hide the
// actual type of the argument or return value behind a DummyBoxRoot and
// interpret it with or produce the value of the correct type later.
//
// In the case of series_new takes a vec of DummyBoxRoots, and we recurse when
// data_type turns out to be a GADTDataType::List(inner_data_type).
fn series_new(
    cr: &mut &mut OCamlRuntime,
    data_type: &GADTDataType,
    name: String,
    // TODO: can we just pass a DummyBoxRoot here instead of a Vec, and save a copy?
    values: Vec<DummyBoxRoot>,
    are_values_options: bool,
) -> Result<Series, String> {
    macro_rules! create_series {
        ($ocaml_type:ty, $rust_type:ty) => {{
            if are_values_options {
                let values: Vec<Option<$rust_type>> = values
                    .into_iter()
                    .map(|v| v.interpret::<Option<$ocaml_type>>(cr).to_rust())
                    .collect();
                Ok(Series::new(&name, values))
            } else {
                let values: Vec<$rust_type> = values
                    .into_iter()
                    .map(|v| v.interpret::<$ocaml_type>(cr).to_rust())
                    .collect();
                Ok(Series::new(&name, values))
            }
        }};
    }

    macro_rules! create_int_series {
        ($rust_type:ty) => {{
            if are_values_options {
                let values = values
                    .into_iter()
                    .map(
                        |v| match v.interpret::<Option<OCamlInt>>(cr).to_rust::<Option<i64>>() {
                            None => Ok(None),
                            Some(int) => int.try_into().map(Some),
                        },
                    )
                    .collect::<Result<Vec<Option<$rust_type>>, _>>()
                    .map_err(|err| err.to_string())?;
                Ok(Series::new(&name, values))
            } else {
                let values = values
                    .into_iter()
                    .map(|v| v.interpret::<OCamlInt>(cr).to_rust::<i64>().try_into())
                    .collect::<Result<Vec<$rust_type>, _>>()
                    .map_err(|err| err.to_string())?;
                Ok(Series::new(&name, values))
            }
        }};
    }

    match data_type {
        GADTDataType::Boolean => create_series!(bool, bool),
        GADTDataType::UInt8 => create_int_series!(u8),
        GADTDataType::UInt16 => create_int_series!(u16),
        GADTDataType::UInt32 => create_int_series!(u32),
        GADTDataType::UInt64 => create_int_series!(u64),
        GADTDataType::Int8 => create_int_series!(i8),
        GADTDataType::Int16 => create_int_series!(i16),
        GADTDataType::Int32 => create_int_series!(i32),
        GADTDataType::Int64 => create_int_series!(i64),
        GADTDataType::Float32 => {
            if are_values_options {
                let values: Vec<Option<f32>> = values
                    .into_iter()
                    .map(|v| {
                        v.interpret::<Option<OCamlFloat>>(cr)
                            .to_rust::<Option<f64>>()
                            .map(|f64| f64 as f32)
                    })
                    .collect();
                Ok(Series::new(&name, values))
            } else {
                let values: Vec<f32> = values
                    .into_iter()
                    .map(|v| v.interpret::<OCamlFloat>(cr).to_rust::<f64>() as f32)
                    .collect();
                Ok(Series::new(&name, values))
            }
        }
        GADTDataType::Float64 => create_series!(OCamlFloat, f64),
        GADTDataType::Utf8 => create_series!(String, String),
        GADTDataType::Binary => create_series!(OCamlBytes, Vec<u8>),
        GADTDataType::List(data_type) => {
            if are_values_options {
                let values: Vec<Option<Series>> = values
                    .into_iter()
                    .map(|v| {
                        match v.interpret::<Option<OCamlList<DummyBoxRoot>>>(cr).to_rust() {
                            None => Ok(None),
                            Some(list) => series_new(
                                cr,
                                data_type,
                                name.clone(),
                                list,
                                // The OCaml GADT's type assumes all values are non-null for simplicity,
                                // but we expose a top-level function that allows the outermost layer
                                // to be optional. So, all recursive layers are non-optional so
                                // are_values_options=false.
                                false,
                            )
                            .map(Some),
                        }
                    })
                    .collect::<Result<Vec<Option<Series>>, _>>()?;
                Ok(Series::new(&name, values))
            } else {
                let values: Vec<Series> = values
                    .into_iter()
                    .map(|v| {
                        let list = v.interpret::<OCamlList<DummyBoxRoot>>(cr).to_rust();
                        series_new(
                            cr,
                            data_type,
                            name.clone(),
                            list,
                            // See call above to series_new
                            false,
                        )
                    })
                    .collect::<Result<Vec<Series>, _>>()?;

                Ok(Series::new(&name, values))
            }
        }
    }
}

#[ocaml_interop_export(raise_on_err)]
fn rust_series_new(
    cr: &mut &mut OCamlRuntime,
    data_type: OCamlRef<GADTDataType>,
    name: OCamlRef<String>,
    values: OCamlRef<OCamlList<DummyBoxRoot>>,
) -> OCaml<DynBox<Series>> {
    let name: String = name.to_rust(cr);
    let data_type: GADTDataType = data_type.to_rust(cr);
    let values: Vec<DummyBoxRoot> = values.to_rust(cr);

    let series = series_new(cr, &data_type, name, values, false)?;

    OCaml::box_value(cr, series)
}

#[ocaml_interop_export(raise_on_err)]
fn rust_series_new_option(
    cr: &mut &mut OCamlRuntime,
    data_type: OCamlRef<GADTDataType>,
    name: OCamlRef<String>,
    values: OCamlRef<OCamlList<DummyBoxRoot>>,
) -> OCaml<DynBox<Series>> {
    let name: String = name.to_rust(cr);
    let data_type: GADTDataType = data_type.to_rust(cr);
    let values: Vec<DummyBoxRoot> = values.to_rust(cr);

    let series = series_new(cr, &data_type, name, values, true)?;

    OCaml::box_value(cr, series)
}

#[ocaml_interop_export(raise_on_err)]
fn rust_series_new_array(
    cr: &mut &mut OCamlRuntime,
    data_type: OCamlRef<GADTDataType>,
    name: OCamlRef<String>,
    values: OCamlRef<OCamlArray<DummyBoxRoot>>,
) -> OCaml<DynBox<Series>> {
    let name: String = name.to_rust(cr);
    let data_type: GADTDataType = data_type.to_rust(cr);
    let values: Vec<DummyBoxRoot> = values.to_rust(cr);

    let series = series_new(cr, &data_type, name, values, false)?;

    OCaml::box_value(cr, series)
}

#[ocaml_interop_export(raise_on_err)]
fn rust_series_new_option_array(
    cr: &mut &mut OCamlRuntime,
    data_type: OCamlRef<GADTDataType>,
    name: OCamlRef<String>,
    values: OCamlRef<OCamlArray<DummyBoxRoot>>,
) -> OCaml<DynBox<Series>> {
    let name: String = name.to_rust(cr);
    let data_type: GADTDataType = data_type.to_rust(cr);
    let values: Vec<DummyBoxRoot> = values.to_rust(cr);

    let series = series_new(cr, &data_type, name, values, true)?;

    OCaml::box_value(cr, series)
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
fn rust_series_new_datetime_option(
    cr: &mut &mut OCamlRuntime,
    name: OCamlRef<String>,
    values: OCamlRef<OCamlList<Option<DynBox<NaiveDateTime>>>>,
) -> OCaml<DynBox<Series>> {
    let name: String = name.to_rust(cr);
    let values: Vec<Option<NaiveDateTime>> = values
        .to_rust::<Vec<Option<_>>>(cr)
        .into_iter()
        .map(|o| o.map(|Abstract(v)| v))
        .collect();

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
fn rust_series_new_date_option(
    cr: &mut &mut OCamlRuntime,
    name: OCamlRef<String>,
    values: OCamlRef<OCamlList<Option<DynBox<NaiveDate>>>>,
) -> OCaml<DynBox<Series>> {
    let name: String = name.to_rust(cr);
    let values: Vec<Option<NaiveDate>> = values
        .to_rust::<Vec<Option<_>>>(cr)
        .into_iter()
        .map(|o| o.map(|Abstract(v)| v))
        .collect();

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

fn series_to_boxrooted_ocaml_list<'a>(
    cr: &mut &'a mut OCamlRuntime,
    data_type: &GADTDataType,
    series: Series,
    allow_nulls: bool,
) -> Result<DummyBoxRoot, String> {
    if !allow_nulls && series.null_count() > 0 {
        return Err(format!(
            "Series contains {} null values, expected none",
            series.null_count()
        )
        .to_string());
    }

    macro_rules! create_boxrooted_ocaml_list {
        ($rust_type:ty, $ocaml_type:ty, $body:expr) => {{
            let vec: Vec<$rust_type> = $body;

            let return_value: BoxRoot<OCamlList<$ocaml_type>> = vec.to_ocaml(cr).root();

            Ok(unsafe { DummyBoxRoot::new(return_value) })
        }};
    }

    macro_rules! create_boxrooted_ocaml_list_handle_nulls {
        ($rust_type:ty, $ocaml_type:ty, $body_allow_nulls:expr, $body_disallow_nulls:expr) => {{
            if allow_nulls {
                create_boxrooted_ocaml_list!(
                    Option<$rust_type>,
                    Option<$ocaml_type>,
                    $body_allow_nulls
                )
            } else {
                create_boxrooted_ocaml_list!($rust_type, $ocaml_type, $body_disallow_nulls)
            }
        }};
    }

    macro_rules! ocaml_intable {
        ($rust_type:ty, $ca:expr) => {{
            let ca = $ca.map_err(|err| err.to_string())?;

            create_boxrooted_ocaml_list_handle_nulls!(
                OCamlIntable<$rust_type>,
                OCamlInt,
                ca.into_iter().map(|n| n.map(OCamlIntable)).collect(),
                {
                    let mut buf = Vec::with_capacity(ca.len());
                    for arr in ca.downcast_iter() {
                        buf.extend(arr.values_iter().map(|n| OCamlIntable(*n)))
                    }
                    buf
                }
            )
        }};
    }

    match data_type {
        GADTDataType::Boolean => {
            let ca = series.bool().map_err(|err| err.to_string())?;

            create_boxrooted_ocaml_list_handle_nulls!(bool, bool, ca.into_iter().collect(), {
                let mut buf = Vec::with_capacity(ca.len());
                for arr in ca.downcast_iter() {
                    buf.extend(arr.values_iter())
                }
                buf
            })
        }
        GADTDataType::UInt8 => ocaml_intable!(u8, series.u8()),
        GADTDataType::UInt16 => ocaml_intable!(u16, series.u16()),
        GADTDataType::UInt32 => ocaml_intable!(u32, series.u32()),
        GADTDataType::UInt64 => ocaml_intable!(u64, series.u64()),
        GADTDataType::Int8 => ocaml_intable!(i8, series.i8()),
        GADTDataType::Int16 => ocaml_intable!(i16, series.i16()),
        GADTDataType::Int32 => ocaml_intable!(i32, series.i32()),
        GADTDataType::Int64 => ocaml_intable!(i64, series.i64()),
        GADTDataType::Float32 => {
            let ca = series.f32().map_err(|err| err.to_string())?;

            create_boxrooted_ocaml_list_handle_nulls!(
                f64,
                OCamlFloat,
                ca.into_iter()
                    .map(|f32o| f32o.map(|f32| f32 as f64))
                    .collect(),
                {
                    let mut buf = Vec::with_capacity(ca.len());
                    for arr in ca.downcast_iter() {
                        buf.extend(arr.values_iter().map(|f32| *f32 as f64))
                    }
                    buf
                }
            )
        }
        GADTDataType::Float64 => {
            let ca = series.f64().map_err(|err| err.to_string())?;

            create_boxrooted_ocaml_list_handle_nulls!(f64, OCamlFloat, ca.into_iter().collect(), {
                let mut buf = Vec::with_capacity(ca.len());
                for arr in ca.downcast_iter() {
                    buf.extend(arr.values_iter())
                }
                buf
            })
        }
        GADTDataType::Utf8 => {
            let ca = series.utf8().map_err(|err| err.to_string())?;
            create_boxrooted_ocaml_list_handle_nulls!(&str, String, ca.into_iter().collect(), {
                let mut buf = Vec::with_capacity(ca.len());
                for arr in ca.downcast_iter() {
                    buf.extend(arr.values_iter())
                }
                buf
            })
        }
        GADTDataType::Binary => {
            let ca = series.binary().map_err(|err| err.to_string())?;

            create_boxrooted_ocaml_list_handle_nulls!(&[u8], String, ca.into_iter().collect(), {
                let mut buf = Vec::with_capacity(ca.len());
                for arr in ca.downcast_iter() {
                    buf.extend(arr.values_iter())
                }
                buf
            })
        }
        GADTDataType::List(data_type) => {
            let ca = series.list().map_err(|err| err.to_string())?;

            create_boxrooted_ocaml_list_handle_nulls!(
                DummyBoxRoot,
                DummyBoxRoot,
                ca.into_iter()
                    .map(|serieso| match serieso {
                        None => Ok(None),
                        Some(series) => {
                            // See comment on similar recursive call in series_new on why allow_nulls=false
                            series_to_boxrooted_ocaml_list(cr, data_type, series, false).map(Some)
                        }
                    })
                    .collect::<Result<_, _>>()?,
                {
                    let mut buf = Vec::with_capacity(ca.len());

                    // Based off of
                    // https://github.com/pola-rs/polars/blob/b91cd2d3fa42f80319d23d057fd9691ba474bd61/crates/polars-core/src/chunked_array/iterator/mod.rs#L304
                    // TBH I don't fully understand how this works, but hey, it's quickcheck tested!
                    for arr in ca
                        .downcast_iter()
                        .flat_map(|arr| arr.iter().unwrap_required())
                    {
                        let series = unsafe {
                            Series::from_chunks_and_dtype_unchecked(
                                "",
                                vec![arr],
                                &ca.inner_dtype(),
                            )
                        };
                        // See comment on similar recursive call in series_new on why allow_nulls=false
                        buf.push(series_to_boxrooted_ocaml_list(
                            cr, data_type, series, false,
                        )?)
                    }

                    buf
                }
            )
        }
    }
}

#[ocaml_interop_export(raise_on_err)]
fn rust_series_to_list(
    cr: &mut &mut OCamlRuntime,
    data_type: OCamlRef<GADTDataType>,
    series: OCamlRef<DynBox<Series>>,
) -> OCaml<DummyBoxRoot> {
    let data_type: GADTDataType = data_type.to_rust(cr);
    let Abstract(series) = series.to_rust(cr);

    series_to_boxrooted_ocaml_list(cr, &data_type, series, false)?.to_ocaml(cr)
}

#[ocaml_interop_export(raise_on_err)]
fn rust_series_to_option_list(
    cr: &mut &mut OCamlRuntime,
    data_type: OCamlRef<GADTDataType>,
    series: OCamlRef<DynBox<Series>>,
) -> OCaml<DummyBoxRoot> {
    let data_type: GADTDataType = data_type.to_rust(cr);
    let Abstract(series) = series.to_rust(cr);

    series_to_boxrooted_ocaml_list(cr, &data_type, series, true)?.to_ocaml(cr)
}

fn series_get<'a>(
    cr: &mut &'a mut OCamlRuntime,
    data_type: &GADTDataType,
    series: Series,
    index: usize,
) -> Result<DummyBoxRoot, String> {
    macro_rules! extract_value {
        ($rust_type:ty, $ocaml_type:ty, $body:expr) => {{
            let value: Option<$rust_type> = $body;

            let return_value: BoxRoot<Option<$ocaml_type>> = value.to_ocaml(cr).root();

            Ok(unsafe { DummyBoxRoot::new(return_value) })
        }};
    }

    macro_rules! extract_int_value {
        ($rust_type:ty, $body:expr) => {{
            extract_value!(
                OCamlIntable<$rust_type>,
                OCamlInt,
                $body
                    .map_err(|err| err.to_string())?
                    .get(index)
                    .map(OCamlIntable)
            )
        }};
    }

    match data_type {
        GADTDataType::Boolean => extract_value!(
            bool,
            bool,
            series.bool().map_err(|err| err.to_string())?.get(index)
        ),
        GADTDataType::Int8 => extract_int_value!(i8, series.i8()),
        GADTDataType::Int16 => extract_int_value!(i16, series.i16()),
        GADTDataType::Int32 => extract_int_value!(i32, series.i32()),
        GADTDataType::Int64 => extract_int_value!(i64, series.i64()),
        GADTDataType::UInt8 => extract_int_value!(u8, series.u8()),
        GADTDataType::UInt16 => extract_int_value!(u16, series.u16()),
        GADTDataType::UInt32 => extract_int_value!(u32, series.u32()),
        GADTDataType::UInt64 => extract_int_value!(u64, series.u64()),
        GADTDataType::Float32 => extract_value!(
            f64,
            OCamlFloat,
            series
                .f32()
                .map_err(|err| err.to_string())?
                .get(index)
                .map(|f32| f32 as f64)
        ),
        GADTDataType::Float64 => extract_value!(
            f64,
            OCamlFloat,
            series.f64().map_err(|err| err.to_string())?.get(index)
        ),
        GADTDataType::Utf8 => extract_value!(
            &str,
            String,
            series.utf8().map_err(|err| err.to_string())?.get(index)
        ),
        GADTDataType::Binary => extract_value!(
            &[u8],
            String,
            series.binary().map_err(|err| err.to_string())?.get(index)
        ),
        GADTDataType::List(data_type) => extract_value!(
            DummyBoxRoot,
            DummyBoxRoot,
            match series.list().map_err(|err| err.to_string())?.get(index) {
                None => Ok(None),
                Some(series) => {
                    series_to_boxrooted_ocaml_list(cr, data_type, series, false).map(Some)
                }
            }?
        ),
    }
}

#[ocaml_interop_export(raise_on_err)]
fn rust_series_get(
    cr: &mut &mut OCamlRuntime,
    data_type: OCamlRef<GADTDataType>,
    series: OCamlRef<DynBox<Series>>,
    index: OCamlRef<OCamlInt>,
) -> OCaml<DummyBoxRoot> {
    let data_type: GADTDataType = data_type.to_rust(cr);
    let Abstract(series) = series.to_rust(cr);
    let index = index.to_rust::<Coerce<_, i64, usize>>(cr).get()?;

    series_get(cr, &data_type, series, index)?.to_ocaml(cr)
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
fn rust_series_dtype(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<Series>>,
) -> OCaml<DataType> {
    let Abstract(series) = series.to_rust(cr);
    PolarsDataType(series.dtype().clone()).to_ocaml(cr)
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

#[ocaml_interop_export(raise_on_err)]
fn rust_series_head(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<Series>>,
    length: OCamlRef<Option<OCamlInt>>,
) -> OCaml<DynBox<Series>> {
    let Abstract(series) = series.to_rust(cr);
    let length = length
        .to_rust::<Coerce<_, Option<i64>, Option<usize>>>(cr)
        .get()?;

    Abstract(series.head(length)).to_ocaml(cr)
}

#[ocaml_interop_export(raise_on_err)]
fn rust_series_tail(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<Series>>,
    length: OCamlRef<Option<OCamlInt>>,
) -> OCaml<DynBox<Series>> {
    let Abstract(series) = series.to_rust(cr);
    let length = length
        .to_rust::<Coerce<_, Option<i64>, Option<usize>>>(cr)
        .get()?;

    Abstract(series.tail(length)).to_ocaml(cr)
}

#[ocaml_interop_export(raise_on_err)]
fn rust_series_sample_n(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<Series>>,
    n: OCamlRef<OCamlInt>,
    with_replacement: OCamlRef<bool>,
    shuffle: OCamlRef<bool>,
    seed: OCamlRef<Option<OCamlInt>>,
) -> OCaml<Result<DynBox<Series>, String>> {
    let Abstract(series) = series.to_rust(cr);
    let n = n.to_rust::<Coerce<_, i64, usize>>(cr).get()?;
    let with_replacement: bool = with_replacement.to_rust(cr);
    let shuffle: bool = shuffle.to_rust(cr);
    let seed = seed
        .to_rust::<Coerce<_, Option<i64>, Option<u64>>>(cr)
        .get()?;

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
