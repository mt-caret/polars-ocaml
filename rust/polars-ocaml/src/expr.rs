use crate::interop::*;
use crate::polars_types::PolarsDataType;
use crate::polars_types::*;
use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use ocaml_interop::{
    DynBox, OCaml, OCamlBytes, OCamlFloat, OCamlInt, OCamlList, OCamlRef, OCamlRuntime, ToOCaml,
};
use polars::prelude::*;
use polars::series::IsSorted;
use polars_ocaml_macros::ocaml_interop_export;
use std::rc::Rc;

macro_rules! expr_op {
    ($name:ident, |$($var:ident),+| $body:expr) => {
        dyn_box_op!($name, Expr, |$($var),+| $body);
    }
}

fn expr_series_map<'a>(
    cr: &'a mut &'a mut OCamlRuntime,
    expr: OCamlRef<'a, DynBox<Expr>>,
    f: impl Fn(Series) -> Result<Option<Series>, PolarsError>
        + std::marker::Sync
        + std::marker::Send
        + 'static,
    output_type: GetOutput,
) -> OCaml<'a, DynBox<Expr>> {
    let Abstract(expr) = expr.to_rust(cr);
    OCaml::box_value(cr, expr.map(f, output_type))
}

#[ocaml_interop_export]
fn rust_expr_col(cr: &mut &mut OCamlRuntime, name: OCamlRef<String>) -> OCaml<DynBox<Expr>> {
    let name: String = name.to_rust(cr);
    OCaml::box_value(cr, col(&name))
}

#[ocaml_interop_export]
fn rust_expr_cols(
    cr: &mut &mut OCamlRuntime,
    names: OCamlRef<OCamlList<String>>,
) -> OCaml<DynBox<Expr>> {
    let names: Vec<String> = names.to_rust(cr);
    OCaml::box_value(cr, cols(names))
}

#[ocaml_interop_export]
fn rust_expr_all(cr: &mut &mut OCamlRuntime, unit: OCamlRef<()>) -> OCaml<DynBox<Expr>> {
    let _: () = unit.to_rust(cr);
    OCaml::box_value(cr, all())
}

#[ocaml_interop_export]
fn rust_expr_exclude(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    names: OCamlRef<OCamlList<String>>,
) -> OCaml<DynBox<Expr>> {
    let names: Vec<String> = names.to_rust(cr);
    dyn_box(cr, expr, |expr| expr.exclude(names))
}

#[ocaml_interop_export]
fn rust_expr_null(cr: &mut &mut OCamlRuntime, unit: OCamlRef<()>) -> OCaml<DynBox<Expr>> {
    let (): () = unit.to_rust(cr);
    OCaml::box_value(cr, lit(NULL))
}

