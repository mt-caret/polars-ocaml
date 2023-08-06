use crate::utils::PolarsDataType;
use crate::utils::*;
use chrono::naive::{NaiveDate, NaiveDateTime};
use ocaml_interop::{DynBox, OCaml, OCamlInt, OCamlList, OCamlRef, ToOCaml};
use polars::prelude::*;
use polars_ocaml_macros::ocaml_interop_export;

#[ocaml_interop_export]
fn rust_naive_date(
    cr: &mut &mut OCamlRuntime,
    year: OCamlRef<OCamlInt>,
    month: OCamlRef<OCamlInt>,
    day: OCamlRef<OCamlInt>,
) -> OCaml<Option<DynBox<NaiveDate>>> {
    let year: i32 = year.to_rust(cr);
    let month = month.to_rust::<Coerce<_, i32, u32>>(cr).get();
    let day = day.to_rust::<Coerce<_, i32, u32>>(cr).get();

    NaiveDate::from_ymd_opt(year, month, day)
        .map(Abstract)
        .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_naive_date_to_naive_datetime(
    cr: &mut &mut OCamlRuntime,
    date: OCamlRef<DynBox<NaiveDate>>,
    hour: OCamlRef<Option<OCamlInt>>,
    min: OCamlRef<Option<OCamlInt>>,
    sec: OCamlRef<Option<OCamlInt>>,
) -> OCaml<Option<DynBox<NaiveDateTime>>> {
    let Abstract(date) = date.to_rust(cr);

    let hour: u32 = hour
        .to_rust::<Coerce<_, Option<i64>, Option<u32>>>(cr)
        .get()
        .unwrap_or(0);
    let min: u32 = min
        .to_rust::<Coerce<_, Option<i64>, Option<u32>>>(cr)
        .get()
        .unwrap_or(0);
    let sec: u32 = sec
        .to_rust::<Coerce<_, Option<i64>, Option<u32>>>(cr)
        .get()
        .unwrap_or(0);

    date.and_hms_opt(hour, min, sec).map(Abstract).to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_naive_datetime_to_string(
    cr: &mut &mut OCamlRuntime,
    datetime: OCamlRef<DynBox<NaiveDateTime>>,
) -> OCaml<String> {
    let Abstract(datetime) = datetime.to_rust(cr);

    datetime.to_string().to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_schema_create(
    cr: &mut &mut OCamlRuntime,
    fields: OCamlRef<OCamlList<(String, DataType)>>,
) -> OCaml<DynBox<Schema>> {
    let fields: Vec<(String, PolarsDataType)> = fields.to_rust(cr);
    let schema: Schema = fields
        .into_iter()
        .map(|(name, PolarsDataType(data_type))| Field::new(&name, data_type))
        .collect();
    OCaml::box_value(cr, schema)
}

#[ocaml_interop_export]
fn rust_schema_to_fields(
    cr: &mut &mut OCamlRuntime,
    schema: OCamlRef<DynBox<Schema>>,
) -> OCaml<OCamlList<(String, DataType)>> {
    let Abstract(schema) = schema.to_rust(cr);
    let fields: Vec<(String, PolarsDataType)> = schema
        .iter_fields()
        .map(|Field { name, dtype }| (name.to_string(), PolarsDataType(dtype)))
        .collect();
    fields.to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_test_panic(cr: &mut &mut OCamlRuntime, error_message: OCamlRef<String>) -> OCaml<()> {
    let error_message: String = error_message.to_rust(cr);

    // see rust_test_exception
    if true {
        panic!("{}", error_message);
    }
    OCaml::unit()
}

#[ocaml_interop_export]
fn rust_test_exception(cr: &mut &mut OCamlRuntime, error_message: OCamlRef<String>) -> OCaml<()> {
    let error_message: String = error_message.to_rust(cr);

    // ideally we would put #[allow(unused_unsafe)] here, but that causes
    // VSCode to get confused and warn on all code in the ocaml_export! macro.
    if true {
        unsafe { ocaml_failwith(&error_message) }
    }
    OCaml::unit()
}

#[ocaml_interop_export]
fn rust_install_panic_hook(
    cr: &mut &mut OCamlRuntime,
    suppress_backtrace: OCamlRef<bool>,
) -> OCaml<()> {
    let suppress_backtrace = suppress_backtrace.to_rust(cr);
    std::panic::set_hook(Box::new(move |panic_info| {
        let payload = panic_info.payload();
        let message = if let Some(s) = payload.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.to_string()
        } else {
            "Box<Any>".to_string()
        };

        let message = if suppress_backtrace {
            format!("Rust panic: {}", message)
        } else {
            let backtrace = std::backtrace::Backtrace::force_capture();
            format!(
                "Rust panic: {}\nbacktrace:\n{}",
                message,
                backtrace.to_string()
            )
        };
        let message = std::ffi::CString::new(message).expect("CString::new failed");

        unsafe {
            ocaml_sys::caml_failwith(message.as_ptr());
        }
    }));
    OCaml::unit()
}

#[ocaml_interop_export]
fn rust_test_fill_null_strategy(
    cr: &mut &mut OCamlRuntime,
    fill_null_strategy: OCamlRef<FillNullStrategy>,
) -> OCaml<FillNullStrategy> {
    let PolarsFillNullStrategy(fill_null_strategy) = fill_null_strategy.to_rust(cr);

    PolarsFillNullStrategy(fill_null_strategy).to_ocaml(cr)
}
