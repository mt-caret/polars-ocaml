use crate::utils::*;
use chrono::naive::{NaiveDate, NaiveDateTime};
use ocaml_interop::{
    impl_to_ocaml_variant, BoxRoot, DynBox, FromOCaml, OCaml, OCamlBytes, OCamlFloat, OCamlInt,
    OCamlInt32, OCamlList, OCamlRef, OCamlRuntime, ToOCaml,
};
use polars::prelude::prelude::*;
use polars::prelude::*;
use polars_ocaml_macros::ocaml_interop_export;
use std::fmt::Debug;

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

pub struct DummyBoxRoot(BoxRoot<DummyBoxRoot>);

unsafe impl FromOCaml<DummyBoxRoot> for DummyBoxRoot {
    fn from_ocaml(v: OCaml<DummyBoxRoot>) -> Self {
        DummyBoxRoot(v.root())
    }
}

unsafe impl ToOCaml<DummyBoxRoot> for DummyBoxRoot {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, DummyBoxRoot> {
        self.0.get(cr)
    }
}

impl DummyBoxRoot {
    unsafe fn new<'a, T>(boxroot: BoxRoot<T>) -> Self {
        let boxroot: BoxRoot<DummyBoxRoot> = std::mem::transmute(boxroot);

        DummyBoxRoot(boxroot)
    }

    fn interpret<'a, T>(&self, cr: &'a OCamlRuntime) -> OCaml<'a, T> {
        let ocaml_value: OCaml<DummyBoxRoot> = self.0.get(cr);

        unsafe { OCaml::new(cr, ocaml_value.raw()) }
    }
}

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
                                // TODO: explain why this needs to be false
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
                            // TODO: explain why this needs to be false
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

pub struct OCamlIntable<T>(T);

unsafe impl<T> ToOCaml<OCamlInt> for OCamlIntable<T>
where
    T: TryInto<i64> + Copy,
    <T as TryInto<i64>>::Error: Debug,
{
    fn to_ocaml<'a>(&self, _cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlInt> {
        OCaml::of_i64(self.0.try_into().expect("Couldn't convert to i64"))
            .expect("Number couldn't fit in OCaml integer")
    }
}