#[ocaml_interop_export(raise_on_err)]
fn rust_expr_lit(
    cr: &mut &mut OCamlRuntime,
    data_type: OCamlRef<GADTDataType>,
    value: OCamlRef<DummyBoxRoot>,
) -> OCaml<DynBox<Expr>> {
    let data_type: GADTDataType = data_type.to_rust(cr);
    let value: DummyBoxRoot = value.to_rust(cr);

    fn expr_lit_int<T>(cr: &OCamlRuntime, value: DummyBoxRoot) -> Result<Expr, String>
    where
        T: TryFrom<i64> + polars::prelude::Literal,
        T::Error: std::fmt::Display,
    {
        let value = value.interpret::<OCamlInt>(cr).to_rust::<i64>();
        let value = TryInto::<T>::try_into(value).map_err(|err| err.to_string())?;
        Ok(lit(value))
    }

    let lit = match data_type {
        GADTDataType::Boolean => lit(value.interpret::<bool>(cr).to_rust::<bool>()),
        GADTDataType::UInt8 => expr_lit_int::<u8>(cr, value)?,
        GADTDataType::UInt16 => expr_lit_int::<u16>(cr, value)?,
        GADTDataType::UInt32 => expr_lit_int::<u32>(cr, value)?,
        GADTDataType::UInt64 => expr_lit_int::<u64>(cr, value)?,
        GADTDataType::Int8 => expr_lit_int::<i8>(cr, value)?,
        GADTDataType::Int16 => expr_lit_int::<i16>(cr, value)?,
        GADTDataType::Int32 => expr_lit_int::<i32>(cr, value)?,
        GADTDataType::Int64 => expr_lit_int::<i64>(cr, value)?,
        GADTDataType::Float32 => lit(value.interpret::<OCamlFloat>(cr).to_rust::<f64>() as f32),
        GADTDataType::Float64 => lit(value.interpret::<OCamlFloat>(cr).to_rust::<f64>()),
        GADTDataType::Utf8 => lit(value.interpret::<String>(cr).to_rust::<String>()),
        GADTDataType::Binary => lit(value.interpret::<OCamlBytes>(cr).to_rust::<Vec<u8>>()),
        GADTDataType::Date => {
            // This cast is a hack to work around the bug (demonstrated in lib.rs)
            // where passing a date directly to lit() results in the dtype of the
            // literal getting converted into a datetime[ns].
            let date = value
                .interpret::<DynBox<NaiveDate>>(cr)
                .to_rust::<Abstract<NaiveDate>>()
                .get();
            lit(Series::new("literal", [date]))
        }
        GADTDataType::Datetime(PolarsTimeUnit(time_unit), time_zone) => {
            let datetime = value
                .interpret::<DynBox<NaiveDateTime>>(cr)
                .to_rust::<Abstract<NaiveDateTime>>()
                .get();

            let timestamp = match time_unit {
                TimeUnit::Nanoseconds => datetime
                    .timestamp_nanos_opt()
                    .ok_or_else(|| format!("out of range datetime: {:?}", datetime))?,
                TimeUnit::Microseconds => datetime.timestamp_micros(),
                TimeUnit::Milliseconds => datetime.timestamp_millis(),
            };

            let time_zone = time_zone.map(|time_zone| time_zone.get().name().to_string());

            lit(LiteralValue::DateTime(timestamp, time_unit, time_zone))
        }
        GADTDataType::Duration(PolarsTimeUnit(time_unit)) => {
            let duration = value
                .interpret::<DynBox<Duration>>(cr)
                .to_rust::<Abstract<Duration>>()
                .get();

            let duration = match time_unit {
                TimeUnit::Nanoseconds => duration
                    .num_nanoseconds()
                    .ok_or_else(|| format!("out of range duration: {:?}", duration))?,
                TimeUnit::Microseconds => duration
                    .num_microseconds()
                    .ok_or_else(|| format!("out of range duration: {:?}", duration))?,

                TimeUnit::Milliseconds => duration.num_milliseconds(),
            };

            lit(LiteralValue::Duration(duration, time_unit))
        }
        GADTDataType::Time => {
            let time = value
                .interpret::<DynBox<NaiveTime>>(cr)
                .to_rust::<Abstract<NaiveTime>>()
                .get();

            lit(LiteralValue::Time(crate::time::time_to_time64ns(&time)))
        }
        GADTDataType::List(data_type) => {
            // Since there is no direct way to create a List-based literal, we
            // create a one-element series instead, and use that.
            let series = crate::series::series_new(
                cr,
                &GADTDataType::List(data_type),
                "series",
                vec![value].into_iter(),
                false,
            )?;
            lit(series)
        }
    };

    OCaml::box_value(cr, lit)
}

#[ocaml_interop_export]
fn rust_expr_naive_date(
    cr: &mut &mut OCamlRuntime,
    value: OCamlRef<DynBox<NaiveDate>>,
) -> OCaml<DynBox<Expr>> {
    dyn_box(cr, value, lit)
}

#[ocaml_interop_export]
fn rust_expr_naive_datetime(
    cr: &mut &mut OCamlRuntime,
    value: OCamlRef<DynBox<NaiveDateTime>>,
) -> OCaml<DynBox<Expr>> {
    dyn_box(cr, value, lit)
}

#[ocaml_interop_export]
fn rust_expr_series(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<DynBox<crate::series::PolarsSeries>>,
) -> OCaml<DynBox<Expr>> {
    dyn_box(cr, series, |series| match Rc::try_unwrap(series) {
        Ok(series) => lit(series.into_inner()),
        Err(series) => lit(series.borrow().clone()),
    })
}

#[ocaml_interop_export]
fn rust_expr_cast(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    data_type: OCamlRef<DataType>,
    is_strict: OCamlRef<bool>,
) -> OCaml<DynBox<Expr>> {
    let PolarsDataType(data_type): PolarsDataType = data_type.to_rust(cr);
    let is_strict: bool = is_strict.to_rust(cr);
    dyn_box(cr, expr, |expr| {
        if is_strict {
            expr.strict_cast(data_type)
        } else {
            expr.cast(data_type)
        }
    })
}

#[ocaml_interop_export]
fn rust_expr_sort(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    descending: OCamlRef<bool>,
) -> OCaml<DynBox<Expr>> {
    let descending: bool = descending.to_rust(cr);
    dyn_box(cr, expr, |expr| expr.sort(descending))
}

#[ocaml_interop_export]
fn rust_expr_sort_by(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    descending: OCamlRef<bool>,
    by: OCamlRef<OCamlList<DynBox<Expr>>>,
) -> OCaml<DynBox<Expr>> {
    let by = unwrap_abstract_vec(by.to_rust(cr));
    let descending: bool = descending.to_rust(cr);
    let descending = vec![descending; by.len()];
    dyn_box(cr, expr, |expr| expr.sort_by(by, descending))
}

