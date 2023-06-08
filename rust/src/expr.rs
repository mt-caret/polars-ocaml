use ocaml_interop::{
    ocaml_export, DynBox, OCaml, OCamlFloat, OCamlInt, OCamlList, OCamlRef, OCamlRuntime, ToOCaml,
};
use polars::prelude::*;

use crate::utils::PolarsDataType;
use crate::utils::*;

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
}
