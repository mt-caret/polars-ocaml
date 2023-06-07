#![feature(try_blocks)]
use chrono::naive::{NaiveDate, NaiveDateTime};
use ocaml_interop::{
    ocaml_alloc_tagged_block, ocaml_alloc_variant, ocaml_export, ocaml_unpack_variant, DynBox,
    FromOCaml, OCaml, OCamlBytes, OCamlFloat, OCamlInt, OCamlList, OCamlRef, OCamlRuntime, ToOCaml,
};
use polars::prelude::prelude::*;
use polars::prelude::*;
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

unsafe impl ToOCaml<TimeUnit> for PolarsTimeUnit {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, TimeUnit> {
        let PolarsTimeUnit(timeunit) = self;
        ocaml_alloc_variant! {
            cr, timeunit => {
                TimeUnit::Nanoseconds,
                TimeUnit::Microseconds,
                TimeUnit::Milliseconds,
            }
        }
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
                DataType::Datetime(timeunit: TimeUnit, timezone: Option<String>) => {
                    let PolarsTimeUnit(timeunit) = timeunit;
                    DataType::Datetime(timeunit, timezone)},
                DataType::Duration(timeunit: TimeUnit) => {
                    let PolarsTimeUnit(timeunit) = timeunit;
                    DataType::Duration(timeunit)},
                DataType::Time,
                DataType::List(datatype: DataType) => {
                    let PolarsDataType(datatype) = datatype;
                    DataType::List(Box::new(datatype))
                },
                DataType::Null,
                DataType::Unknown,
            }
        };
        PolarsDataType(result.expect("Failure when unpacking an OCaml<DataType> variant into PolarsDataType (unexpected tag value"))
    }
}

unsafe fn ocaml_value<'a, T>(cr: &'a mut OCamlRuntime, n: i32) -> OCaml<'a, T> {
    unsafe { OCaml::new(cr, OCaml::of_i32(n).raw()) }
}

unsafe impl ToOCaml<DataType> for PolarsDataType {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, DataType> {
        let PolarsDataType(datatype) = self;
        // We expand out the macro here since we need to do some massaging of the
        // values to get things into the right shape to convert to OCaml values
        unsafe {
            match datatype {
                DataType::Boolean => ocaml_value(cr, 0),
                DataType::UInt8 => ocaml_value(cr, 1),
                DataType::UInt16 => ocaml_value(cr, 2),
                DataType::UInt32 => ocaml_value(cr, 3),
                DataType::UInt64 => ocaml_value(cr, 4),
                DataType::Int8 => ocaml_value(cr, 5),
                DataType::Int16 => ocaml_value(cr, 6),
                DataType::Int32 => ocaml_value(cr, 7),
                DataType::Int64 => ocaml_value(cr, 8),
                DataType::Float32 => ocaml_value(cr, 9),
                DataType::Float64 => ocaml_value(cr, 10),
                DataType::Utf8 => ocaml_value(cr, 11),
                DataType::Binary => ocaml_value(cr, 12),
                DataType::Date => ocaml_value(cr, 13),
                DataType::Datetime(timeunit, timezone) => {
                    let timeunit = PolarsTimeUnit(*timeunit);
                    let timezone = timezone.clone();
                    ocaml_alloc_tagged_block!(cr, 0, timeunit : TimeUnit, timezone: Option<String>)
                }
                DataType::Duration(timeunit) => {
                    let timeunit = PolarsTimeUnit(*timeunit);
                    ocaml_alloc_tagged_block!(cr, 1,  timeunit: TimeUnit)
                }
                DataType::Time => ocaml_value(cr, 14),
                DataType::List(datatype) => {
                    let datatype = PolarsDataType(*datatype.clone());
                    ocaml_alloc_tagged_block!(cr, 2,  datatype: DataType)
                }
                DataType::Null => ocaml_value(cr, 15),
                DataType::Unknown => ocaml_value(cr, 16),
            }
        }
    }
}

struct Abstract<T>(T);
unsafe impl<T: 'static + Clone> FromOCaml<DynBox<T>> for Abstract<T> {
    fn from_ocaml(v: OCaml<DynBox<T>>) -> Self {
        Abstract(Borrow::<T>::borrow(&v).clone())
    }
}

unsafe impl<T: 'static + Clone> ToOCaml<DynBox<T>> for Abstract<T> {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, DynBox<T>> {
        // TODO: I don't fully understand why ToOCaml takes a &self, since that
        // prevents us from using box_value without a clone() call.
        OCaml::box_value(cr, self.0.clone())
    }
}

fn unwrap_abstract_vec<T>(v: Vec<Abstract<T>>) -> Vec<T> {
    v.into_iter().map(|Abstract(v)| v).collect()
}