#[ocaml_interop_export]
fn rust_expr_set_sorted_flag(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    is_sorted: OCamlRef<IsSorted>,
) -> OCaml<DynBox<Expr>> {
    let PolarsIsSorted(is_sorted) = is_sorted.to_rust(cr);
    dyn_box(cr, expr, |expr| expr.set_sorted_flag(is_sorted))
}

expr_op!(rust_expr_first, |expr| expr.first());
expr_op!(rust_expr_last, |expr| expr.last());
expr_op!(rust_expr_reverse, |expr| expr.reverse());

// TODO: the following functions are ~roughly the same between Expr, Series,
// and DataFrame; it would be nice if we could reduce the boilerplace around
// this:
// - head
// - tail
// - sample_n

#[ocaml_interop_export(raise_on_err)]
fn rust_expr_head(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    length: OCamlRef<Option<OCamlInt>>,
) -> OCaml<DynBox<Expr>> {
    let length = length
        .to_rust::<Coerce<_, Option<i64>, Option<usize>>>(cr)
        .get()?;

    dyn_box(cr, expr, |expr| expr.head(length))
}

#[ocaml_interop_export(raise_on_err)]
fn rust_expr_tail(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    length: OCamlRef<Option<OCamlInt>>,
) -> OCaml<DynBox<Expr>> {
    let length = length
        .to_rust::<Coerce<_, Option<i64>, Option<usize>>>(cr)
        .get()?;

    dyn_box(cr, expr, |expr| expr.tail(length))
}

expr_op!(rust_expr_take, |expr, idx| expr.take(idx));

#[ocaml_interop_export(raise_on_err)]
fn rust_expr_sample_n(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    n: OCamlRef<OCamlInt>,
    with_replacement: OCamlRef<bool>,
    shuffle: OCamlRef<bool>,
    seed: OCamlRef<Option<OCamlInt>>,
    fixed_seed: OCamlRef<bool>,
) -> OCaml<DynBox<Expr>> {
    let n = n.to_rust::<Coerce<_, i64, usize>>(cr).get()?;
    let with_replacement: bool = with_replacement.to_rust(cr);
    let shuffle: bool = shuffle.to_rust(cr);
    let seed = seed
        .to_rust::<Coerce<_, Option<i64>, Option<u64>>>(cr)
        .get()?;
    let fixed_seed = fixed_seed.to_rust(cr);

    dyn_box(cr, expr, |expr| {
        expr.sample_n(n, with_replacement, shuffle, seed, fixed_seed)
    })
}

expr_op!(rust_expr_filter, |expr, predicate| expr.filter(predicate));
expr_op!(rust_expr_is_in, |expr, other| expr.is_in(other));
expr_op!(rust_expr_ceil, |expr| expr.ceil());
expr_op!(rust_expr_floor, |expr| expr.floor());

#[ocaml_interop_export]
fn rust_expr_clip_min_float(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    min: OCamlRef<OCamlFloat>,
) -> OCaml<DynBox<Expr>> {
    let min: f64 = min.to_rust(cr);
    dyn_box(cr, expr, |expr| expr.clip_min(AnyValue::Float64(min)))
}

#[ocaml_interop_export]
fn rust_expr_clip_max_float(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    max: OCamlRef<OCamlFloat>,
) -> OCaml<DynBox<Expr>> {
    let max: f64 = max.to_rust(cr);
    dyn_box(cr, expr, |expr| expr.clip_max(AnyValue::Float64(max)))
}

#[ocaml_interop_export]
fn rust_expr_clip_min_int(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    min: OCamlRef<OCamlInt>,
) -> OCaml<DynBox<Expr>> {
    let min: i64 = min.to_rust(cr);
    dyn_box(cr, expr, |expr| expr.clip_min(AnyValue::Int64(min)))
}

#[ocaml_interop_export]
fn rust_expr_clip_max_int(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    max: OCamlRef<OCamlInt>,
) -> OCaml<DynBox<Expr>> {
    let max: i64 = max.to_rust(cr);
    dyn_box(cr, expr, |expr| expr.clip_max(AnyValue::Int64(max)))
}

expr_op!(rust_expr_pow, |base, exponent| base.pow(exponent));
expr_op!(rust_expr_sum, |expr| expr.sum());
expr_op!(rust_expr_mean, |expr| expr.mean());
expr_op!(rust_expr_median, |expr| expr.median());
expr_op!(rust_expr_quantile, |expr, quantile_num, quantile_interpol_option| expr.quantile(quantile_num, quantile_interpol_option));
expr_op!(rust_expr_mode, |expr| expr.mode());
expr_op!(rust_expr_max, |expr| expr.max());
expr_op!(rust_expr_min, |expr| expr.min());
expr_op!(rust_expr_arg_max, |expr| expr.arg_max());
expr_op!(rust_expr_arg_min, |expr| expr.arg_min());
expr_op!(rust_expr_count, |expr| expr.count());