fn series_to_boxrooted_ocaml_list<'a>(
    cr: &mut &'a mut OCamlRuntime,
    data_type: &GADTDataType,
    series: Series,
    allow_nulls: bool,
) -> Result<DummyBoxRoot, String> {
    macro_rules! ocaml_intable {
        ($rust_type:ty, $ca:expr) => {{
            let ca = $ca.map_err(|err| err.to_string())?;

            if allow_nulls {
                let vec: Vec<Option<OCamlIntable<$rust_type>>> =
                    ca.into_iter().map(|n| n.map(OCamlIntable)).collect();

                let return_value: BoxRoot<OCamlList<Option<OCamlInt>>> = vec.to_ocaml(cr).root();

                Ok(unsafe { DummyBoxRoot::new(return_value) })
            } else {
                if ca.null_count() > 0 {
                    return Err("Series contains null values".to_string());
                }

                let vec: Vec<OCamlIntable<$rust_type>> = {
                    let mut buf = Vec::with_capacity(ca.len());

                    for arr in ca.downcast_iter() {
                        buf.extend(arr.values_iter().map(|n| OCamlIntable(*n)))
                    }
                    buf
                };

                let return_value: BoxRoot<OCamlList<OCamlInt>> = vec.to_ocaml(cr).root();

                Ok(unsafe { DummyBoxRoot::new(return_value) })
            }
        }};
    }

    match data_type {
        GADTDataType::Boolean => {
            let ca = series.bool().map_err(|err| err.to_string())?;

            if allow_nulls {
                let vec: Vec<Option<bool>> = ca.into_iter().collect();

                let return_value: BoxRoot<OCamlList<Option<bool>>> = vec.to_ocaml(cr).root();

                Ok(unsafe { DummyBoxRoot::new(return_value) })
            } else {
                if ca.null_count() > 0 {
                    return Err("Series contains null values".to_string());
                }

                let vec: Vec<bool> = {
                    let mut buf = Vec::with_capacity(ca.len());

                    for arr in ca.downcast_iter() {
                        buf.extend(arr.values_iter())
                    }
                    buf
                };

                let return_value: BoxRoot<OCamlList<bool>> = vec.to_ocaml(cr).root();

                Ok(unsafe { DummyBoxRoot::new(return_value) })
            }
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

            if allow_nulls {
                let vec: Vec<Option<f64>> = ca
                    .into_iter()
                    .map(|f32o| f32o.map(|f32| f32 as f64))
                    .collect();

                let return_value: BoxRoot<OCamlList<Option<OCamlFloat>>> = vec.to_ocaml(cr).root();

                Ok(unsafe { DummyBoxRoot::new(return_value) })
            } else {
                if ca.null_count() > 0 {
                    return Err("Series contains null values".to_string());
                }

                let vec: Vec<f64> = {
                    let mut buf = Vec::with_capacity(ca.len());

                    for arr in ca.downcast_iter() {
                        buf.extend(arr.values_iter().map(|f32| *f32 as f64))
                    }
                    buf
                };

                let return_value: BoxRoot<OCamlList<OCamlFloat>> = vec.to_ocaml(cr).root();

                Ok(unsafe { DummyBoxRoot::new(return_value) })
            }
        }
        GADTDataType::Float64 => {
            let ca = series.f64().map_err(|err| err.to_string())?;

            if allow_nulls {
                let vec: Vec<Option<f64>> = ca.into_iter().collect();

                let return_value: BoxRoot<OCamlList<Option<OCamlFloat>>> = vec.to_ocaml(cr).root();

                Ok(unsafe { DummyBoxRoot::new(return_value) })
            } else {
                if ca.null_count() > 0 {
                    return Err("Series contains null values".to_string());
                }

                let vec: Vec<f64> = {
                    let mut buf = Vec::with_capacity(ca.len());

                    for arr in ca.downcast_iter() {
                        buf.extend(arr.values_iter())
                    }
                    buf
                };

                let return_value: BoxRoot<OCamlList<OCamlFloat>> = vec.to_ocaml(cr).root();

                Ok(unsafe { DummyBoxRoot::new(return_value) })
            }
        }
        GADTDataType::Utf8 => {
            let ca = series.utf8().map_err(|err| err.to_string())?;

            if allow_nulls {
                let vec: Vec<Option<String>> = ca
                    .into_iter()
                    .map(|utf8o| utf8o.map(|utf8| utf8.to_string()))
                    .collect();

                let return_value: BoxRoot<OCamlList<Option<String>>> = vec.to_ocaml(cr).root();

                Ok(unsafe { DummyBoxRoot::new(return_value) })
            } else {
                if ca.null_count() > 0 {
                    return Err("Series contains null values".to_string());
                }

                let vec: Vec<String> = {
                    let mut buf = Vec::with_capacity(ca.len());

                    for arr in ca.downcast_iter() {
                        buf.extend(arr.values_iter().map(|str| str.to_string()))
                    }
                    buf
                };

                let return_value: BoxRoot<OCamlList<String>> = vec.to_ocaml(cr).root();

                Ok(unsafe { DummyBoxRoot::new(return_value) })
            }
        }
        GADTDataType::Binary => {
            let ca = series.binary().map_err(|err| err.to_string())?;

            if allow_nulls {
                let vec: Vec<Option<Vec<u8>>> = ca
                    .into_iter()
                    .map(|binaryo| binaryo.map(|binary| binary.to_vec()))
                    .collect();

                let return_value: BoxRoot<OCamlList<Option<String>>> = vec.to_ocaml(cr).root();

                Ok(unsafe { DummyBoxRoot::new(return_value) })
            } else {
                if ca.null_count() > 0 {
                    return Err("Series contains null values".to_string());
                }

                let vec: Vec<&[u8]> = {
                    let mut buf = Vec::with_capacity(ca.len());

                    for arr in ca.downcast_iter() {
                        buf.extend(arr.values_iter())
                    }
                    buf
                };

                let return_value: BoxRoot<OCamlList<OCamlBytes>> = vec.to_ocaml(cr).root();

                Ok(unsafe { DummyBoxRoot::new(return_value) })
            }
        }
        GADTDataType::List(data_type) => {
            let ca = series.list().map_err(|err| err.to_string())?;

            if allow_nulls {
                let vec: Vec<Option<DummyBoxRoot>> = ca
                    .into_iter()
                    .map(|serieso| match serieso {
                        None => Ok(None),
                        Some(series) => {
                            series_to_boxrooted_ocaml_list(cr, data_type, series, false).map(Some)
                        }
                    })
                    .collect::<Result<_, _>>()?;

                let return_value: BoxRoot<OCamlList<Option<DummyBoxRoot>>> =
                    vec.to_ocaml(cr).root();

                Ok(unsafe { DummyBoxRoot::new(return_value) })
            } else {
                if ca.null_count() > 0 {
                    return Err("Series contains null values".to_string());
                }

                let vec: Vec<DummyBoxRoot> = {
                    let mut buf = Vec::with_capacity(ca.len());

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
                        buf.push(series_to_boxrooted_ocaml_list(
                            cr, data_type, series, false,
                        )?)
                    }

                    buf
                };

                let return_value: BoxRoot<OCamlList<DummyBoxRoot>> = vec.to_ocaml(cr).root();

                Ok(unsafe { DummyBoxRoot::new(return_value) })
            }
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
