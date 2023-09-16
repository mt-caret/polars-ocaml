// TODO: rust_lazy_frame_profile's return type is triggering this warning, but
// due to how I've implemented the proc macro specifying the allow clippy
// attribute alongside the proc macro invocation doesn't work, which probably
// is fixable; disabling the warning file-wide in the interim.
#![allow(clippy::type_complexity)]
use crate::utils::*;
use ocaml_interop::{DynBox, OCaml, OCamlInt, OCamlList, OCamlRef, ToOCaml};
use polars::prelude::*;
use polars_ocaml_macros::ocaml_interop_export;
use smartstring::{LazyCompact, SmartString};
use std::rc::Rc;
use std::{cell::RefCell, path::Path};

#[ocaml_interop_export]
fn rust_lazy_frame_scan_csv(
    cr: &mut &mut OCamlRuntime,
    path: OCamlRef<String>,
) -> OCaml<Result<DynBox<LazyFrame>, String>> {
    let path: String = path.to_rust(cr);

    LazyCsvReader::new(path)
        .finish()
        .map(Abstract)
        .map_err(|err| err.to_string())
        .to_ocaml(cr)
}

// TODO: properly return error type instead of a string
#[ocaml_interop_export]
fn rust_lazy_frame_scan_parquet(
    cr: &mut &mut OCamlRuntime,
    path: OCamlRef<String>,
) -> OCaml<Result<DynBox<LazyFrame>, String>> {
    let path: String = path.to_rust(cr);
    let path: &Path = Path::new(&path);

    LazyFrame::scan_parquet(path, Default::default())
        .map(Abstract)
        .map_err(|err| err.to_string())
        .to_ocaml(cr)
}