#[ocaml_interop_export]
fn rust_expr_count_(cr: &mut &mut OCamlRuntime, unit: OCamlRef<()>) -> OCaml<DynBox<Expr>> {
    let () = unit.to_rust(cr);
    OCaml::box_value(cr, count())
}

expr_op!(rust_expr_n_unique, |expr| expr.n_unique());
expr_op!(rust_expr_approx_n_unique, |expr| expr.approx_n_unique());
expr_op!(rust_expr_explode, |expr| expr.explode());

#[ocaml_interop_export]
fn rust_expr_over(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    partition_by: OCamlRef<OCamlList<DynBox<Expr>>>,
    mapping_strategy: OCamlRef<WindowMapping>,
) -> OCaml<DynBox<Expr>> {
    let PolarsWindowMapping(mapping_strategy) = mapping_strategy.to_rust(cr);
    let partition_by: Vec<_> = unwrap_abstract_vec(partition_by.to_rust(cr));
    dyn_box(cr, expr, |expr| {
        expr.over_with_options(
            &partition_by,
            WindowOptions {
                mapping: mapping_strategy,
            },
        )
    })
}

#[ocaml_interop_export]
fn rust_expr_concat_list(
    cr: &mut &mut OCamlRuntime,
    exprs: OCamlRef<OCamlList<DynBox<Expr>>>,
) -> OCaml<Result<DynBox<Expr>, String>> {
    let exprs: Vec<_> = unwrap_abstract_vec(exprs.to_rust(cr));
    concat_list(exprs)
        .map(Abstract)
        .map_err(|e| e.to_string())
        .to_ocaml(cr)
}

expr_op!(rust_expr_null_count, |expr| expr.null_count());
expr_op!(rust_expr_is_null, |expr| expr.is_null());
expr_op!(rust_expr_is_not_null, |expr| expr.is_not_null());
expr_op!(rust_expr_is_nan, |expr| expr.is_nan());
expr_op!(rust_expr_is_finite, |expr| expr.is_finite());
expr_op!(rust_expr_is_infinite, |expr| expr.is_infinite());
expr_op!(rust_expr_is_not_nan, |expr| expr.is_not_nan());
expr_op!(rust_expr_fill_null, |expr, with| expr.fill_null(with));
expr_op!(rust_expr_fill_nan, |expr, with| expr.fill_nan(with));

#[ocaml_interop_export]
fn rust_expr_quantile(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    quantile_expr: OCamlRef<DynBox<Expr>>,
    interpol_option: OCamlRef<QuantileInterpolOptions>,
) -> OCaml<DynBox<Expr>> {
    let PolarsQuantileInterpolOptions(interpol_option): PolarsQuantileInterpolOptions =
        interpol_option.to_rust(cr);

    dyn_box2(cr, expr, quantile_expr, |expr, quantile_expr| {
        expr.quantile(quantile_expr, interpol_option)
    })
}

#[ocaml_interop_export]
fn rust_expr_fill_null_with_strategy(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    strategy: OCamlRef<FillNullStrategy>,
) -> OCaml<DynBox<Expr>> {
    let PolarsFillNullStrategy(strategy) = strategy.to_rust(cr);
    expr_series_map(
        cr,
        expr,
        move |series| Ok(Some(series.fill_null(strategy)?)),
        GetOutput::default(),
    )
}

#[ocaml_interop_export]
fn rust_expr_interpolate(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    method: OCamlRef<InterpolationMethod>,
) -> OCaml<DynBox<Expr>> {
    let PolarsInterpolationMethod(method) = method.to_rust(cr);
    dyn_box(cr, expr, |expr| expr.interpolate(method))
}

#[ocaml_interop_export(raise_on_err)]
fn rust_expr_rank(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    method: OCamlRef<RankMethod>,
    descending: OCamlRef<bool>,
    seed: OCamlRef<Option<OCamlInt>>,
) -> OCaml<DynBox<Expr>> {
    let PolarsRankMethod(method) = method.to_rust(cr);
    let descending: bool = descending.to_rust(cr);
    let seed = seed
        .to_rust::<Coerce<_, Option<i64>, Option<u64>>>(cr)
        .get()?;
    dyn_box(cr, expr, |expr| {
        expr.rank(RankOptions { method, descending }, seed)
    })
}