fn expr_unary_op<'a>(
    cr: &'a mut &'a mut OCamlRuntime,
    expr: OCamlRef<'a, DynBox<Expr>>,
    f: impl Fn(Expr) -> Expr,
) -> OCaml<'a, DynBox<Expr>> {
    let Abstract(expr) = expr.to_rust(cr);
    OCaml::box_value(cr, f(expr))
}

fn expr_binary_op<'a>(
    cr: &'a mut &'a mut OCamlRuntime,
    expr: OCamlRef<'a, DynBox<Expr>>,
    other: OCamlRef<'a, DynBox<Expr>>,
    f: impl Fn(Expr, Expr) -> Expr,
) -> OCaml<'a, DynBox<Expr>> {
    let Abstract(expr) = expr.to_rust(cr);
    let Abstract(other) = other.to_rust(cr);
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
        let month: i32 = month.to_rust(cr);
        let day: i32 = day.to_rust(cr);

        let result: Option<_> = try {
            Abstract(NaiveDate::from_ymd_opt(year, month.try_into().ok()?, day.try_into().ok()?)?)
        };
        result.to_ocaml(cr)
    }

    fn rust_naive_date_to_naive_datetime(cr, date: OCamlRef<DynBox<NaiveDate>>) -> OCaml<Option<DynBox<NaiveDateTime>>> {
        let Abstract(date) = date.to_rust(cr);
        date.and_hms_opt(0, 0, 0).map(Abstract).to_ocaml(cr)
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

    fn rust_expr_first(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.first())
    }

    fn rust_expr_last(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.last())
    }

    fn rust_expr_reverse(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.reverse())
    }

    // TODO: the following functions are ~roughly the same between Expr, Series,
    // and DataFrame; it would be nice if we could reduce the boilerplace around
    // this:
    // - head
    // - tail
    // - sample_n

    fn rust_expr_head(cr, expr: OCamlRef<DynBox<Expr>>, length: OCamlRef<Option<OCamlInt>>) -> OCaml<Option<DynBox<Expr>>> {
        let Abstract(expr) = expr.to_rust(cr);
        let length: Option<i64> = length.to_rust(cr);

        match length.map(|length| length.try_into().ok()) {
            None => Some(Abstract(expr.head(None))),
            Some(None) => None,
            Some(Some(length)) => Some(Abstract(expr.head(Some(length)))),
        }.to_ocaml(cr)
    }

    fn rust_expr_tail(cr, expr: OCamlRef<DynBox<Expr>>, length: OCamlRef<Option<OCamlInt>>) -> OCaml<Option<DynBox<Expr>>> {
        let Abstract(expr) = expr.to_rust(cr);
        let length: Option<i64> = length.to_rust(cr);

        match length.map(|length| length.try_into().ok()) {
            None => Some(Abstract(expr.tail(None))),
            Some(None) => None,
            Some(Some(length)) => Some(Abstract(expr.tail(Some(length)))),
        }.to_ocaml(cr)
    }

    fn rust_expr_sample_n(cr, expr: OCamlRef<DynBox<Expr>>, n: OCamlRef<OCamlInt>, with_replacement: OCamlRef<bool>, shuffle: OCamlRef<bool>, seed: OCamlRef<Option<OCamlInt>>) -> OCaml<Option<DynBox<Expr>>> {
        let result: Option<_> = try {
            let Abstract(expr) = expr.to_rust(cr);
            let n: usize = n.to_rust::<i64>(cr).try_into().ok()?;
            let with_replacement: bool = with_replacement.to_rust(cr);
            let shuffle: bool = shuffle.to_rust(cr);
            let seed: Option<Result<u64,_>> = seed.to_rust::<Option<i64>>(cr).map(|seed| seed.try_into());
            let seed: Option<u64> = seed.map_or(Ok(None), |seed| seed.map(Some)).ok()?;

            Abstract(expr.sample_n(n, with_replacement, shuffle, seed))
        };
        result.to_ocaml(cr)
    }

    fn rust_expr_filter(cr, expr: OCamlRef<DynBox<Expr>>, predicate: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, predicate, |expr, predicate| expr.filter(predicate))
    }

    fn rust_expr_sum(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.sum())
    }

    fn rust_expr_mean(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.mean())
    }

    fn rust_expr_count(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.count())
    }

    fn rust_expr_n_unique(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.n_unique())
    }

    fn rust_expr_approx_unique(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.approx_unique())
    }

    fn rust_expr_is_null(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.is_null())
    }

    fn rust_expr_is_not_null(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.is_not_null())
    }

    fn rust_expr_when_then(cr, when_then_clauses: OCamlRef<OCamlList<(DynBox<Expr>, DynBox<Expr>)>>, otherwise: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        let when_then_clauses: Vec<(Abstract<Expr>, Abstract<Expr>)> = when_then_clauses.to_rust(cr);
        let when_then_clauses: Vec<(Expr,Expr)> =
            when_then_clauses.into_iter().map(|(Abstract(when),Abstract(then))| (when,then)).collect();
        let Abstract(otherwise) = otherwise.to_rust(cr);

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

    fn rust_series_to_string_hum(cr, series: OCamlRef<DynBox<Series>>) -> OCaml<String> {
        let Abstract(series) = series.to_rust(cr);
        ToString::to_string(&series).to_ocaml(cr)
    }

    fn rust_data_frame_new(cr, series: OCamlRef<OCamlList<DynBox<Series>>>) -> OCaml<Result<DynBox<DataFrame>,String>> {
        let series: Vec<Series> = unwrap_abstract_vec(series.to_rust(cr));

        DataFrame::new(series).map(Abstract).map_err(|err| err.to_string()).to_ocaml(cr)
    }

   fn rust_data_frame_describe(cr, data_frame: OCamlRef<DynBox<DataFrame>>, percentiles: OCamlRef<Option<OCamlList<OCamlFloat>>>) -> OCaml<Result<DynBox<DataFrame>,String>> {
        let Abstract(data_frame) = data_frame.to_rust(cr);
        let percentiles: Option<Vec<f64>> = percentiles.to_rust(cr);

        // TODO: I'm not sure why I can't do this with something like
        // .map(|percentiles| percentiles.as_slice()
        match percentiles {
            None => data_frame.describe(None),
            Some(percentiles) => data_frame.describe(Some(percentiles.as_slice()))
        }
        .map(Abstract).map_err(|err| err.to_string()).to_ocaml(cr)
    }

   fn rust_data_frame_lazy(cr, data_frame: OCamlRef<DynBox<DataFrame>>) -> OCaml<DynBox<LazyFrame>> {
        let Abstract(data_frame) = data_frame.to_rust(cr);
        OCaml::box_value(cr, data_frame.lazy())
    }

    fn rust_data_frame_head(cr, data_frame: OCamlRef<DynBox<DataFrame>>, length: OCamlRef<Option<OCamlInt>>) -> OCaml<Option<DynBox<DataFrame>>> {
        let Abstract(data_frame) = data_frame.to_rust(cr);
        let length: Option<i64> = length.to_rust(cr);

        match length.map(|length| length.try_into().ok()) {
            None => Some(Abstract(data_frame.head(None))),
            Some(None) => None,
            Some(Some(length)) => Some(Abstract(data_frame.head(Some(length)))),
        }.to_ocaml(cr)
    }

    fn rust_data_frame_tail(cr, data_frame: OCamlRef<DynBox<DataFrame>>, length: OCamlRef<Option<OCamlInt>>) -> OCaml<Option<DynBox<DataFrame>>> {
        let Abstract(data_frame) = data_frame.to_rust(cr);
        let length: Option<i64> = length.to_rust(cr);

        match length.map(|length| length.try_into().ok()) {
            None => Some(Abstract(data_frame.tail(None))),
            Some(None) => None,
            Some(Some(length)) => Some(Abstract(data_frame.tail(Some(length)))),
        }.to_ocaml(cr)
    }

    fn rust_data_frame_sample_n(cr, data_frame: OCamlRef<DynBox<DataFrame>>, n: OCamlRef<OCamlInt>, with_replacement: OCamlRef<bool>, shuffle: OCamlRef<bool>, seed: OCamlRef<Option<OCamlInt>>) -> OCaml<Option<Result<DynBox<DataFrame>,String>>> {
        let result: Option<_> = try {
            let Abstract(data_frame) = data_frame.to_rust(cr);
            let n: usize = n.to_rust::<i64>(cr).try_into().ok()?;
            let with_replacement: bool = with_replacement.to_rust(cr);
            let shuffle: bool = shuffle.to_rust(cr);
            let seed: Option<Result<u64,_>> = seed.to_rust::<Option<i64>>(cr).map(|seed| seed.try_into());
            let seed: Option<u64> = seed.map_or(Ok(None), |seed| seed.map(Some)).ok()?;

            data_frame.sample_n(n, with_replacement, shuffle, seed)
            .map(Abstract).map_err(|err| err.to_string())
        };
        result.to_ocaml(cr)
    }

    fn rust_data_frame_schema(cr, data_frame: OCamlRef<DynBox<DataFrame>>) -> OCaml<DynBox<Schema>> {
        let Abstract(data_frame) = data_frame.to_rust(cr);
        OCaml::box_value(cr, data_frame.schema())
    }

    fn rust_data_frame_to_string_hum(cr, data_frame: OCamlRef<DynBox<DataFrame>>) -> OCaml<String> {
        let Abstract(data_frame) = data_frame.to_rust(cr);
        data_frame.to_string().to_ocaml(cr)
    }

    // TODO: properly return error type instead of a string
    fn rust_lazy_frame_scan_parquet(cr, path: OCamlRef<OCamlBytes>) -> OCaml<Result<DynBox<LazyFrame>, String>>{
        let path:String = path.to_rust(cr);
        let path:&Path = Path::new(&path);

        LazyFrame::scan_parquet(path, Default::default())
        .map(Abstract).map_err(|err| err.to_string())
        .to_ocaml(cr)
    }

    fn rust_lazy_frame_to_dot(cr, lazy_frame: OCamlRef<DynBox<LazyFrame>>) -> OCaml<Result<String,String>>{
        let Abstract(lazy_frame) = lazy_frame.to_rust(cr);

        // TODO: make configurable
        lazy_frame.to_dot(false).map_err(|err| err.to_string()).to_ocaml(cr)
    }

    fn rust_lazy_frame_collect(cr, lazy_frame: OCamlRef<DynBox<LazyFrame>>)-> OCaml<Result<DynBox<DataFrame>, String>> {
        let Abstract(lazy_frame) = lazy_frame.to_rust(cr);
        lazy_frame.collect()
        .map(Abstract).map_err(|err| err.to_string())
        .to_ocaml(cr)
    }

    fn rust_lazy_frame_filter(cr, lazy_frame: OCamlRef<DynBox<LazyFrame>>, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<LazyFrame>> {
        let Abstract(lazy_frame) = lazy_frame.to_rust(cr);
        let Abstract(expr) = expr.to_rust(cr);
        OCaml::box_value(cr, lazy_frame.filter(expr))
    }

    fn rust_lazy_frame_select(cr, lazy_frame: OCamlRef<DynBox<LazyFrame>>, exprs: OCamlRef<OCamlList<DynBox<Expr>>>) -> OCaml<DynBox<LazyFrame>> {
        let exprs = unwrap_abstract_vec(exprs.to_rust(cr));
        let Abstract(lazy_frame) = lazy_frame.to_rust(cr);
        OCaml::box_value(cr, lazy_frame.select(&exprs))
    }

    fn rust_lazy_frame_with_columns(cr, lazy_frame: OCamlRef<DynBox<LazyFrame>>, exprs: OCamlRef<OCamlList<DynBox<Expr>>>) -> OCaml<DynBox<LazyFrame>> {
        let exprs = unwrap_abstract_vec(exprs.to_rust(cr));
        let Abstract(lazy_frame) = lazy_frame.to_rust(cr);
        OCaml::box_value(cr, lazy_frame.with_columns(&exprs))
    }

    fn rust_lazy_frame_groupby(cr, lazy_frame: OCamlRef<DynBox<LazyFrame>>, is_stable: OCamlRef<bool>, by: OCamlRef<OCamlList<DynBox<Expr>>>, agg: OCamlRef<OCamlList<DynBox<Expr>>>) -> OCaml<DynBox<LazyFrame>> {
        let is_stable = is_stable.to_rust(cr);
        let by = unwrap_abstract_vec(by.to_rust(cr));
        let agg = unwrap_abstract_vec(agg.to_rust(cr));
        let Abstract(lazy_frame) = lazy_frame.to_rust(cr);
        let groupby = if is_stable { lazy_frame.groupby_stable(by) } else { lazy_frame.groupby(by) };
        OCaml::box_value(cr, groupby.agg(agg))
    }

    fn rust_lazy_frame_schema(cr, lazy_frame: OCamlRef<DynBox<LazyFrame>>) -> OCaml<Result<DynBox<Schema>,String>> {
        let Abstract(lazy_frame) = lazy_frame.to_rust(cr);
        lazy_frame.schema()
        .map(|schema| Abstract((*schema).clone()))
        .map_err(|err| err.to_string()).to_ocaml(cr)
    }

    fn rust_schema_create(cr, fields: OCamlRef<OCamlList<(String, DataType)>>) -> OCaml<DynBox<Schema>> {
        let fields: Vec<(String, PolarsDataType)> = fields.to_rust(cr);
        let schema: Schema =
            fields
            .into_iter()
            .map(|(name, PolarsDataType(data_type))| Field::new(&name, data_type))
            .collect();
        OCaml::box_value(cr, schema)
    }

    fn rust_schema_to_fields(cr, schema: OCamlRef<DynBox<Schema>>) -> OCaml<OCamlList<(String, DataType)>> {
        let Abstract(schema) = schema.to_rust(cr);
        let fields: Vec<(String, PolarsDataType)> =
            schema
            .iter_fields()
            .map(|Field { name, dtype }| (name.to_string(), PolarsDataType(dtype)))
            .collect();
        fields.to_ocaml(cr)
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
