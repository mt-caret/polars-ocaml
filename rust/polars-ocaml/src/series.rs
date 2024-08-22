// Note: The macro modify_series_with_cast expands to contain a closure inside of a
// closure, like this:
//
// .map(|(index, v)| {
//     (
//         index as usize,
//         $perform_cast(v.interpret::<$ocaml_type>(cr).to_rust::<$rust_type>()),
//     )
// })
#![allow(clippy::redundant_closure_call)]
use crate::interop::*;
use crate::polars_types::*;

use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};
use chrono::Duration;
use ocaml_interop::{
    BoxRoot, DynBox, OCaml, OCamlBytes, OCamlException, OCamlFloat, OCamlInt, OCamlList, OCamlRef,
    OCamlRuntime, ToOCaml,
};

use polars::export::arrow::array::BooleanArray;
use polars::export::arrow::array::PrimitiveArray;
use polars::export::arrow::bitmap::Bitmap;
use polars::export::arrow::types::NativeType;
use polars::prelude::prelude::*;
use polars::prelude::*;
use polars_ocaml_macros::ocaml_interop_export;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

pub type PolarsSeries = Rc<RefCell<Series>>;

// The rough idea of functions which take GADTDataType is that the argument or
// the return type depends on the value of the GADTDataType, so we hide the
// actual type of the argument or return value behind a DummyBoxRoot and
// interpret it with or produce the value of the correct type later.
//
// In the case of series_new takes a vec of DummyBoxRoots, and we recurse when
// data_type turns out to be a GADTDataType::List(inner_data_type).
pub fn series_new<I>(
    cr: &mut &mut OCamlRuntime,
    data_type: &GADTDataType,
    name: &str,
    // TODO: can we just pass a DummyBoxRoot here instead of a Vec, and save a copy?
    values: I,
    are_values_options: bool,
) -> Result<Series, String>
where
    I: Iterator<Item = DummyBoxRoot>,
{
    macro_rules! create_series {
        ($ocaml_type:ty, $rust_type:ty) => {{
            if are_values_options {
                let values: Vec<Option<$rust_type>> = values
                    .map(|v| v.interpret::<Option<$ocaml_type>>(cr).to_rust())
                    .collect();
                Ok(Series::new(&name, values))
            } else {
                let values: Vec<$rust_type> = values
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
                    .map(|v| {
                        v.interpret::<Option<OCamlFloat>>(cr)
                            .to_rust::<Option<f64>>()
                            .map(|f64| f64 as f32)
                    })
                    .collect();
                Ok(Series::new(name, values))
            } else {
                let values: Vec<f32> = values
                    .map(|v| v.interpret::<OCamlFloat>(cr).to_rust::<f64>() as f32)
                    .collect();
                Ok(Series::new(name, values))
            }
        }
        GADTDataType::Float64 => create_series!(OCamlFloat, f64),
        GADTDataType::Utf8 => create_series!(String, String),
        GADTDataType::Binary => create_series!(OCamlBytes, Vec<u8>),
        GADTDataType::Date => {
            if are_values_options {
                let values: Vec<Option<NaiveDate>> = values
                    .into_iter()
                    .map(|v| v.interpret::<Option<DynBox<NaiveDate>>>(cr).to_rust())
                    .map(|v: Option<Abstract<NaiveDate>>| v.map(|Abstract(naive_date)| naive_date))
                    .collect();

                Ok(Series::new(name, values))
            } else {
                let values: Vec<NaiveDate> = values
                    .into_iter()
                    .map(|v| v.interpret::<DynBox<NaiveDate>>(cr).to_rust())
                    .map(|Abstract(naive_date)| naive_date)
                    .collect();

                Ok(Series::new(name, values))
            }
        }
        GADTDataType::Datetime(PolarsTimeUnit(time_unit), time_zone) => {
            if are_values_options {
                let values = values
                    .into_iter()
                    .map(|v| v.interpret::<Option<DynBox<NaiveDateTime>>>(cr).to_rust())
                    .map(|v: Option<Abstract<NaiveDateTime>>| {
                        v.map(|Abstract(naive_datetime)| naive_datetime)
                    });

                let mut logical = Logical::from_naive_datetime_options(name, values, *time_unit);

                match time_zone {
                    None => (),
                    Some(time_zone) => logical
                        .set_time_zone(time_zone.clone().get().name().to_string())
                        .map_err(|err| err.to_string())?,
                };

                Ok(logical.into())
            } else {
                let values = values
                    .into_iter()
                    .map(|v| v.interpret::<DynBox<NaiveDateTime>>(cr).to_rust())
                    .map(|Abstract(naive_datetime)| naive_datetime);

                Ok(Logical::from_naive_datetime(name, values, *time_unit).into())
            }
        }
        GADTDataType::Duration(PolarsTimeUnit(time_unit)) => {
            if are_values_options {
                let values = values
                    .into_iter()
                    .map(|v| v.interpret::<Option<DynBox<Duration>>>(cr).to_rust())
                    .map(|v: Option<Abstract<Duration>>| v.map(|Abstract(duration)| duration));

                Ok(Logical::from_duration_options(name, values, *time_unit).into())
            } else {
                let values = values
                    .into_iter()
                    .map(|v| v.interpret::<DynBox<Duration>>(cr).to_rust())
                    .map(|Abstract(duration)| duration);

                Ok(Logical::from_duration(name, values, *time_unit).into())
            }
        }
        GADTDataType::Time => {
            if are_values_options {
                let values = values
                    .into_iter()
                    .map(|v| v.interpret::<Option<DynBox<NaiveTime>>>(cr).to_rust())
                    .map(|v: Option<Abstract<NaiveTime>>| v.map(|Abstract(time)| time));

                Ok(Logical::from_naive_time_options(name, values).into())
            } else {
                let values = values
                    .into_iter()
                    .map(|v| v.interpret::<DynBox<NaiveTime>>(cr).to_rust())
                    .map(|Abstract(time)| time);

                Ok(Logical::from_naive_time(name, values).into())
            }
        }
        GADTDataType::List(data_type) => {
            let mut values = values.peekable();
            // Series creation doesn't work for empty lists and use of
            // `Series::new_empty` is suggested instead.
            // https://github.com/pola-rs/polars/pull/10558#issuecomment-1684923274
            if values.peek().is_none() {
                let data_type = DataType::List(Box::new(data_type.to_data_type()));
                return Ok(Series::new_empty(name, &data_type));
            }

            if are_values_options {
                let values: Vec<Option<Series>> = values
                    .map(|v| {
                        match v
                            .interpret::<Option<OCamlList<DummyBoxRoot>>>(cr)
                            .to_rust::<Option<Vec<DummyBoxRoot>>>()
                        {
                            None => Ok(None),
                            Some(list) => series_new(
                                cr,
                                data_type,
                                name,
                                list.into_iter(),
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
                Ok(Series::new(name, values))
            } else {
                let values: Vec<Series> = values
                    .map(|v| {
                        let list: Vec<DummyBoxRoot> =
                            v.interpret::<OCamlList<DummyBoxRoot>>(cr).to_rust();
                        series_new(
                            cr,
                            data_type,
                            name,
                            list.into_iter(), // See call above to series_new
                            false,
                        )
                    })
                    .collect::<Result<Vec<Series>, _>>()?;

                Ok(Series::new(name, values))
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
) -> OCaml<DynBox<PolarsSeries>> {
    let name: String = name.to_rust(cr);
    let data_type: GADTDataType = data_type.to_rust(cr);
    let values: Vec<DummyBoxRoot> = values.to_rust(cr);

    let series = series_new(cr, &data_type, &name, values.into_iter(), false)?;

    OCaml::box_value(cr, Rc::new(RefCell::new(series)))
}

#[ocaml_interop_export(raise_on_err)]
fn rust_series_new_option(
    cr: &mut &mut OCamlRuntime,
    data_type: OCamlRef<GADTDataType>,
    name: OCamlRef<String>,
    values: OCamlRef<OCamlList<DummyBoxRoot>>,
) -> OCaml<DynBox<PolarsSeries>> {
    let name: String = name.to_rust(cr);
    let data_type: GADTDataType = data_type.to_rust(cr);
    let values: Vec<DummyBoxRoot> = values.to_rust(cr);

    let series = series_new(cr, &data_type, &name, values.into_iter(), true)?;

    OCaml::box_value(cr, Rc::new(RefCell::new(series)))
}

#[ocaml_interop_export(raise_on_err)]
fn rust_series_new_array(
    cr: &mut &mut OCamlRuntime,
    data_type: OCamlRef<GADTDataType>,
    name: OCamlRef<String>,
    values: OCamlRef<OCamlUniformArray<DummyBoxRoot>>,
) -> OCaml<DynBox<PolarsSeries>> {
    let name: String = name.to_rust(cr);
    let data_type: GADTDataType = data_type.to_rust(cr);
    let values: Vec<DummyBoxRoot> = values.to_rust(cr);

    let series = series_new(cr, &data_type, &name, values.into_iter(), false)?;

    OCaml::box_value(cr, Rc::new(RefCell::new(series)))
}

#[ocaml_interop_export(raise_on_err)]
fn rust_series_new_option_array(
    cr: &mut &mut OCamlRuntime,
    data_type: OCamlRef<GADTDataType>,
    name: OCamlRef<String>,
    values: OCamlRef<OCamlUniformArray<DummyBoxRoot>>,
) -> OCaml<DynBox<PolarsSeries>> {
    let name: String = name.to_rust(cr);
    let data_type: GADTDataType = data_type.to_rust(cr);
    let values: Vec<DummyBoxRoot> = values.to_rust(cr);

    let series = series_new(cr, &data_type, &name, values.into_iter(), true)?;

    OCaml::box_value(cr, Rc::new(RefCell::new(series)))
}

#[ocaml_interop_export(raise_on_err)]
fn rust_series_new_float_array(
    cr: &mut &mut OCamlRuntime,
    name: OCamlRef<String>,
    values: OCamlRef<OCamlFloatArray>,
    downcast_to_f32: OCamlRef<bool>,
) -> OCaml<DynBox<PolarsSeries>> {
    let name: String = name.to_rust(cr);
    let values: Vec<f64> = values.to_rust(cr);
    let downcast_to_f32: bool = downcast_to_f32.to_rust(cr);

    let series = if downcast_to_f32 {
        Series::new(
            &name,
            values
                .into_iter()
                .map(|f64| f64 as f32)
                .collect::<Vec<f32>>(),
        )
    } else {
        Series::new(&name, values)
    };

    OCaml::box_value(cr, Rc::new(RefCell::new(series)))
}

#[ocaml_interop_export]
fn rust_series_date_range(
    cr: &mut &mut OCamlRuntime,
    name: OCamlRef<String>,
    start: OCamlRef<DynBox<NaiveDateTime>>,
    stop: OCamlRef<DynBox<NaiveDateTime>>,
    every: OCamlRef<Option<String>>,
    cast_to_date: OCamlRef<bool>,
) -> OCaml<Result<DynBox<PolarsSeries>, String>> {
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
        polars::prelude::Duration::parse(&every),
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
    .map(|s| Abstract(Rc::new(RefCell::new(s))))
    .map_err(|err| err.to_string());

    series.to_ocaml(cr)
}

fn series_to_boxrooted_ocaml_list(
    cr: &mut &mut OCamlRuntime,
    data_type: &GADTDataType,
    series: &Series,
    allow_nulls: bool,
) -> Result<DummyBoxRoot, String> {
    if !allow_nulls && series.null_count() > 0 {
        return Err(format!(
            "Series contains {} null values, expected none",
            series.null_count()
        ));
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
        GADTDataType::Date => {
            if matches!(series.dtype(), DataType::Int32) {
                let ca = series.i32().map_err(|err| err.to_string())?;

                create_boxrooted_ocaml_list_handle_nulls!(
                    Abstract<NaiveDate>,
                    DynBox<NaiveDate>,
                    ca.into_iter()
                        .map(|o| o
                            .map(|i32| Abstract(arrow2::temporal_conversions::date32_to_date(i32))))
                        .collect(),
                    {
                        let mut buf = Vec::with_capacity(ca.len());
                        for arr in ca.downcast_iter() {
                            buf.extend(arr.values_iter().map(|date| {
                                Abstract(arrow2::temporal_conversions::date32_to_date(*date))
                            }))
                        }
                        buf
                    }
                )
            } else {
                let ca = series.date().map_err(|err| err.to_string())?;

                create_boxrooted_ocaml_list_handle_nulls!(
                    Abstract<NaiveDate>,
                    DynBox<NaiveDate>,
                    ca.as_date_iter().map(|o| o.map(Abstract)).collect(),
                    {
                        let mut buf = Vec::with_capacity(ca.len());
                        for arr in ca.downcast_iter() {
                            buf.extend(arr.values_iter().map(|date| {
                                Abstract(arrow2::temporal_conversions::date32_to_date(*date))
                            }))
                        }
                        buf
                    }
                )
            }
        }
        GADTDataType::Datetime(PolarsTimeUnit(time_unit), _time_zone) => {
            let timestamp_to_datetime = match time_unit {
                TimeUnit::Nanoseconds => arrow2::temporal_conversions::timestamp_ns_to_datetime,
                TimeUnit::Microseconds => arrow2::temporal_conversions::timestamp_us_to_datetime,
                TimeUnit::Milliseconds => arrow2::temporal_conversions::timestamp_ms_to_datetime,
            };

            if matches!(series.dtype(), DataType::Int64) {
                let ca = series.i64().map_err(|err| err.to_string())?;

                create_boxrooted_ocaml_list_handle_nulls!(
                    Abstract<NaiveDateTime>,
                    DynBox<NaiveDateTime>,
                    ca.into_iter()
                        .map(|o| o.map(|i64| { Abstract(timestamp_to_datetime(i64)) }))
                        .collect(),
                    {
                        let mut buf = Vec::with_capacity(ca.len());

                        for arr in ca.downcast_iter() {
                            buf.extend(
                                arr.values_iter()
                                    .map(|timestamp| Abstract(timestamp_to_datetime(*timestamp))),
                            )
                        }
                        buf
                    }
                )
            } else {
                let ca = series.datetime().map_err(|err| err.to_string())?;

                create_boxrooted_ocaml_list_handle_nulls!(
                    Abstract<NaiveDateTime>,
                    DynBox<NaiveDateTime>,
                    ca.as_datetime_iter().map(|o| o.map(Abstract)).collect(),
                    {
                        let mut buf = Vec::with_capacity(ca.len());
                        for arr in ca.downcast_iter() {
                            buf.extend(
                                arr.values_iter()
                                    .map(|timestamp| Abstract(timestamp_to_datetime(*timestamp))),
                            )
                        }
                        buf
                    }
                )
            }
        }
        GADTDataType::Duration(PolarsTimeUnit(time_unit)) => {
            let duration_conversion = match time_unit {
                TimeUnit::Nanoseconds => arrow2::temporal_conversions::duration_ns_to_duration,
                TimeUnit::Microseconds => arrow2::temporal_conversions::duration_us_to_duration,
                TimeUnit::Milliseconds => arrow2::temporal_conversions::duration_ms_to_duration,
            };

            if matches!(series.dtype(), DataType::Int64) {
                let ca = series.i64().map_err(|err| err.to_string())?;

                create_boxrooted_ocaml_list_handle_nulls!(
                    Abstract<Duration>,
                    DynBox<Duration>,
                    ca.into_iter()
                        .map(|o| o.map(|i64| { Abstract(duration_conversion(i64)) }))
                        .collect(),
                    {
                        let mut buf = Vec::with_capacity(ca.len());

                        for arr in ca.downcast_iter() {
                            buf.extend(
                                arr.values_iter()
                                    .map(|timestamp| Abstract(duration_conversion(*timestamp))),
                            )
                        }
                        buf
                    }
                )
            } else {
                let ca = series.duration().map_err(|err| err.to_string())?;

                create_boxrooted_ocaml_list_handle_nulls!(
                    Abstract<Duration>,
                    DynBox<Duration>,
                    ca.downcast_iter()
                        .flat_map(|iter| iter.into_iter())
                        .map(|o| o.map(|duration| Abstract(duration_conversion(*duration))))
                        .collect(),
                    {
                        let mut buf = Vec::with_capacity(ca.len());
                        for arr in ca.downcast_iter() {
                            buf.extend(
                                arr.values_iter()
                                    .map(|timestamp| Abstract(duration_conversion(*timestamp))),
                            )
                        }
                        buf
                    }
                )
            }
        }
        GADTDataType::Time => {
            if matches!(series.dtype(), DataType::Int64) {
                let ca = series.i64().map_err(|err| err.to_string())?;

                create_boxrooted_ocaml_list_handle_nulls!(
                    Abstract<NaiveTime>,
                    DynBox<NaiveTime>,
                    ca.into_iter()
                        .map(|o| o.map(|i64| {
                            Abstract(arrow2::temporal_conversions::time64ns_to_time(i64))
                        }))
                        .collect(),
                    {
                        let mut buf = Vec::with_capacity(ca.len());

                        for arr in ca.downcast_iter() {
                            buf.extend(arr.values_iter().map(|timestamp| {
                                Abstract(arrow2::temporal_conversions::time64ns_to_time(*timestamp))
                            }))
                        }
                        buf
                    }
                )
            } else {
                let ca = series.time().map_err(|err| err.to_string())?;

                create_boxrooted_ocaml_list_handle_nulls!(
                    Abstract<NaiveTime>,
                    DynBox<NaiveTime>,
                    ca.as_time_iter().map(|o| o.map(Abstract)).collect(),
                    {
                        let mut buf = Vec::with_capacity(ca.len());
                        for arr in ca.downcast_iter() {
                            buf.extend(arr.values_iter().map(|time| {
                                Abstract(arrow2::temporal_conversions::time64ns_to_time(*time))
                            }))
                        }
                        buf
                    }
                )
            }
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
                            series_to_boxrooted_ocaml_list(cr, data_type, &series, false).map(Some)
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
                            cr, data_type, &series, false,
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
    series: OCamlRef<DynBox<PolarsSeries>>,
) -> OCaml<DummyBoxRoot> {
    let data_type: GADTDataType = data_type.to_rust(cr);
    let Abstract(series) = series.to_rust(cr);
    let series = series.borrow();

    series_to_boxrooted_ocaml_list(cr, &data_type, &series, false)?.to_ocaml(cr)
}

#[ocaml_interop_export(raise_on_err)]
fn rust_series_to_option_list(
    cr: &mut &mut OCamlRuntime,
    data_type: OCamlRef<GADTDataType>,
    series: OCamlRef<DynBox<PolarsSeries>>,
) -> OCaml<DummyBoxRoot> {
    let data_type: GADTDataType = data_type.to_rust(cr);
    let Abstract(series) = series.to_rust(cr);
    let series = series.borrow();

    series_to_boxrooted_ocaml_list(cr, &data_type, &series, true)?.to_ocaml(cr)
}

fn series_get(
    cr: &mut &mut OCamlRuntime,
    data_type: &GADTDataType,
    series: &Series,
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
        GADTDataType::Date => extract_value!(
            Abstract<NaiveDate>,
            DynBox<NaiveDate>,
            series
                .date()
                .map_err(|err| err.to_string())?
                .get(index)
                .map(|date| Abstract(arrow2::temporal_conversions::date32_to_date(date)))
        ),
        GADTDataType::Datetime(PolarsTimeUnit(time_unit), _time_zone) => extract_value!(
            Abstract<NaiveDateTime>,
            DynBox<NaiveDateTime>,
            series
                .datetime()
                .map_err(|err| err.to_string())?
                .get(index)
                .map(|datetime| {
                    let timestamp_to_datetime = match time_unit {
                        TimeUnit::Nanoseconds => {
                            arrow2::temporal_conversions::timestamp_ns_to_datetime
                        }
                        TimeUnit::Microseconds => {
                            arrow2::temporal_conversions::timestamp_us_to_datetime
                        }
                        TimeUnit::Milliseconds => {
                            arrow2::temporal_conversions::timestamp_ms_to_datetime
                        }
                    };
                    Abstract(timestamp_to_datetime(datetime))
                })
        ),
        GADTDataType::Duration(PolarsTimeUnit(time_unit)) => extract_value!(
            Abstract<Duration>,
            DynBox<Duration>,
            series
                .duration()
                .map_err(|err| err.to_string())?
                .get(index)
                .map(|duration| {
                    let duration_conversion = match time_unit {
                        TimeUnit::Nanoseconds => {
                            arrow2::temporal_conversions::duration_ns_to_duration
                        }
                        TimeUnit::Microseconds => {
                            arrow2::temporal_conversions::duration_us_to_duration
                        }
                        TimeUnit::Milliseconds => {
                            arrow2::temporal_conversions::duration_ms_to_duration
                        }
                    };
                    Abstract(duration_conversion(duration))
                })
        ),
        GADTDataType::Time => extract_value!(
            Abstract<NaiveTime>,
            DynBox<NaiveTime>,
            series
                .time()
                .map_err(|err| err.to_string())?
                .get(index)
                .map(|duration| {
                    Abstract(arrow2::temporal_conversions::time64ns_to_time(duration))
                })
        ),
        GADTDataType::List(data_type) => extract_value!(
            DummyBoxRoot,
            DummyBoxRoot,
            match series.list().map_err(|err| err.to_string())?.get(index) {
                None => Ok(None),
                Some(series) => {
                    series_to_boxrooted_ocaml_list(cr, data_type, &series, false).map(Some)
                }
            }?
        ),
    }
}

#[ocaml_interop_export(raise_on_err)]
fn rust_series_get(
    cr: &mut &mut OCamlRuntime,
    data_type: OCamlRef<GADTDataType>,
    series: OCamlRef<DynBox<PolarsSeries>>,
    index: OCamlRef<OCamlInt>,
) -> OCaml<DummyBoxRoot> {
    let data_type: GADTDataType = data_type.to_rust(cr);
    let Abstract(series) = series.to_rust(cr);
    let series = series.borrow();
    let index = index.to_rust::<Coerce<_, i64, usize>>(cr).get()?;

    series_get(cr, &data_type, &series, index)?.to_ocaml(cr)
}

enum SeriesMapError {
    SeriesGetError(String),
    FunctionCallError(BoxRoot<OCamlException>),
}

#[ocaml_interop_export(raise_on_err)]
fn rust_series_map(
    cr: &mut &mut OCamlRuntime,
    arg_type: OCamlRef<GADTDataType>,
    return_type: OCamlRef<GADTDataType>,
    series: OCamlRef<DynBox<PolarsSeries>>,
    f: OCamlRef<fn(DummyBoxRoot) -> DummyBoxRoot>,
) -> OCaml<Result<DynBox<PolarsSeries>, OCamlException>> {
    let arg_type: GADTDataType = arg_type.to_rust(cr);
    let return_type: GADTDataType = return_type.to_rust(cr);
    let Abstract(series) = series.to_rust(cr);
    let f = f.to_boxroot(cr);

    let mut return_values = Vec::new();

    // TODO: Intuitively this try_fold feels quite odd; why not just return
    // the OCaml exception if f.try_call fails? This doesn't actually do what
    // you might expect, since the rust_interop_export extracts the body of the
    // function into a code block inside a larger function, so the return
    // returns out a scope that actually expects a RawOCaml return value.
    //
    // This is quite unfortunate and I think forces users to write quite
    // non-idiomatic Rust code to work around, which I think should be fixed.
    // Instead of the current code conversion, perhaps the function annotation
    // should keep the function intact and create a wrapper that calls the
    // function with the appropriate arguments. In that world, an implementation
    // like the one in rust_series_map_idiomatic (below) would work.
    let result = (0..series.borrow().len()).try_fold((), |(), i| {
        let arg: DummyBoxRoot = series_get(cr, &arg_type, &series.borrow(), i)
            .map_err(SeriesMapError::SeriesGetError)?;

        match f.try_call(cr, &arg) {
            Ok(return_value) => {
                return_values.push(return_value.to_rust());
                Ok(())
            }
            Err(exception) => {
                let ocaml_exception = exception.root();

                Err(SeriesMapError::FunctionCallError(ocaml_exception))
            }
        }
    });

    match result {
        Err(SeriesMapError::SeriesGetError(error)) => Err(error)?,
        Err(SeriesMapError::FunctionCallError(ocaml_exception)) => {
            ocaml_interop::alloc_error(cr, &ocaml_exception)
        }
        Ok(()) => {
            let series = series_new(
                cr,
                &return_type,
                series.borrow().name(),
                return_values.into_iter(),
                true,
            )?;

            let ocaml_series = Abstract(Rc::new(RefCell::new(series))).to_ocaml(cr).root();

            ocaml_interop::alloc_ok(cr, &ocaml_series)
        }
    }
}

#[allow(dead_code)]
fn rust_series_map_idiomatic<'a>(
    cr: &'a mut &'a mut OCamlRuntime,
    arg_type: OCamlRef<'a, GADTDataType>,
    return_type: OCamlRef<'a, GADTDataType>,
    series: OCamlRef<'a, DynBox<PolarsSeries>>,
    f: OCamlRef<'a, fn(DummyBoxRoot) -> DummyBoxRoot>,
) -> Result<OCaml<'a, Result<DynBox<PolarsSeries>, OCamlException>>, String> {
    let arg_type: GADTDataType = arg_type.to_rust(cr);
    let return_type: GADTDataType = return_type.to_rust(cr);
    let Abstract(series) = series.to_rust(cr);
    let f = f.to_boxroot(cr);

    let mut return_values = Vec::new();
    for i in 0..series.borrow().len() {
        let arg: DummyBoxRoot = series_get(cr, &arg_type, &series.borrow(), i)?;

        match f.try_call(cr, &arg) {
            Ok(return_value) => return_values.push(return_value.to_rust()),
            Err(exception) => {
                let ocaml_exception = exception.root();

                return Ok(ocaml_interop::alloc_error(cr, &ocaml_exception));
            }
        }
    }

    let series = series_new(
        cr,
        &return_type,
        series.borrow().name(),
        return_values.into_iter(),
        true,
    )?;

    let ocaml_series = Abstract(Rc::new(RefCell::new(series))).to_ocaml(cr).root();

    Ok(ocaml_interop::alloc_ok(cr, &ocaml_series))
}

#[ocaml_interop_export]
fn rust_series_cast(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<PolarsSeries>>,
    dtype: OCamlRef<DataType>,
    is_strict: OCamlRef<bool>,
) -> OCaml<Result<DynBox<PolarsSeries>, String>> {
    let PolarsDataType(dtype) = dtype.to_rust(cr);
    let is_strict: bool = is_strict.to_rust(cr);

    dyn_box_result(cr, series, |series| {
        let series = series.borrow();
        if is_strict {
            series.strict_cast(&dtype)
        } else {
            series.cast(&dtype)
        }
        .map(|s| Rc::new(RefCell::new(s)))
    })
}

#[ocaml_interop_export]
fn rust_series_name(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<PolarsSeries>>,
) -> OCaml<String> {
    let Abstract(series) = series.to_rust(cr);
    let series = series.borrow();
    series.name().to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_series_rename(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<PolarsSeries>>,
    name: OCamlRef<String>,
) -> OCaml<()> {
    let Abstract(series) = series.to_rust(cr);
    let name: String = name.to_rust(cr);

    let _ = series.borrow_mut().rename(&name);

    OCaml::unit()
}

#[ocaml_interop_export]
fn rust_series_dtype(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<PolarsSeries>>,
) -> OCaml<DataType> {
    let Abstract(series) = series.to_rust(cr);
    let series = series.borrow();
    PolarsDataType(series.dtype().clone()).to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_series_to_data_frame(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<PolarsSeries>>,
) -> OCaml<DynBox<crate::data_frame::PolarsDataFrame>> {
    dyn_box(cr, series, |series| {
        let data_frame = match Rc::try_unwrap(series) {
            Ok(series) => series.into_inner().into_frame(),
            Err(series) => series.borrow().clone().into_frame(),
        };
        Rc::new(RefCell::new(data_frame))
    })
}

#[ocaml_interop_export]
fn rust_series_sort(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<PolarsSeries>>,
    descending: OCamlRef<bool>,
) -> OCaml<DynBox<PolarsSeries>> {
    let descending: bool = descending.to_rust(cr);

    dyn_box(cr, series, |series| {
        let series = series.borrow();
        Rc::new(RefCell::new(series.sort(descending)))
    })
}

#[ocaml_interop_export(raise_on_err)]
fn rust_series_head(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<PolarsSeries>>,
    length: OCamlRef<Option<OCamlInt>>,
) -> OCaml<DynBox<PolarsSeries>> {
    let length = length
        .to_rust::<Coerce<_, Option<i64>, Option<usize>>>(cr)
        .get()?;

    dyn_box(cr, series, |series| {
        let series = series.borrow();
        Rc::new(RefCell::new(series.head(length)))
    })
}

#[ocaml_interop_export(raise_on_err)]
fn rust_series_tail(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<PolarsSeries>>,
    length: OCamlRef<Option<OCamlInt>>,
) -> OCaml<DynBox<PolarsSeries>> {
    let length = length
        .to_rust::<Coerce<_, Option<i64>, Option<usize>>>(cr)
        .get()?;

    dyn_box(cr, series, |series| {
        let series = series.borrow();
        Rc::new(RefCell::new(series.tail(length)))
    })
}

#[ocaml_interop_export]
fn rust_series_estimated_size(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<PolarsSeries>>,
) -> OCaml<OCamlInt> {
    let Abstract(series) = series.to_rust(cr);
    let estimated_size = series.borrow().estimated_size() as i64;
    estimated_size.to_ocaml(cr)
}

#[ocaml_interop_export(raise_on_err)]
fn rust_series_sample_n(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<PolarsSeries>>,
    n: OCamlRef<OCamlInt>,
    with_replacement: OCamlRef<bool>,
    shuffle: OCamlRef<bool>,
    seed: OCamlRef<Option<OCamlInt>>,
) -> OCaml<Result<DynBox<PolarsSeries>, String>> {
    let n = n.to_rust::<Coerce<_, i64, usize>>(cr).get()?;
    let with_replacement: bool = with_replacement.to_rust(cr);
    let shuffle: bool = shuffle.to_rust(cr);
    let seed = seed
        .to_rust::<Coerce<_, Option<i64>, Option<u64>>>(cr)
        .get()?;

    dyn_box_result(cr, series, |series| {
        let series = series.borrow();
        series
            .sample_n(n, with_replacement, shuffle, seed)
            .map(|s| Rc::new(RefCell::new(s)))
    })
}

#[ocaml_interop_export]
fn rust_series_fill_null_with_strategy(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<PolarsSeries>>,
    strategy: OCamlRef<FillNullStrategy>,
) -> OCaml<Result<DynBox<PolarsSeries>, String>> {
    let PolarsFillNullStrategy(strategy) = strategy.to_rust(cr);

    dyn_box_result(cr, series, |series| {
        let series = series.borrow();
        series.fill_null(strategy).map(|s| Rc::new(RefCell::new(s)))
    })
}

#[ocaml_interop_export]
fn rust_series_interpolate(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<PolarsSeries>>,
    method: OCamlRef<InterpolationMethod>,
) -> OCaml<DynBox<PolarsSeries>> {
    let PolarsInterpolationMethod(method) = method.to_rust(cr);

    dyn_box(cr, series, |series| {
        let series = series.borrow();
        Rc::new(RefCell::new(interpolate(&series, method)))
    })
}

fn modify_native_series_at_chunk_index<T: NativeType>(
    series: &mut Series,
    chunk_index: usize,
    indices_and_values: &Vec<(usize, T)>,
) -> Result<(), String> {
    unsafe {
        let chunks = series.chunks_mut();
        let chunk = chunks
            .get_mut(chunk_index)
            .ok_or("modify_series_at_chunk_index: index out of range")?;

        let chunk: &mut PrimitiveArray<T> = chunk
            .as_any_mut()
            .downcast_mut()
            .ok_or("modify_chunk_at_index: unable to downcast to type T")?;
        let chunk = chunk
        .get_mut_values()
        .ok_or("modify_chunk_at_index: unable to get chunk values as mutable. Is someone else accessing the data?")?;
        for (index, value) in indices_and_values {
            chunk[*index] = *value;
        }
        Ok(())
    }
}

// CR-someday mtakeda: It's slightly sad we have this duplication here. Maybe
// this is something that can be resolved with traits associating
// [PrimitiveArray<T>] with [T]] and [BooleanArray] with [bool]?
fn modify_boolean_array_at_chunk_index(
    series: &mut Series,
    chunk_index: usize,
    indices_and_values: &Vec<(usize, bool)>,
) -> Result<(), String> {
    unsafe {
        let chunks = series.chunks_mut();
        let chunk = chunks
            .get_mut(chunk_index)
            .ok_or("modify_series_at_chunk_index: index out of range")?;

        let chunk: &mut BooleanArray = chunk
            .as_any_mut()
            .downcast_mut()
            .ok_or("modify_chunk_at_index: unable to downcast to BooleanArray")?;
        chunk.apply_values_mut(|bitmap| {
            for (index, value) in indices_and_values {
                bitmap.set(*index, *value);
            }
        });
        Ok(())
    }
}

fn set_chunk_validity_based_on_null_values<T: NativeType>(
    chunk: &mut PrimitiveArray<T>,
    indices_and_values: &Vec<(usize, Option<T>)>,
) {
    if !chunk.has_validity() {
        // Construct a bitmap that only contains true. Constructing a bitmap from
        // u8 means filling the bitmap vector with 0b11111111 = 255
        let bytes: Vec<u8> = vec![255u8; chunk.len().saturating_add(7) / 8];

        chunk.set_validity(Some(Bitmap::from_u8_vec(bytes, chunk.len())));
    }
    chunk.apply_validity(|validity: Bitmap| {
        let mut validity = match validity.into_mut() {
            // The Left case generally means that someone else is referencing
            // the same memory, which is usually a bad state to be in, but we
            // can try to work around it with a clone.
            arrow2::Either::Left(validity) => validity.clone().into_mut().right().unwrap(),
            arrow2::Either::Right(validity) => validity,
        };
        for (index, value) in indices_and_values {
            validity.set(*index, value.is_some())
        }

        // Warning: validity.into() calls Bitmap::try_new(), which is an O(chunk_length)
        // operation.
        validity.into()
    });
}

fn modify_optional_native_series_at_chunk_index<T: NativeType>(
    series: &mut Series,
    chunk_index: usize,
    indices_and_values: &Vec<(usize, Option<T>)>,
) -> Result<(), String> {
    unsafe {
        // CR-someday mtakeda: I think Series/ChunkedArrays store a null_count
        // separately, so I think we want to explicitly update that as well, if
        // I'm not mistaken. See
        // https://docs.rs/polars/latest/polars/series/struct.Series.html#method.chunks_mut
        // I'm not sure how costly the mentioned [compute_len()] is, though...
        //
        // ozeng: https://docs.pola.rs/docs/rust/dev/src/polars_core/chunked_array/ops/chunkops.rs.html#87
        // I believe this operation is O(n_chunks) btw, as arr.null_count() is O(1) --
        // each chunk also keeps track of its null count.
        //
        // also -- I just added an expect test that prints out the null counts,
        // and it seems correct? I don't think I did anything to make sure that
        // the null counts are correct
        //
        // mtakeda: Hmmm, maybe things are just fine; let's punt on this. I'll
        // try looking into this later. cr-someday-ing

        let chunks = series.chunks_mut();
        let chunk = chunks
            .get_mut(chunk_index)
            .ok_or("modify_series_at_chunk_index: index out of range")?;

        let chunk: &mut PrimitiveArray<T> = chunk
            .as_any_mut()
            .downcast_mut()
            .ok_or("modify_chunk_at_index: unable to downcast to type T")?;

        set_chunk_validity_based_on_null_values(chunk, indices_and_values);

        let chunk = chunk
        .get_mut_values()
        .ok_or("modify_chunk_at_index: unable to get chunk values as mutable. Is someone else accessing the data?")?;
        for (index, value) in indices_and_values {
            match value {
                Some(value) => chunk[*index] = *value,
                None => (),
            }
        }

        Ok(())
    }
}

pub fn modify_series_at_chunk_index(
    cr: &&mut OCamlRuntime,
    series: &mut Series,
    data_type: OCamlRef<GADTDataType>,
    chunk_index: OCamlRef<OCamlInt>,
    indices_and_values: OCamlRef<OCamlList<(OCamlInt, DummyBoxRoot)>>,
    are_values_options: bool,
) -> Result<(), String> {
    let chunk_index = chunk_index.to_rust::<Coerce<_, i64, usize>>(cr).get()?;
    let indices_and_values: Vec<(i64, DummyBoxRoot)> = indices_and_values.to_rust(cr);
    let data_type: GADTDataType = data_type.to_rust(cr);

    macro_rules! modify_series {
        ($ocaml_type:ty, $rust_type:ty) => {{
            if are_values_options {
                let indices_and_values: Vec<(usize, Option<$rust_type>)> = indices_and_values
                    .into_iter()
                    .map(|(index, v)| {
                        (
                            index as usize,
                            v.interpret::<Option<$ocaml_type>>(cr).to_rust(),
                        )
                    })
                    .collect();
                modify_optional_native_series_at_chunk_index(
                    series,
                    chunk_index,
                    &indices_and_values,
                )
            } else {
                let indices_and_values: Vec<(usize, $rust_type)> = indices_and_values
                    .into_iter()
                    .map(|(index, v)| (index as usize, v.interpret::<$ocaml_type>(cr).to_rust()))
                    .collect();
                modify_native_series_at_chunk_index(series, chunk_index, &indices_and_values)
            }
        }};
    }

    macro_rules! modify_series_with_cast {
        ($ocaml_type:ty, $rust_type:ty, $downcast_to:ty, $perform_cast:expr) => {{
            if are_values_options {
                let indices_and_values: Vec<(usize, Option<$downcast_to>)> = indices_and_values
                    .into_iter()
                    .map(|(index, v)| {
                        (
                            index as usize,
                            v.interpret::<Option<$ocaml_type>>(cr)
                                .to_rust::<Option<$rust_type>>()
                                .map($perform_cast),
                        )
                    })
                    .collect();
                modify_optional_native_series_at_chunk_index(
                    series,
                    chunk_index,
                    &indices_and_values,
                )
            } else {
                let indices_and_values: Vec<(usize, $downcast_to)> = indices_and_values
                    .into_iter()
                    .map(|(index, v)| {
                        (
                            index as usize,
                            $perform_cast(v.interpret::<$ocaml_type>(cr).to_rust::<$rust_type>()),
                        )
                    })
                    .collect();
                modify_native_series_at_chunk_index(series, chunk_index, &indices_and_values)
            }
        }};
    }

    match data_type {
        GADTDataType::Boolean => {
            if are_values_options {
                // CR-someday mtakeda: why not add support for bool options
                // series while we're at it? Is this non-trivial to do?
                //
                // mskarupke: This is surprisingly tricky. I tried for a while but I think
                // BooleanArray just doesn't have a way to modify its internal validity()
                // without making a copy. Meaning a O(1) update would turn into a O(n)
                // update. Let me know if I missed anything.
                //
                // mtakeda: That's unfortunate; let's cr-someday this and
                // revisit if we want this at some point.
                Err("Modifying bool option series in place is not supported yet!".to_string())
            } else {
                let indices_and_values: Vec<(usize, bool)> = indices_and_values
                    .into_iter()
                    .map(|(index, v)| (index as usize, v.interpret::<bool>(cr).to_rust::<bool>()))
                    .collect();
                modify_boolean_array_at_chunk_index(series, chunk_index, &indices_and_values)
            }
        }
        GADTDataType::UInt8 => {
            modify_series_with_cast!(OCamlInt, i64, u8, |v: i64| v.try_into().unwrap())
        }
        GADTDataType::UInt16 => {
            modify_series_with_cast!(OCamlInt, i64, u16, |v: i64| v.try_into().unwrap())
        }
        GADTDataType::UInt32 => {
            modify_series_with_cast!(OCamlInt, i64, u32, |v: i64| v.try_into().unwrap())
        }
        GADTDataType::UInt64 => {
            modify_series_with_cast!(OCamlInt, i64, u64, |v: i64| v.try_into().unwrap())
        }
        GADTDataType::Int8 => {
            modify_series_with_cast!(OCamlInt, i64, i8, |v: i64| v.try_into().unwrap())
        }
        GADTDataType::Int16 => {
            modify_series_with_cast!(OCamlInt, i64, i16, |v: i64| v.try_into().unwrap())
        }
        GADTDataType::Int32 => {
            modify_series_with_cast!(OCamlInt, i64, i32, |v: i64| v.try_into().unwrap())
        }
        GADTDataType::Int64 => {
            modify_series!(OCamlInt, i64)
        }
        GADTDataType::Float32 => {
            modify_series_with_cast!(OCamlFloat, f64, f32, |v| v as f32)
        }
        GADTDataType::Float64 => {
            modify_series!(OCamlFloat, f64)
        }
        GADTDataType::Utf8 => {
            Err("Modifying string series in place is not supported yet!".to_string())
        }
        GADTDataType::Binary => {
            Err("Modifying binary series in place is not supported yet!".to_string())
        }
        GADTDataType::Date => {
            Err("Modifying date series in place is not supported yet!".to_string())
        }
        GADTDataType::Datetime(PolarsTimeUnit(_time_unit), _time_zone) => {
            Err("Modifying datetime series in place is not supported yet!".to_string())
        }
        GADTDataType::Duration(PolarsTimeUnit(_time_unit)) => {
            Err("Modifying duration series in place is not supported yet!".to_string())
        }
        GADTDataType::Time => {
            Err("Modifying time series in place is not supported yet!".to_string())
        }
        GADTDataType::List(_data_type) => {
            Err("Modifying list series in place is not supported yet!".to_string())
        }
    }
}

#[ocaml_interop_export(raise_on_err)]
fn rust_series_modify_at_chunk_index(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<PolarsSeries>>,
    data_type: OCamlRef<GADTDataType>,
    chunk_index: OCamlRef<OCamlInt>,
    indices_and_values: OCamlRef<OCamlList<(OCamlInt, DummyBoxRoot)>>,
) -> OCaml<Result<DynBox<()>, String>> {
    let Abstract(series) = series.to_rust(cr);
    let mut series = series.borrow_mut();

    modify_series_at_chunk_index(
        cr,
        &mut series,
        data_type,
        chunk_index,
        indices_and_values,
        false,
    )
    .map(Abstract)
    .map_err(|err| err.to_string())
    .to_ocaml(cr)
}

#[ocaml_interop_export(raise_on_err)]
fn rust_series_modify_optional_at_chunk_index(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<PolarsSeries>>,
    data_type: OCamlRef<GADTDataType>,
    chunk_index: OCamlRef<OCamlInt>,
    indices_and_values: OCamlRef<OCamlList<(OCamlInt, DummyBoxRoot)>>,
) -> OCaml<Result<DynBox<()>, String>> {
    let Abstract(series) = series.to_rust(cr);
    let mut series = series.borrow_mut();

    modify_series_at_chunk_index(
        cr,
        &mut series,
        data_type,
        chunk_index,
        indices_and_values,
        true,
    )
    .map(Abstract)
    .map_err(|err| err.to_string())
    .to_ocaml(cr)
}

#[ocaml_interop_export(raise_on_err)]
fn rust_series_clear(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<PolarsSeries>>,
) -> OCaml<()> {
    let Abstract(series) = series.to_rust(cr);
    let mut series = series.borrow_mut();
    *series = series.clear();

    OCaml::unit()
}

#[ocaml_interop_export(raise_on_err)]
fn rust_series_compute_null_count(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<PolarsSeries>>,
) -> OCaml<OCamlInt> {
    let Abstract(series) = series.to_rust(cr);
    let series = series.borrow_mut();
    // CR-someday ozeng: consider using the function series.compute_len(),
    // which to my knowledge will re-calculate the null counts. Based on tests,
    // the null count is accurate even if we don't call compute_len(), but
    // the rust docs seem to indicate that we need to call this function to keep
    // the null counts accurate after modifying a series.

    // Dumb hack: try using append() to recalculate the null count
    // let empty = Series::new_empty(series.name(), series.dtype());
    // series.append(&empty).unwrap();
    let null_count = series.null_count() as i64;

    null_count.to_ocaml(cr)
}

macro_rules! series_op {
    ($name:ident, |$($var:ident),+| $body:expr) => {
        dyn_box_op!($name, PolarsSeries, |$($var),+| {
            $(
                let $var = $var.borrow();

                // TODO: I'm not sure why we need to explicitly deref here, but
                // without this the compiler complains for various functions
                // below...
                let $var = $var.deref();
            )+
            Rc::new(RefCell::new($body))
    });
    }
}

macro_rules! series_op_result {
    ($name:ident, |$($var:ident),+| $body:expr) => {
        dyn_box_op_result!($name, PolarsSeries, |$($var),+| {
            $(
                let $var = $var.borrow();
                let $var = $var.deref();
            )+

            $body.map(|s| Rc::new(RefCell::new(s)))
        });
    }
}

series_op_result!(rust_series_eq, |series, other| {
    series.equal(other).map(|series| series.into_series())
});
series_op_result!(rust_series_neq, |series, other| {
    series.not_equal(other).map(|series| series.into_series())
});
series_op_result!(rust_series_gt, |series, other| {
    series.gt(other).map(|series| series.into_series())
});
series_op_result!(rust_series_gt_eq, |series, other| {
    series.gt_eq(other).map(|series| series.into_series())
});
series_op_result!(rust_series_lt, |series, other| {
    series.lt(other).map(|series| series.into_series())
});
series_op_result!(rust_series_lt_eq, |series, other| {
    series.lt_eq(other).map(|series| series.into_series())
});

series_op!(rust_series_add, |series, other| series + other);
series_op!(rust_series_sub, |series, other| series - other);
series_op!(rust_series_mul, |series, other| series * other);
series_op!(rust_series_div, |series, other| series / other);

#[ocaml_interop_export]
fn rust_series_to_string_hum(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<PolarsSeries>>,
) -> OCaml<String> {
    let Abstract(series) = series.to_rust(cr);
    let series = series.borrow();
    ToString::to_string(&series).to_ocaml(cr)
}