// Diagram of when/then/otherwise chaining:
// ┌────┐           ┌────┐                ┌───────────┐
// │When├─.then()──►│Then├────.when()────►│ChainedWhen│◄──┐
// └────┘           └─┬──┘                └─────┬─────┘   │
//   ▲                │                         │         │
//   │                │                         │         │
//   │           .otherwise()                .then()   .when()
//   │                │                         │         │
//   │                ▼                         ▼         │
// when()           ┌────┐                ┌───────────┐   │
//                  │Expr│◄──.otherwise()─┤ChainedThen├───┘
//                  └────┘                └───────────┘
enum WhenThenClause {
    Empty,
    Then(Then),
    ChainedThen(ChainedThen),
}

#[ocaml_interop_export]
fn rust_expr_when_then(
    cr: &mut &mut OCamlRuntime,
    when_then_clauses: OCamlRef<OCamlList<(DynBox<Expr>, DynBox<Expr>)>>,
    otherwise: OCamlRef<DynBox<Expr>>,
) -> OCaml<DynBox<Expr>> {
    let when_then_clauses: Vec<(Abstract<Expr>, Abstract<Expr>)> = when_then_clauses.to_rust(cr);
    let when_then_clauses: Vec<(Expr, Expr)> = when_then_clauses
        .into_iter()
        .map(|(Abstract(when), Abstract(then))| (when, then))
        .collect();
    dyn_box(cr, otherwise, |otherwise| {
        let mut ret = WhenThenClause::Empty;

        for (when_expr, then_expr) in when_then_clauses {
            match ret {
                WhenThenClause::Empty => {
                    ret = WhenThenClause::Then(when(when_expr).then(then_expr))
                }
                WhenThenClause::Then(then) => {
                    ret = WhenThenClause::ChainedThen(then.when(when_expr).then(then_expr))
                }
                WhenThenClause::ChainedThen(chained_then) => {
                    ret = WhenThenClause::ChainedThen(chained_then.when(when_expr).then(then_expr))
                }
            }
        }

        match ret {
            WhenThenClause::Empty => otherwise,
            WhenThenClause::Then(then) => then.otherwise(otherwise),
            WhenThenClause::ChainedThen(chained_then) => chained_then.otherwise(otherwise),
        }
    })
}

#[ocaml_interop_export]
fn rust_expr_shift(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    periods: OCamlRef<OCamlInt>,
    fill_value: OCamlRef<Option<DynBox<Expr>>>,
) -> OCaml<DynBox<Expr>> {
    let periods: i64 = periods.to_rust(cr);
    let fill_value: Option<Abstract<Expr>> = fill_value.to_rust(cr);
    dyn_box(cr, expr, |expr| match fill_value {
        None => expr.shift(periods),
        Some(Abstract(fill_value)) => expr.shift_and_fill(periods, fill_value),
    })
}

#[ocaml_interop_export]
fn rust_expr_cum_count(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    reverse: OCamlRef<bool>,
) -> OCaml<DynBox<Expr>> {
    let reverse: bool = reverse.to_rust(cr);
    dyn_box(cr, expr, |expr| expr.cumcount(reverse))
}

#[ocaml_interop_export]
fn rust_expr_cum_sum(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    reverse: OCamlRef<bool>,
) -> OCaml<DynBox<Expr>> {
    let reverse: bool = reverse.to_rust(cr);
    dyn_box(cr, expr, |expr| expr.cumsum(reverse))
}

#[ocaml_interop_export]
fn rust_expr_cum_prod(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    reverse: OCamlRef<bool>,
) -> OCaml<DynBox<Expr>> {
    let reverse: bool = reverse.to_rust(cr);
    dyn_box(cr, expr, |expr| expr.cumprod(reverse))
}

#[ocaml_interop_export]
fn rust_expr_cum_min(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    reverse: OCamlRef<bool>,
) -> OCaml<DynBox<Expr>> {
    let reverse: bool = reverse.to_rust(cr);
    dyn_box(cr, expr, |expr| expr.cummin(reverse))
}

#[ocaml_interop_export]
fn rust_expr_cum_max(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    reverse: OCamlRef<bool>,
) -> OCaml<DynBox<Expr>> {
    let reverse: bool = reverse.to_rust(cr);
    dyn_box(cr, expr, |expr| expr.cummax(reverse))
}

#[ocaml_interop_export]
fn rust_expr_alias(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    name: OCamlRef<String>,
) -> OCaml<DynBox<Expr>> {
    let name: String = name.to_rust(cr);
    dyn_box(cr, expr, |expr| expr.alias(&name))
}

#[ocaml_interop_export]
fn rust_expr_prefix(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    prefix: OCamlRef<String>,
) -> OCaml<DynBox<Expr>> {
    let prefix: String = prefix.to_rust(cr);
    dyn_box(cr, expr, |expr| expr.prefix(&prefix))
}

