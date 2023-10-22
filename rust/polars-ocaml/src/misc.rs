use crate::utils::PolarsDataType;
use crate::utils::*;
use chrono::naive::{NaiveDate, NaiveDateTime};
use chrono::{Datelike, Duration};
use ocaml_interop::{DynBox, OCaml, OCamlInt, OCamlList, OCamlRef, ToOCaml};
use polars::prelude::*;
use polars_ocaml_macros::ocaml_interop_export;

#[ocaml_interop_export(raise_on_err)]
fn rust_naive_date(
    cr: &mut &mut OCamlRuntime,
    year: OCamlRef<OCamlInt>,
    month: OCamlRef<OCamlInt>,
    day: OCamlRef<OCamlInt>,
) -> OCaml<Option<DynBox<NaiveDate>>> {
    let year: i32 = year.to_rust(cr);
    let month = month.to_rust::<Coerce<_, i32, u32>>(cr).get()?;
    let day = day.to_rust::<Coerce<_, i32, u32>>(cr).get()?;

    NaiveDate::from_ymd_opt(year, month, day)
        .map(Abstract)
        .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_naive_date_to_ocaml(
    cr: &mut &mut OCamlRuntime,
    date: OCamlRef<DynBox<NaiveDate>>,
) -> OCaml<(OCamlInt, OCamlInt, OCamlInt)> {
    let Abstract(date) = date.to_rust(cr);

    let year = date.year() as i64;
    let month = date.month() as i64;
    let day = date.day() as i64;

    (year, month, day).to_ocaml(cr)
}

#[ocaml_interop_export(raise_on_err)]
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
        .get()?
        .unwrap_or(0);
    let min: u32 = min
        .to_rust::<Coerce<_, Option<i64>, Option<u32>>>(cr)
        .get()?
        .unwrap_or(0);
    let sec: u32 = sec
        .to_rust::<Coerce<_, Option<i64>, Option<u32>>>(cr)
        .get()?
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
fn rust_time_ns_to_naive_datetime(
    cr: &mut &mut OCamlRuntime,
    time_ns: OCamlRef<OCamlInt63>,
) -> OCaml<Option<DynBox<NaiveDateTime>>> {
    let OCamlInt63(ns_since_epoch) = time_ns.to_rust(cr);

    // We use Euclidean division here instead of the usual div (/) and mod (%)
    // operations since we need the remainder to be non-negative.
    NaiveDateTime::from_timestamp_opt(
        ns_since_epoch.div_euclid(1_000_000_000),
        ns_since_epoch.rem_euclid(1_000_000_000) as u32,
    )
    .map(Abstract)
    .to_ocaml(cr)
}

#[ocaml_interop_export(raise_on_err)]
fn rust_naive_datetime_to_timestamp_nanos(
    cr: &mut &mut OCamlRuntime,
    datetime: OCamlRef<DynBox<NaiveDateTime>>,
) -> OCaml<OCamlInt> {
    let Abstract(datetime) = datetime.to_rust(cr);

    datetime
        .timestamp_nanos_opt()
        .ok_or_else(|| format!("out of range datetime: {:?}", datetime))?
        .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_naive_datetime_round_to_time_unit(
    cr: &mut &mut OCamlRuntime,
    datetime: OCamlRef<DynBox<NaiveDateTime>>,
    time_unit: OCamlRef<TimeUnit>,
) -> OCaml<DynBox<NaiveDateTime>> {
    let Abstract(datetime) = datetime.to_rust(cr);
    let PolarsTimeUnit(time_unit) = time_unit.to_rust(cr);

    let datetime = match time_unit {
        TimeUnit::Nanoseconds => datetime,
        TimeUnit::Microseconds => {
            arrow2::temporal_conversions::timestamp_us_to_datetime(datetime.timestamp_micros())
        }
        TimeUnit::Milliseconds => {
            arrow2::temporal_conversions::timestamp_ms_to_datetime(datetime.timestamp_millis())
        }
    };

    Abstract(datetime).to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_time_ns_span_to_duration(
    cr: &mut &mut OCamlRuntime,
    time_ns_span: OCamlRef<OCamlInt63>,
) -> OCaml<DynBox<Duration>> {
    let OCamlInt63(time_ns_span) = time_ns_span.to_rust(cr);

    let duration = chrono::Duration::nanoseconds(time_ns_span);

    Abstract(duration).to_ocaml(cr)
}

#[ocaml_interop_export(raise_on_err)]
fn rust_duration_to_nanoseconds(
    cr: &mut &mut OCamlRuntime,
    duration: OCamlRef<DynBox<Duration>>,
) -> OCaml<OCamlInt> {
    let Abstract(duration) = duration.to_rust(cr);

    duration
        .num_nanoseconds()
        .ok_or_else(|| format!("out of range duration: {:?}", duration))?
        .to_ocaml(cr)
}

#[ocaml_interop_export(raise_on_err)]
fn rust_duration_round_to_time_unit(
    cr: &mut &mut OCamlRuntime,
    duration: OCamlRef<DynBox<Duration>>,
    time_unit: OCamlRef<TimeUnit>,
) -> OCaml<DynBox<Duration>> {
    let Abstract(duration) = duration.to_rust(cr);
    let PolarsTimeUnit(time_unit) = time_unit.to_rust(cr);

    let duration = match time_unit {
        TimeUnit::Nanoseconds => duration,
        TimeUnit::Microseconds => arrow2::temporal_conversions::duration_us_to_duration(
            duration
                .num_microseconds()
                .ok_or_else(|| format!("out of range duration: {:?}", duration))?,
        ),
        TimeUnit::Milliseconds => {
            arrow2::temporal_conversions::duration_ms_to_duration(duration.num_milliseconds())
        }
    };

    Abstract(duration).to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_all_tzs(
    cr: &mut &mut OCamlRuntime,
    unit: OCamlRef<()>,
) -> OCaml<OCamlList<DynBox<chrono_tz::Tz>>> {
    let () = unit.to_rust(cr);

    let tzs: Vec<Abstract<chrono_tz::Tz>> =
        chrono_tz::TZ_VARIANTS.into_iter().map(Abstract).collect();

    tzs.to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_tz_to_string(
    cr: &mut &mut OCamlRuntime,
    tz: OCamlRef<DynBox<chrono_tz::Tz>>,
) -> OCaml<String> {
    let Abstract(tz) = tz.to_rust(cr);

    tz.name().to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_tz_parse(
    cr: &mut &mut OCamlRuntime,
    tz_str: OCamlRef<String>,
) -> OCaml<Option<DynBox<chrono_tz::Tz>>> {
    let tz_str: String = tz_str.to_rust(cr);

    tz_str
        .parse::<chrono_tz::Tz>()
        .ok()
        .map(Abstract)
        .to_ocaml(cr)
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

    // We use a meaningless if branch here to get rid of the unreachable
    // expression warning.
    if true {
        panic!("test panic: {}", error_message);
    }

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
