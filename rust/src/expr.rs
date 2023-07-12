use ocaml_interop::{
    ocaml_export, DynBox, OCaml, OCamlFloat, OCamlInt, OCamlList, OCamlRef, OCamlRuntime, ToOCaml,
};
use polars::lazy::dsl::GetOutput;
use polars::prelude::*;

use crate::utils::PolarsDataType;
use crate::utils::*;

fn expr_unary_op<'a>(
    cr: &'a mut &'a mut OCamlRuntime,
    expr: OCamlRef<'a, DynBox<Expr>>,
    f: impl FnOnce(Expr) -> Expr,
) -> OCaml<'a, DynBox<Expr>> {
    let Abstract(expr) = expr.to_rust(cr);
    OCaml::box_value(cr, f(expr))
}

fn expr_binary_op<'a>(
    cr: &'a mut &'a mut OCamlRuntime,
    expr: OCamlRef<'a, DynBox<Expr>>,
    other: OCamlRef<'a, DynBox<Expr>>,
    f: impl FnOnce(Expr, Expr) -> Expr,
) -> OCaml<'a, DynBox<Expr>> {
    let Abstract(expr) = expr.to_rust(cr);
    let Abstract(other) = other.to_rust(cr);
    OCaml::box_value(cr, f(expr, other))
}

fn expr_ternary_op<'a>(
    cr: &'a mut &'a mut OCamlRuntime,
    expr: OCamlRef<'a, DynBox<Expr>>,
    other: OCamlRef<'a, DynBox<Expr>>,
    other2: OCamlRef<'a, DynBox<Expr>>,
    f: impl FnOnce(Expr, Expr, Expr) -> Expr,
) -> OCaml<'a, DynBox<Expr>> {
    let Abstract(expr) = expr.to_rust(cr);
    let Abstract(other) = other.to_rust(cr);
    let Abstract(other2) = other2.to_rust(cr);
    OCaml::box_value(cr, f(expr, other, other2))
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

enum WhenThenClause {
    Empty,
    WhenThen(WhenThen),
    WhenThenThen(WhenThenThen),
}