#[ocaml_interop_export]
fn rust_expr_suffix(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    suffix: OCamlRef<String>,
) -> OCaml<DynBox<Expr>> {
    let suffix: String = suffix.to_rust(cr);
    dyn_box(cr, expr, |expr| expr.suffix(&suffix))
}

#[ocaml_interop_export(raise_on_err)]
fn rust_expr_round(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    decimals: OCamlRef<OCamlInt>,
) -> OCaml<DynBox<Expr>> {
    let decimals = decimals.to_rust::<Coerce<_, i64, u32>>(cr).get()?;

    dyn_box(cr, expr, |expr| expr.round(decimals))
}

expr_op!(rust_expr_eq, |expr, other| expr.eq(other));
expr_op!(rust_expr_neq, |expr, other| expr.neq(other));
expr_op!(rust_expr_gt, |expr, other| expr.gt(other));
expr_op!(rust_expr_gt_eq, |expr, other| expr.gt_eq(other));
expr_op!(rust_expr_lt, |expr, other| expr.lt(other));
expr_op!(rust_expr_lt_eq, |expr, other| expr.lt_eq(other));
expr_op!(rust_expr_not, |expr| expr.not());
expr_op!(rust_expr_and, |expr, other| expr.and(other));
expr_op!(rust_expr_or, |expr, other| expr.or(other));
expr_op!(rust_expr_xor, |expr, other| expr.xor(other));
expr_op!(rust_expr_add, |expr, other| expr + other);
expr_op!(rust_expr_sub, |expr, other| expr - other);
expr_op!(rust_expr_mul, |expr, other| expr * other);
expr_op!(rust_expr_div, |expr, other| expr / other);
expr_op!(rust_expr_floor_div, |expr, other| expr.floor_div(other));
expr_op!(rust_expr_abs, |expr| expr.abs());
expr_op!(rust_expr_exp, |expr| expr.exp());
expr_op!(rust_expr_log1p, |expr| expr.log1p());

#[ocaml_interop_export]
fn rust_expr_log(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    base: OCamlRef<OCamlFloat>,
) -> OCaml<DynBox<Expr>> {
    let base: f64 = base.to_rust(cr);
    dyn_box(cr, expr, |expr| expr.log(base))
}

#[ocaml_interop_export]
fn rust_expr_dt_strftime(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    format: OCamlRef<String>,
) -> OCaml<DynBox<Expr>> {
    let format: String = format.to_rust(cr);
    dyn_box(cr, expr, |expr| expr.dt().to_string(&format))
}

#[ocaml_interop_export]
fn rust_expr_dt_convert_time_zone(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    timezone: OCamlRef<String>,
) -> OCaml<DynBox<Expr>> {
    let timezone: String = timezone.to_rust(cr);
    dyn_box(cr, expr, |expr| expr.dt().convert_time_zone(timezone))
}

#[ocaml_interop_export]
fn rust_expr_dt_replace_time_zone(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    timezone: OCamlRef<Option<String>>,
    use_earliest: OCamlRef<Option<bool>>,
) -> OCaml<DynBox<Expr>> {
    let timezone: Option<String> = timezone.to_rust(cr);
    let use_earliest: Option<bool> = use_earliest.to_rust(cr);
    dyn_box(cr, expr, |expr| {
        expr.dt().replace_time_zone(timezone, use_earliest)
    })
}

expr_op!(rust_expr_dt_year, |expr| expr.dt().year());
expr_op!(rust_expr_dt_month, |expr| expr.dt().month());
expr_op!(rust_expr_dt_day, |expr| expr.dt().day());

#[ocaml_interop_export]
fn rust_expr_dt_days(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
) -> OCaml<DynBox<Expr>> {
    expr_series_map(
        cr,
        expr,
        |series| Ok(Some(series.duration()?.days().into_series())),
        GetOutput::from_type(DataType::Int64),
    )
}

#[ocaml_interop_export]
fn rust_expr_dt_hours(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
) -> OCaml<DynBox<Expr>> {
    expr_series_map(
        cr,
        expr,
        |series| Ok(Some(series.duration()?.hours().into_series())),
        GetOutput::from_type(DataType::Int64),
    )
}

#[ocaml_interop_export]
fn rust_expr_dt_minutes(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
) -> OCaml<DynBox<Expr>> {
    expr_series_map(
        cr,
        expr,
        |series| Ok(Some(series.duration()?.minutes().into_series())),
        GetOutput::from_type(DataType::Int64),
    )
}

#[ocaml_interop_export]
fn rust_expr_dt_seconds(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
) -> OCaml<DynBox<Expr>> {
    expr_series_map(
        cr,
        expr,
        |series| Ok(Some(series.duration()?.seconds().into_series())),
        GetOutput::from_type(DataType::Int64),
    )
}