// TODO: Polars only has a lazy version of JSON Lines reader (and no lazy version of JSON reader),
// which I think is mainly because in the case of JSON it loads a single array. If we have a
// SAX-like parser for JSON, I don't see why we can't have a lazy JSON reader too.
#[ocaml_interop_export]
fn rust_lazy_frame_scan_jsonl(
    cr: &mut &mut OCamlRuntime,
    path: OCamlRef<String>,
) -> OCaml<Result<DynBox<LazyFrame>, String>> {
    let path: String = path.to_rust(cr);
    let path: &Path = Path::new(&path);

    LazyJsonLineReader::new(path)
        .finish()
        .map(Abstract)
        .map_err(|err| err.to_string())
        .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_lazy_frame_explain(
    cr: &mut &mut OCamlRuntime,
    lazy_frame: OCamlRef<DynBox<LazyFrame>>,
    optimized: OCamlRef<bool>,
) -> OCaml<Result<String, String>> {
    let Abstract(lazy_frame) = lazy_frame.to_rust(cr);
    let optimized = optimized.to_rust(cr);

    lazy_frame
        .explain(optimized)
        .map_err(|err| err.to_string())
        .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_lazy_frame_cache(
    cr: &mut &mut OCamlRuntime,
    lazy_frame: OCamlRef<DynBox<LazyFrame>>,
) -> OCaml<DynBox<LazyFrame>> {
    dyn_box!(cr, |lazy_frame| lazy_frame.cache())
}

#[ocaml_interop_export]
fn rust_lazy_frame_to_dot(
    cr: &mut &mut OCamlRuntime,
    lazy_frame: OCamlRef<DynBox<LazyFrame>>,
    optimized: OCamlRef<bool>,
) -> OCaml<Result<String, String>> {
    let Abstract(lazy_frame) = lazy_frame.to_rust(cr);
    let optimized = optimized.to_rust(cr);

    lazy_frame
        .to_dot(optimized)
        .map_err(|err| err.to_string())
        .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_lazy_frame_collect(
    cr: &mut &mut OCamlRuntime,
    lazy_frame: OCamlRef<DynBox<LazyFrame>>,
    streaming: OCamlRef<bool>,
) -> OCaml<Result<DynBox<crate::data_frame::PolarsDataFrame>, String>> {
    let streaming = streaming.to_rust(cr);

    dyn_box_result!(cr, |lazy_frame| {
        cr.releasing_runtime(|| {
            lazy_frame
                .with_streaming(streaming)
                .collect()
                .map(|df| Rc::new(RefCell::new(df)))
        })
    })
}

#[ocaml_interop_export]
fn rust_lazy_frame_collect_all(
    cr: &mut &mut OCamlRuntime,
    lazy_frames: OCamlRef<OCamlList<DynBox<LazyFrame>>>,
) -> OCaml<Result<OCamlList<DynBox<crate::data_frame::PolarsDataFrame>>, String>> {
    let lazy_frames = unwrap_abstract_vec(lazy_frames.to_rust(cr));

    cr.releasing_runtime(|| {
        collect_all(lazy_frames)
            .map(|data_frames| {
                data_frames
                    .into_iter()
                    .map(|df| Abstract(Rc::new(RefCell::new(df))))
                    .collect::<Vec<_>>()
            })
            .map_err(|err| err.to_string())
    })
    .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_lazy_frame_profile(
    cr: &mut &mut OCamlRuntime,
    lazy_frame: OCamlRef<DynBox<LazyFrame>>,
) -> OCaml<
    Result<
        (
            DynBox<crate::data_frame::PolarsDataFrame>,
            DynBox<crate::data_frame::PolarsDataFrame>,
        ),
        String,
    >,
> {
    let Abstract(lazy_frame) = lazy_frame.to_rust(cr);

    cr.releasing_runtime(|| {
        lazy_frame
            .profile()
            .map(|(materialized, profile)| {
                (
                    Abstract(Rc::new(RefCell::new(materialized))),
                    Abstract(Rc::new(RefCell::new(profile))),
                )
            })
            .map_err(|err| err.to_string())
    })
    .to_ocaml(cr)
}

#[ocaml_interop_export(raise_on_err)]
fn rust_lazy_frame_fetch(
    cr: &mut &mut OCamlRuntime,
    lazy_frame: OCamlRef<DynBox<LazyFrame>>,
    n_rows: OCamlRef<OCamlInt>,
) -> OCaml<Result<DynBox<crate::data_frame::PolarsDataFrame>, String>> {
    let n_rows = n_rows.to_rust::<Coerce<_, i64, usize>>(cr).get()?;

    dyn_box_result!(cr, |lazy_frame| {
        cr.releasing_runtime(|| lazy_frame.fetch(n_rows).map(|df| Rc::new(RefCell::new(df))))
    })
}

#[ocaml_interop_export]
fn rust_lazy_frame_filter(
    cr: &mut &mut OCamlRuntime,
    lazy_frame: OCamlRef<DynBox<LazyFrame>>,
    expr: OCamlRef<DynBox<Expr>>,
) -> OCaml<DynBox<LazyFrame>> {
    dyn_box!(cr, |lazy_frame, expr| lazy_frame.filter(expr))
}

#[ocaml_interop_export]
fn rust_lazy_frame_select(
    cr: &mut &mut OCamlRuntime,
    lazy_frame: OCamlRef<DynBox<LazyFrame>>,
    exprs: OCamlRef<OCamlList<DynBox<Expr>>>,
) -> OCaml<DynBox<LazyFrame>> {
    let exprs = unwrap_abstract_vec(exprs.to_rust(cr));

    dyn_box!(cr, |lazy_frame| lazy_frame.select(&exprs))
}

#[ocaml_interop_export]
fn rust_lazy_frame_with_columns(
    cr: &mut &mut OCamlRuntime,
    lazy_frame: OCamlRef<DynBox<LazyFrame>>,
    exprs: OCamlRef<OCamlList<DynBox<Expr>>>,
) -> OCaml<DynBox<LazyFrame>> {
    let exprs = unwrap_abstract_vec(exprs.to_rust(cr));

    dyn_box!(cr, |lazy_frame| lazy_frame.with_columns(&exprs))
}

#[ocaml_interop_export]
fn rust_lazy_frame_groupby(
    cr: &mut &mut OCamlRuntime,
    lazy_frame: OCamlRef<DynBox<LazyFrame>>,
    is_stable: OCamlRef<bool>,
    by: OCamlRef<OCamlList<DynBox<Expr>>>,
    agg: OCamlRef<OCamlList<DynBox<Expr>>>,
) -> OCaml<DynBox<LazyFrame>> {
    let is_stable = is_stable.to_rust(cr);
    let by = unwrap_abstract_vec(by.to_rust(cr));
    let agg = unwrap_abstract_vec(agg.to_rust(cr));

    dyn_box!(cr, |lazy_frame| {
        let groupby = if is_stable {
            lazy_frame.groupby_stable(by)
        } else {
            lazy_frame.groupby(by)
        };
        groupby.agg(agg)
    })
}

#[ocaml_interop_export]
fn rust_lazy_frame_groupby_dynamic(
    cr: &mut &mut OCamlRuntime,
    lazy_frame: OCamlRef<DynBox<LazyFrame>>,
    index_column: OCamlRef<DynBox<Expr>>,
    by: OCamlRef<OCamlList<DynBox<Expr>>>,
    every: OCamlRef<Option<String>>,
    period: OCamlRef<Option<String>>,
    offset: OCamlRef<Option<String>>,
    truncate: OCamlRef<Option<bool>>,
    include_boundaries: OCamlRef<Option<bool>>,
    closed_window: OCamlRef<Option<ClosedWindow>>,
    start_by: OCamlRef<Option<StartBy>>,
    check_sorted: OCamlRef<Option<bool>>,
    agg: OCamlRef<OCamlList<DynBox<Expr>>>,
) -> OCaml<DynBox<LazyFrame>> {
    let every = every
        .to_rust::<Option<String>>(cr)
        .as_deref()
        .map(Duration::parse);
    let period = period
        .to_rust::<Option<String>>(cr)
        .as_deref()
        .map(Duration::parse);
    let offset = offset
        .to_rust::<Option<String>>(cr)
        .as_deref()
        .map(Duration::parse);
    let truncate: Option<bool> = truncate.to_rust(cr);
    let include_boundaries: Option<bool> = include_boundaries.to_rust(cr);
    let closed_window = closed_window
        .to_rust::<Option<PolarsClosedWindow>>(cr)
        .map(|PolarsClosedWindow(closed_window)| closed_window);
    let start_by = start_by
        .to_rust::<Option<PolarsStartBy>>(cr)
        .map(|PolarsStartBy(start_by)| start_by);
    let check_sorted: Option<bool> = check_sorted.to_rust(cr);

    let options: DynamicGroupOptions = Default::default();
    let options = DynamicGroupOptions {
        // index_column is set within LazyFrame::groupby_dynamic()
        index_column: "".into(),
        every: every.unwrap_or(options.every),
        period: period.unwrap_or(options.period),
        offset: offset.unwrap_or(options.offset),
        truncate: truncate.unwrap_or(options.truncate),
        include_boundaries: include_boundaries.unwrap_or(options.include_boundaries),
        closed_window: closed_window.unwrap_or(options.closed_window),
        start_by: start_by.unwrap_or(options.start_by),
        check_sorted: check_sorted.unwrap_or(options.check_sorted),
    };

    let by = unwrap_abstract_vec(by.to_rust(cr));

    let agg = unwrap_abstract_vec(agg.to_rust(cr));

    dyn_box!(cr, |lazy_frame, index_column| {
        lazy_frame
            .groupby_dynamic(index_column, by, options)
            .agg(agg)
    })
}

#[ocaml_interop_export]
fn rust_lazy_frame_join(
    cr: &mut &mut OCamlRuntime,
    lazy_frame: OCamlRef<DynBox<LazyFrame>>,
    other: OCamlRef<DynBox<LazyFrame>>,
    left_on: OCamlRef<OCamlList<DynBox<Expr>>>,
    right_on: OCamlRef<OCamlList<DynBox<Expr>>>,
    how: OCamlRef<JoinType>,
) -> OCaml<DynBox<LazyFrame>> {
    let left_on = unwrap_abstract_vec(left_on.to_rust(cr));
    let right_on = unwrap_abstract_vec(right_on.to_rust(cr));
    let PolarsJoinType(how) = how.to_rust(cr);

    dyn_box!(cr, |lazy_frame, other| {
        lazy_frame.join(other, &left_on, &right_on, JoinArgs::new(how))
    })
}

#[ocaml_interop_export]
fn rust_lazy_frame_sort(
    cr: &mut &mut OCamlRuntime,
    lazy_frame: OCamlRef<DynBox<LazyFrame>>,
    by_column: OCamlRef<String>,
    descending: OCamlRef<Option<bool>>,
    nulls_last: OCamlRef<Option<bool>>,
    multithreaded: OCamlRef<Option<bool>>,
    maintain_order: OCamlRef<Option<bool>>,
) -> OCaml<DynBox<LazyFrame>> {
    let by_column: String = by_column.to_rust(cr);
    let descending: Option<bool> = descending.to_rust(cr);
    let nulls_last: Option<bool> = nulls_last.to_rust(cr);
    let multithreaded: Option<bool> = multithreaded.to_rust(cr);
    let maintain_order: Option<bool> = maintain_order.to_rust(cr);
    let sort_options: SortOptions = Default::default();
    let sort_options = SortOptions {
        descending: descending.unwrap_or(sort_options.descending),
        nulls_last: nulls_last.unwrap_or(sort_options.nulls_last),
        multithreaded: multithreaded.unwrap_or(sort_options.multithreaded),
        maintain_order: maintain_order.unwrap_or(sort_options.maintain_order),
    };

    dyn_box!(cr, |lazy_frame| lazy_frame.sort(&by_column, sort_options))
}

#[ocaml_interop_export]
fn rust_lazy_frame_vertical_concat(
    cr: &mut &mut OCamlRuntime,
    lazy_frames: OCamlRef<OCamlList<DynBox<LazyFrame>>>,
    rechunk: OCamlRef<bool>,
    parallel: OCamlRef<bool>,
    to_supertypes: OCamlRef<bool>,
) -> OCaml<Result<DynBox<LazyFrame>, String>> {
    let lazy_frames = unwrap_abstract_vec(lazy_frames.to_rust(cr));
    let rechunk = rechunk.to_rust(cr);
    let parallel = parallel.to_rust(cr);
    let to_supertypes = to_supertypes.to_rust(cr);
    concat(
        &lazy_frames,
        UnionArgs {
            rechunk,
            parallel,
            to_supertypes,
        },
    )
    .map(Abstract)
    .map_err(|err| err.to_string())
    .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_lazy_frame_diagonal_concat(
    cr: &mut &mut OCamlRuntime,
    lazy_frames: OCamlRef<OCamlList<DynBox<LazyFrame>>>,
    rechunk: OCamlRef<bool>,
    parallel: OCamlRef<bool>,
) -> OCaml<Result<DynBox<LazyFrame>, String>> {
    let lazy_frames = unwrap_abstract_vec(lazy_frames.to_rust(cr));
    let rechunk = rechunk.to_rust(cr);
    let parallel = parallel.to_rust(cr);
    diag_concat_lf(&lazy_frames, rechunk, parallel)
        .map(Abstract)
        .map_err(|err| err.to_string())
        .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_lazy_frame_melt(
    cr: &mut &mut OCamlRuntime,
    lazy_frame: OCamlRef<DynBox<LazyFrame>>,
    id_vars: OCamlRef<OCamlList<String>>,
    value_vars: OCamlRef<OCamlList<String>>,
    variable_name: OCamlRef<Option<String>>,
    value_name: OCamlRef<Option<String>>,
    streamable: OCamlRef<bool>,
) -> OCaml<DynBox<LazyFrame>> {
    let id_vars: Vec<SmartString<LazyCompact>> = id_vars
        .to_rust::<Vec<String>>(cr)
        .into_iter()
        .map(|s| s.into())
        .collect();
    let value_vars: Vec<SmartString<LazyCompact>> = value_vars
        .to_rust::<Vec<String>>(cr)
        .into_iter()
        .map(|s| s.into())
        .collect();
    let variable_name: Option<SmartString<LazyCompact>> = variable_name
        .to_rust::<Option<String>>(cr)
        .map(|s| s.into());
    let value_name: Option<SmartString<LazyCompact>> =
        value_name.to_rust::<Option<String>>(cr).map(|s| s.into());
    let streamable: bool = streamable.to_rust(cr);

    let melt_args = MeltArgs {
        id_vars,
        value_vars,
        variable_name,
        value_name,
        streamable,
    };

    dyn_box!(cr, |lazy_frame| lazy_frame.melt(melt_args))
}

#[ocaml_interop_export(raise_on_err)]
fn rust_lazy_frame_limit(
    cr: &mut &mut OCamlRuntime,
    lazy_frame: OCamlRef<DynBox<LazyFrame>>,
    n: OCamlRef<OCamlInt>,
) -> OCaml<DynBox<LazyFrame>> {
    let n = n.to_rust::<Coerce<_, i64, u32>>(cr).get()?;

    dyn_box!(cr, |lazy_frame| lazy_frame.limit(n))
}

#[ocaml_interop_export]
fn rust_lazy_frame_explode(
    cr: &mut &mut OCamlRuntime,
    lazy_frame: OCamlRef<DynBox<LazyFrame>>,
    columns: OCamlRef<OCamlList<DynBox<Expr>>>,
) -> OCaml<DynBox<LazyFrame>> {
    let columns = unwrap_abstract_vec(columns.to_rust(cr));

    dyn_box!(cr, |lazy_frame| lazy_frame.explode(&columns))
}

#[ocaml_interop_export]
fn rust_lazy_frame_with_streaming(
    cr: &mut &mut OCamlRuntime,
    lazy_frame: OCamlRef<DynBox<LazyFrame>>,
    toggle: OCamlRef<bool>,
) -> OCaml<DynBox<LazyFrame>> {
    let toggle = toggle.to_rust(cr);

    dyn_box!(cr, |lazy_frame| lazy_frame.with_streaming(toggle))
}

#[ocaml_interop_export]
fn rust_lazy_frame_schema(
    cr: &mut &mut OCamlRuntime,
    lazy_frame: OCamlRef<DynBox<LazyFrame>>,
) -> OCaml<Result<DynBox<Schema>, String>> {
    dyn_box_result!(cr, |lazy_frame| {
        lazy_frame.schema().map(|schema| (*schema).clone())
    })
}