ocaml_export! {
    fn rust_expr_col(cr, name: OCamlRef<String>) -> OCaml<DynBox<Expr>> {
        let name: String = name.to_rust(cr);
        OCaml::box_value(cr, col(&name))
    }

    fn rust_expr_cols(cr, names: OCamlRef<OCamlList<String>>) -> OCaml<DynBox<Expr>> {
        let names: Vec<String> = names.to_rust(cr);
        OCaml::box_value(cr, cols(&names))
    }

    fn rust_expr_all(cr, unit: OCamlRef<()>) -> OCaml<DynBox<Expr>> {
        let _: () = unit.to_rust(cr);
        OCaml::box_value(cr, all())
    }

    fn rust_expr_exclude(cr, name: OCamlRef<String>) -> OCaml<DynBox<Expr>> {
        let name: String = name.to_rust(cr);
        OCaml::box_value(cr, all().exclude([name]))
    }

    fn rust_expr_null(cr, unit: OCamlRef<()>) -> OCaml<DynBox<Expr>> {
        let (): () = unit.to_rust(cr);
        OCaml::box_value(cr, lit(NULL))
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

    fn rust_expr_sort_by(cr, expr: OCamlRef<DynBox<Expr>>, descending: OCamlRef<bool>, by: OCamlRef<OCamlList<DynBox<Expr>>>, ) -> OCaml<DynBox<Expr>> {
        let by = unwrap_abstract_vec(by.to_rust(cr));
        let descending: bool = descending.to_rust(cr);
        let descending = vec![descending; by.len()];
        expr_unary_op(cr, expr, |expr| expr.sort_by(by, descending))
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

    fn rust_expr_median(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.median())
    }

    fn rust_expr_max(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.max())
    }

    fn rust_expr_min(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.min())
    }

    fn rust_expr_count(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.count())
    }

    fn rust_expr_count_(cr, unit: OCamlRef<()>) -> OCaml<DynBox<Expr>> {
        let () = unit.to_rust(cr);
        OCaml::box_value(cr, count())
    }

    fn rust_expr_n_unique(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.n_unique())
    }

    fn rust_expr_approx_unique(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.approx_unique())
    }

    fn rust_expr_explode(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.explode())
    }

    fn rust_expr_over(cr, expr: OCamlRef<DynBox<Expr>>, partition_by: OCamlRef<OCamlList<DynBox<Expr>>>, mapping_strategy: OCamlRef<WindowMapping>) -> OCaml<DynBox<Expr>> {
        let PolarsWindowMapping(mapping_strategy) = mapping_strategy.to_rust(cr);
        let partition_by: Vec<_> = unwrap_abstract_vec(partition_by.to_rust(cr));
        expr_unary_op(cr, expr, |expr| expr.over_with_options(&partition_by, WindowOptions { mapping: mapping_strategy }))
    }

    fn rust_expr_concat_list(cr, exprs: OCamlRef<OCamlList<DynBox<Expr>>>) -> OCaml<Result<DynBox<Expr>,String>> {
        let exprs: Vec<_> = unwrap_abstract_vec(exprs.to_rust(cr));
        concat_list(exprs).map(Abstract).map_err(|e| e.to_string()).to_ocaml(cr)
    }

    fn rust_expr_null_count(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.null_count())
    }

    fn rust_expr_is_null(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.is_null())
    }

    fn rust_expr_is_not_null(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.is_not_null())
    }

    fn rust_expr_fill_null(cr, expr: OCamlRef<DynBox<Expr>>, with: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, with, |expr, with| expr.fill_null(with))
    }

    fn rust_expr_fill_null_with_strategy(cr, expr: OCamlRef<DynBox<Expr>>, strategy: OCamlRef<FillNullStrategy>) -> OCaml<DynBox<Expr>> {
        let PolarsFillNullStrategy(strategy) = strategy.to_rust(cr);
        expr_series_map(cr, expr, move |series| Ok(Some(series.fill_null(strategy)?)), GetOutput::default())
    }

    fn rust_expr_interpolate(cr, expr: OCamlRef<DynBox<Expr>>, method: OCamlRef<InterpolationMethod>) -> OCaml<DynBox<Expr>> {
        let PolarsInterpolationMethod(method) = method.to_rust(cr);
        expr_unary_op(cr, expr, |expr| expr.interpolate(method))
    }

    fn rust_expr_fill_nan(cr, expr: OCamlRef<DynBox<Expr>>, with: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, with, |expr, with| expr.fill_nan(with))
    }

    fn rust_expr_rank(cr, expr: OCamlRef<DynBox<Expr>>, method: OCamlRef<RankMethod>, descending: OCamlRef<bool>, seed: OCamlRef<Option<OCamlInt>>) -> OCaml<Option<DynBox<Expr>>> {
        let result: Option<_> = try {
            let Abstract(expr) = expr.to_rust(cr);
            let PolarsRankMethod(method) = method.to_rust(cr);
            let descending: bool = descending.to_rust(cr);
            let seed: Option<Result<u64,_>> = seed.to_rust::<Option<i64>>(cr).map(|seed| seed.try_into());
            let seed: Option<u64> = seed.map_or(Ok(None), |seed| seed.map(Some)).ok()?;

            Abstract(expr.rank(RankOptions { method, descending }, seed))
        };

        result.to_ocaml(cr)
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

    fn rust_expr_shift(cr, expr: OCamlRef<DynBox<Expr>>, periods: OCamlRef<OCamlInt>, fill_value: OCamlRef<Option<DynBox<Expr>>>) -> OCaml<DynBox<Expr>> {
        let Abstract(expr) = expr.to_rust(cr);
        let periods: i64 = periods.to_rust(cr);
        let fill_value: Option<Abstract<Expr>> = fill_value.to_rust(cr);

        let expr =
            match fill_value {
                None => expr.shift(periods),
                Some(Abstract(fill_value)) => expr.shift_and_fill(periods, fill_value),
            };
        Abstract(expr).to_ocaml(cr)
    }

    fn rust_expr_cumcount(cr, expr: OCamlRef<DynBox<Expr>>, reverse: OCamlRef<bool>) -> OCaml<DynBox<Expr>> {
        let reverse: bool = reverse.to_rust(cr);
        expr_unary_op(cr, expr, |expr| expr.cumcount(reverse))
    }

    fn rust_expr_cumsum(cr, expr: OCamlRef<DynBox<Expr>>, reverse: OCamlRef<bool>) -> OCaml<DynBox<Expr>> {
        let reverse: bool = reverse.to_rust(cr);
        expr_unary_op(cr, expr, |expr| expr.cumsum(reverse))
    }

    fn rust_expr_cumprod(cr, expr: OCamlRef<DynBox<Expr>>, reverse: OCamlRef<bool>) -> OCaml<DynBox<Expr>> {
        let reverse: bool = reverse.to_rust(cr);
        expr_unary_op(cr, expr, |expr| expr.cumprod(reverse))
    }

    fn rust_expr_cummin(cr, expr: OCamlRef<DynBox<Expr>>, reverse: OCamlRef<bool>) -> OCaml<DynBox<Expr>> {
        let reverse: bool = reverse.to_rust(cr);
        expr_unary_op(cr, expr, |expr| expr.cummin(reverse))
    }

    fn rust_expr_cummax(cr, expr: OCamlRef<DynBox<Expr>>, reverse: OCamlRef<bool>) -> OCaml<DynBox<Expr>> {
        let reverse: bool = reverse.to_rust(cr);
        expr_unary_op(cr, expr, |expr| expr.cummax(reverse))
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

    fn rust_expr_round(cr, expr: OCamlRef<DynBox<Expr>>, decimals: OCamlRef<OCamlInt>) -> OCaml<Option<DynBox<Expr>>> {
        let result: Option<_> = try {
            let decimals: u32 = decimals.to_rust::<i64>(cr).try_into().ok()?;

            let Abstract(expr) = expr.to_rust(cr);
            Abstract(expr.round(decimals))
        };
        result.to_ocaml(cr)
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

    fn rust_expr_floor_div(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a.floor_div(b))
    }

    fn rust_expr_dt_strftime(cr, expr: OCamlRef<DynBox<Expr>>, format:OCamlRef<String>)-> OCaml<DynBox<Expr>> {
        let format: String = format.to_rust(cr);
        expr_unary_op(cr, expr, |expr| expr.dt().to_string(&format))
    }

    fn rust_expr_dt_convert_time_zone(cr, expr: OCamlRef<DynBox<Expr>>, timezone: OCamlRef<String>) -> OCaml<DynBox<Expr>> {
        let timezone: String = timezone.to_rust(cr);
        expr_unary_op(cr, expr, |expr| expr.dt().convert_time_zone(timezone))
    }

    fn rust_expr_dt_year(cr, expr: OCamlRef<DynBox<Expr>>)-> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.dt().year())
    }

    fn rust_expr_dt_month(cr, expr: OCamlRef<DynBox<Expr>>)-> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.dt().month())
    }

    fn rust_expr_dt_day(cr, expr: OCamlRef<DynBox<Expr>>)-> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.dt().day())
    }

    fn rust_expr_str_split(cr, expr: OCamlRef<DynBox<Expr>>, by: OCamlRef<String>, inclusive: OCamlRef<bool>) -> OCaml<DynBox<Expr>> {
        let by: String = by.to_rust(cr);
        let inclusive = inclusive.to_rust(cr);
        expr_unary_op(cr, expr, |expr| if inclusive { expr.str().split_inclusive(&by) } else { expr.str().split(&by) })
    }

    fn rust_expr_str_strptime(cr, expr: OCamlRef<DynBox<Expr>>, data_type: OCamlRef<DataType>, format:OCamlRef<String>) -> OCaml<DynBox<Expr>> {
        let PolarsDataType(data_type): PolarsDataType = data_type.to_rust(cr);
        let format: String = format.to_rust(cr);

        expr_unary_op(cr, expr, |expr| {
            let options = StrptimeOptions {
                format: Some(format.clone()), strict: true, exact:true, cache: false
            };
            expr.str().strptime(data_type.clone(), options)
        })
    }

    fn rust_expr_str_lengths(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_series_map(cr, expr, |series| {
            Ok(Some(series.utf8()?.str_lengths().into_series()))
        }, GetOutput::from_type(DataType::UInt32))
    }

    fn rust_expr_str_n_chars(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_series_map(cr, expr, |series| {
            Ok(Some(series.utf8()?.str_n_chars().into_series()))
        }, GetOutput::from_type(DataType::UInt32))
    }

    fn rust_expr_str_contains(cr, expr: OCamlRef<DynBox<Expr>>, pat: OCamlRef<String>, literal: OCamlRef<bool>) -> OCaml<DynBox<Expr>> {
        let pat: String = pat.to_rust(cr);
        let literal: bool = literal.to_rust(cr);
        expr_series_map(cr, expr, move |series| {
            let chunked = series.utf8()?;
            let contains =
                if literal { chunked.contains_literal(&pat) } else { chunked.contains(&pat, true) };
            Ok(Some(contains?.into_series()))
        }, GetOutput::from_type(DataType::Boolean))
    }

    fn rust_expr_str_starts_with(cr, expr: OCamlRef<DynBox<Expr>>, prefix: OCamlRef<String>) -> OCaml<DynBox<Expr>> {
        let prefix: String = prefix.to_rust(cr);
        expr_series_map(cr, expr, move |series| {
            Ok(Some(series.utf8()?.starts_with(&prefix).into_series()))
        }, GetOutput::from_type(DataType::Boolean))
    }

    fn rust_expr_str_ends_with(cr, expr: OCamlRef<DynBox<Expr>>, suffix: OCamlRef<String>) -> OCaml<DynBox<Expr>> {
        let suffix: String = suffix.to_rust(cr);
        expr_series_map(cr, expr, move |series| {
            Ok(Some(series.utf8()?.ends_with(&suffix).into_series()))
        }, GetOutput::from_type(DataType::Boolean))
    }

    fn rust_expr_str_extract(cr, expr: OCamlRef<DynBox<Expr>>, pat: OCamlRef<String>, group_index: OCamlRef<OCamlInt>) -> OCaml<Option<DynBox<Expr>>> {
        let result: Option<_> = try {
            let pat: String = pat.to_rust(cr);
            let group_index: usize = group_index.to_rust::<i64>(cr).try_into().ok()?;

            let Abstract(expr) = expr.to_rust(cr);
            let f = move |series: Series| {
                Ok(Some(series.utf8()?.extract(&pat, group_index)?.into_series()))
            };
            Abstract(expr.map(f, GetOutput::from_type(DataType::Utf8)))
        };
        result.to_ocaml(cr)
    }

    fn rust_expr_str_extract_all(cr, expr: OCamlRef<DynBox<Expr>>, pat: OCamlRef<String>) -> OCaml<DynBox<Expr>> {
        let pat: String = pat.to_rust(cr);
        expr_series_map(cr, expr, move |series| {
            Ok(Some(series.utf8()?.extract_all(&pat)?.into_series()))
        }, GetOutput::from_type(DataType::List(Box::new(DataType::Utf8))))
    }

    fn rust_expr_str_replace(cr, expr: OCamlRef<DynBox<Expr>>, pat: OCamlRef<String>, with: OCamlRef<String>, literal: OCamlRef<bool>) -> OCaml<DynBox<Expr>> {
        let pat: String = pat.to_rust(cr);
        let with: String = with.to_rust(cr);
        let literal: bool = literal.to_rust(cr);
        expr_series_map(cr, expr, move |series| {
            let chunked = series.utf8()?;
            let replaced =
                if literal { chunked.replace_literal(&pat, &with, 1) } else { chunked.replace(&pat, &with) };
            Ok(Some(replaced?.into_series()))
        }, GetOutput::from_type(DataType::List(Box::new(DataType::Utf8))))
    }

    fn rust_expr_str_replace_all(cr, expr: OCamlRef<DynBox<Expr>>, pat: OCamlRef<String>, with: OCamlRef<String>, literal: OCamlRef<bool>) -> OCaml<DynBox<Expr>> {
        let pat: String = pat.to_rust(cr);
        let with: String = with.to_rust(cr);
        let literal: bool = literal.to_rust(cr);
        expr_series_map(cr, expr, move |series| {
            let chunked = series.utf8()?;
            let replaced =
                if literal { chunked.replace_literal_all(&pat, &with) } else { chunked.replace_all(&pat, &with) };
            Ok(Some(replaced?.into_series()))
        }, GetOutput::from_type(DataType::List(Box::new(DataType::Utf8))))
    }

    fn rust_expr_list_lengths(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.list().lengths())
    }

    fn rust_expr_list_slice(cr, expr: OCamlRef<DynBox<Expr>>, offset: OCamlRef<DynBox<Expr>>, length: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_ternary_op(cr, expr, offset, length, |expr, offset, length| expr.list().slice(offset, length))
    }

    fn rust_expr_list_head(cr, expr: OCamlRef<DynBox<Expr>>, n: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, n, |expr, n| expr.list().head(n))
    }

    fn rust_expr_list_tail(cr, expr: OCamlRef<DynBox<Expr>>, n: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, n, |expr, n| expr.list().tail(n))
    }

    fn rust_expr_list_sum(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_unary_op(cr, expr, |expr| expr.list().sum())
    }

    fn rust_expr_list_eval(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>, parallel: OCamlRef<bool>) -> OCaml<DynBox<Expr>> {
        let parallel = parallel.to_rust(cr);
        expr_binary_op(cr, expr, other, |expr, other| expr.list().eval(other, parallel))
    }
}