#[ocaml_interop_export]
fn rust_expr_dt_milliseconds(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
) -> OCaml<DynBox<Expr>> {
    expr_series_map(
        cr,
        expr,
        |series| Ok(Some(series.duration()?.milliseconds().into_series())),
        GetOutput::from_type(DataType::Int64),
    )
}

#[ocaml_interop_export]
fn rust_expr_dt_microseconds(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
) -> OCaml<DynBox<Expr>> {
    expr_series_map(
        cr,
        expr,
        |series| Ok(Some(series.duration()?.microseconds().into_series())),
        GetOutput::from_type(DataType::Int64),
    )
}

#[ocaml_interop_export]
fn rust_expr_dt_nanoseconds(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
) -> OCaml<DynBox<Expr>> {
    expr_series_map(
        cr,
        expr,
        |series| Ok(Some(series.duration()?.nanoseconds().into_series())),
        GetOutput::from_type(DataType::Int64),
    )
}

expr_op!(rust_expr_dt_date, |expr| expr.dt().date());
expr_op!(rust_expr_dt_time, |expr| expr.dt().time());

#[ocaml_interop_export]
fn rust_expr_str_split(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    by: OCamlRef<String>,
    inclusive: OCamlRef<bool>,
) -> OCaml<DynBox<Expr>> {
    let by: String = by.to_rust(cr);
    let inclusive = inclusive.to_rust(cr);
    dyn_box(cr, expr, |expr| {
        if inclusive {
            expr.str().split_inclusive(&by)
        } else {
            expr.str().split(&by)
        }
    })
}

#[ocaml_interop_export]
fn rust_expr_str_strptime(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    data_type: OCamlRef<DataType>,
    format: OCamlRef<String>,
) -> OCaml<DynBox<Expr>> {
    let PolarsDataType(data_type): PolarsDataType = data_type.to_rust(cr);
    let format: String = format.to_rust(cr);

    dyn_box(cr, expr, |expr| {
        // TODO: make other options configurable
        let options = StrptimeOptions {
            format: Some(format),
            strict: true,
            exact: true,
            cache: false,
            use_earliest: None,
        };
        expr.str().strptime(data_type, options)
    })
}

#[ocaml_interop_export]
fn rust_expr_str_lengths(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
) -> OCaml<DynBox<Expr>> {
    expr_series_map(
        cr,
        expr,
        |series| Ok(Some(series.utf8()?.str_lengths().into_series())),
        GetOutput::from_type(DataType::UInt32),
    )
}

#[ocaml_interop_export]
fn rust_expr_str_n_chars(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
) -> OCaml<DynBox<Expr>> {
    expr_series_map(
        cr,
        expr,
        |series| Ok(Some(series.utf8()?.str_n_chars().into_series())),
        GetOutput::from_type(DataType::UInt32),
    )
}

#[ocaml_interop_export]
fn rust_expr_str_contains(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    pat: OCamlRef<String>,
    literal: OCamlRef<bool>,
) -> OCaml<DynBox<Expr>> {
    let pat: String = pat.to_rust(cr);
    let literal: bool = literal.to_rust(cr);
    expr_series_map(
        cr,
        expr,
        move |series| {
            let chunked = series.utf8()?;
            let contains = if literal {
                chunked.contains_literal(&pat)
            } else {
                chunked.contains(&pat, true)
            };
            Ok(Some(contains?.into_series()))
        },
        GetOutput::from_type(DataType::Boolean),
    )
}

#[ocaml_interop_export]
fn rust_expr_str_starts_with(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    prefix: OCamlRef<String>,
) -> OCaml<DynBox<Expr>> {
    let prefix: String = prefix.to_rust(cr);
    expr_series_map(
        cr,
        expr,
        move |series| Ok(Some(series.utf8()?.starts_with(&prefix).into_series())),
        GetOutput::from_type(DataType::Boolean),
    )
}

#[ocaml_interop_export]
fn rust_expr_str_ends_with(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    suffix: OCamlRef<String>,
) -> OCaml<DynBox<Expr>> {
    let suffix: String = suffix.to_rust(cr);
    expr_series_map(
        cr,
        expr,
        move |series| Ok(Some(series.utf8()?.ends_with(&suffix).into_series())),
        GetOutput::from_type(DataType::Boolean),
    )
}

#[ocaml_interop_export(raise_on_err)]
fn rust_expr_str_extract(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    pat: OCamlRef<String>,
    group_index: OCamlRef<OCamlInt>,
) -> OCaml<DynBox<Expr>> {
    let pat: String = pat.to_rust(cr);
    let group_index = group_index.to_rust::<Coerce<_, i64, usize>>(cr).get()?;

    dyn_box(cr, expr, |expr| {
        let f = move |series: Series| {
            Ok(Some(
                series.utf8()?.extract(&pat, group_index)?.into_series(),
            ))
        };
        expr.map(f, GetOutput::from_type(DataType::Utf8))
    })
}

#[ocaml_interop_export]
fn rust_expr_str_extract_all(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    pat: OCamlRef<String>,
) -> OCaml<DynBox<Expr>> {
    let pat: String = pat.to_rust(cr);
    expr_series_map(
        cr,
        expr,
        move |series| Ok(Some(series.utf8()?.extract_all(&pat)?.into_series())),
        GetOutput::from_type(DataType::List(Box::new(DataType::Utf8))),
    )
}

#[ocaml_interop_export]
fn rust_expr_str_replace(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    pat: OCamlRef<String>,
    with: OCamlRef<String>,
    literal: OCamlRef<bool>,
) -> OCaml<DynBox<Expr>> {
    let pat: String = pat.to_rust(cr);
    let with: String = with.to_rust(cr);
    let literal: bool = literal.to_rust(cr);
    expr_series_map(
        cr,
        expr,
        move |series| {
            let chunked = series.utf8()?;
            let replaced = if literal {
                chunked.replace_literal(&pat, &with, 1)
            } else {
                chunked.replace(&pat, &with)
            };
            Ok(Some(replaced?.into_series()))
        },
        GetOutput::from_type(DataType::List(Box::new(DataType::Utf8))),
    )
}

#[ocaml_interop_export]
fn rust_expr_str_replace_all(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    pat: OCamlRef<String>,
    with: OCamlRef<String>,
    literal: OCamlRef<bool>,
) -> OCaml<DynBox<Expr>> {
    let pat: String = pat.to_rust(cr);
    let with: String = with.to_rust(cr);
    let literal: bool = literal.to_rust(cr);
    expr_series_map(
        cr,
        expr,
        move |series| {
            let chunked = series.utf8()?;
            let replaced = if literal {
                chunked.replace_literal_all(&pat, &with)
            } else {
                chunked.replace_all(&pat, &with)
            };
            Ok(Some(replaced?.into_series()))
        },
        GetOutput::from_type(DataType::List(Box::new(DataType::Utf8))),
    )
}

#[ocaml_interop_export]
fn rust_expr_str_strip(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    matches: OCamlRef<Option<String>>,
) -> OCaml<DynBox<Expr>> {
    let matches: Option<String> = matches.to_rust(cr);
    dyn_box(cr, expr, |expr| expr.str().strip(matches))
}

#[ocaml_interop_export]
fn rust_expr_str_lstrip(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    matches: OCamlRef<Option<String>>,
) -> OCaml<DynBox<Expr>> {
    let matches: Option<String> = matches.to_rust(cr);
    dyn_box(cr, expr, |expr| expr.str().lstrip(matches))
}

#[ocaml_interop_export]
fn rust_expr_str_rstrip(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    matches: OCamlRef<Option<String>>,
) -> OCaml<DynBox<Expr>> {
    let matches: Option<String> = matches.to_rust(cr);
    dyn_box(cr, expr, |expr| expr.str().rstrip(matches))
}

expr_op!(rust_expr_str_to_lowercase, |expr| expr.str().to_lowercase());
expr_op!(rust_expr_str_to_uppercase, |expr| expr.str().to_uppercase());

#[ocaml_interop_export(raise_on_err)]
fn rust_expr_str_slice(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    start: OCamlRef<OCamlInt>,
    length: OCamlRef<Option<OCamlInt>>,
) -> OCaml<DynBox<Expr>> {
    let start: i64 = start.to_rust(cr);
    let length = length
        .to_rust::<Coerce<_, Option<i64>, Option<u64>>>(cr)
        .get()?;
    dyn_box(cr, expr, |expr| expr.str().str_slice(start, length))
}

expr_op!(rust_expr_list_lengths, |expr| expr.list().lengths());
expr_op!(rust_expr_list_slice, |expr, offset, length| {
    expr.list().slice(offset, length)
});
expr_op!(rust_expr_list_head, |expr, n| expr.list().head(n));
expr_op!(rust_expr_list_tail, |expr, n| expr.list().tail(n));
expr_op!(rust_expr_list_sum, |expr| expr.list().sum());

#[ocaml_interop_export]
fn rust_expr_list_eval(
    cr: &mut &mut OCamlRuntime,
    expr: OCamlRef<DynBox<Expr>>,
    other: OCamlRef<DynBox<Expr>>,
    parallel: OCamlRef<bool>,
) -> OCaml<DynBox<Expr>> {
    let parallel = parallel.to_rust(cr);
    dyn_box2(cr, expr, other, |expr, other| {
        expr.list().eval(other, parallel)
    })
}
