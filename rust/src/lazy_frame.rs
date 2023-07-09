use crate::utils::*;
use ocaml_interop::{ocaml_export, DynBox, OCaml, OCamlInt, OCamlList, OCamlRef, ToOCaml};
use polars::prelude::*;
use smartstring::{LazyCompact, SmartString};
use std::path::Path;

ocaml_export! {
    fn rust_lazy_frame_scan_csv(cr, path: OCamlRef<String>) -> OCaml<Result<DynBox<LazyFrame>, String>>{
        let path: String = path.to_rust(cr);

        LazyCsvReader::new(path).finish()
        .map(Abstract).map_err(|err| err.to_string())
        .to_ocaml(cr)
    }

    // TODO: properly return error type instead of a string
    fn rust_lazy_frame_scan_parquet(cr, path: OCamlRef<String>) -> OCaml<Result<DynBox<LazyFrame>, String>>{
        let path: String = path.to_rust(cr);
        let path: &Path = Path::new(&path);

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

    fn rust_lazy_frame_join(cr, lazy_frame: OCamlRef<DynBox<LazyFrame>>, other: OCamlRef<DynBox<LazyFrame>>, left_on: OCamlRef<OCamlList<DynBox<Expr>>>, right_on: OCamlRef<OCamlList<DynBox<Expr>>>, how: OCamlRef<JoinType>) -> OCaml<DynBox<LazyFrame>> {
        let left_on = unwrap_abstract_vec(left_on.to_rust(cr));
        let right_on = unwrap_abstract_vec(right_on.to_rust(cr));
        let PolarsJoinType(how) = how.to_rust(cr);
        let Abstract(lazy_frame) = lazy_frame.to_rust(cr);
        let Abstract(other) = other.to_rust(cr);
        OCaml::box_value(cr, lazy_frame.join(other, &left_on, &right_on, how))
    }

    fn rust_lazy_frame_sort(cr, lazy_frame: OCamlRef<DynBox<LazyFrame>>, by_column: OCamlRef<String>, descending: OCamlRef<Option<bool>>, nulls_last: OCamlRef<Option<bool>>, multithreaded: OCamlRef<Option<bool>>) -> OCaml<DynBox<LazyFrame>> {
        let by_column: String = by_column.to_rust(cr);
        let descending: Option<bool> = descending.to_rust(cr);
        let nulls_last: Option<bool> = nulls_last.to_rust(cr);
        let multithreaded: Option<bool> = multithreaded.to_rust(cr);
        let sort_options: SortOptions = Default::default();
        let sort_options = SortOptions {
            descending: descending.unwrap_or(sort_options.descending),
            nulls_last: nulls_last.unwrap_or(sort_options.nulls_last),
            multithreaded: multithreaded.unwrap_or(sort_options.multithreaded),
        };

        let Abstract(lazy_frame) = lazy_frame.to_rust(cr);
        OCaml::box_value(cr, lazy_frame.sort(&by_column, sort_options))
    }

    fn rust_lazy_frame_vertical_concat(cr, lazy_frames: OCamlRef<OCamlList<DynBox<LazyFrame>>>, rechunk: OCamlRef<bool>, parallel: OCamlRef<bool>) -> OCaml<Result<DynBox<LazyFrame>,String>> {
        let lazy_frames = unwrap_abstract_vec(lazy_frames.to_rust(cr));
        let rechunk = rechunk.to_rust(cr);
        let parallel = parallel.to_rust(cr);
        concat(&lazy_frames, rechunk, parallel).map(Abstract).map_err(|err| err.to_string()).to_ocaml(cr)
    }

    fn rust_lazy_frame_diagonal_concat(cr, lazy_frames: OCamlRef<OCamlList<DynBox<LazyFrame>>>, rechunk: OCamlRef<bool>, parallel: OCamlRef<bool>) -> OCaml<Result<DynBox<LazyFrame>,String>> {
        let lazy_frames = unwrap_abstract_vec(lazy_frames.to_rust(cr));
        let rechunk = rechunk.to_rust(cr);
        let parallel = parallel.to_rust(cr);
        diag_concat_lf(&lazy_frames, rechunk, parallel).map(Abstract).map_err(|err| err.to_string()).to_ocaml(cr)
    }

    fn rust_lazy_frame_melt(cr,
        lazy_frame: OCamlRef<DynBox<LazyFrame>>,
        id_vars: OCamlRef<OCamlList<String>>,
        value_vars: OCamlRef<OCamlList<String>>,
        variable_name: OCamlRef<Option<String>>,
        value_name: OCamlRef<Option<String>>,
        streamable: OCamlRef<bool>,
    ) -> OCaml<DynBox<LazyFrame>> {
        let Abstract(lazy_frame) = lazy_frame.to_rust(cr);

        let id_vars: Vec<SmartString<LazyCompact>> = id_vars.to_rust::<Vec<String>>(cr).into_iter().map(|s| s.into()).collect();
        let value_vars: Vec<SmartString<LazyCompact>> = value_vars.to_rust::<Vec<String>>(cr).into_iter().map(|s| s.into()).collect();
        let variable_name: Option<SmartString<LazyCompact>> = variable_name.to_rust::<Option<String>>(cr).map(|s| s.into());
        let value_name: Option<SmartString<LazyCompact>> = value_name.to_rust::<Option<String>>(cr).map(|s| s.into());
        let streamable: bool = streamable.to_rust(cr);

        let melt_args = MeltArgs {
            id_vars, value_vars, variable_name, value_name, streamable
        };
        Abstract(lazy_frame.melt(melt_args)).to_ocaml(cr)
    }

    fn rust_lazy_frame_limit(cr, lazy_frame: OCamlRef<DynBox<LazyFrame>>, n: OCamlRef<OCamlInt>) -> OCaml<Option<DynBox<LazyFrame>>> {
        let result: Option<_> = try {
            let n = n.to_rust::<i64>(cr).try_into().ok()?;
            let Abstract(lazy_frame) = lazy_frame.to_rust(cr);
            Abstract(lazy_frame.limit(n))
        };
        result.to_ocaml(cr)
    }

    fn rust_lazy_frame_explode(cr, lazy_frame: OCamlRef<DynBox<LazyFrame>>, columns: OCamlRef<OCamlList<DynBox<Expr>>>) -> OCaml<DynBox<LazyFrame>> {
        let Abstract(lazy_frame) = lazy_frame.to_rust(cr);
        let columns = unwrap_abstract_vec(columns.to_rust(cr));

        Abstract(lazy_frame.explode(&columns)).to_ocaml(cr)
    }

    fn rust_lazy_frame_with_streaming(cr, lazy_frame: OCamlRef<DynBox<LazyFrame>>, toggle: OCamlRef<bool>) -> OCaml<DynBox<LazyFrame>> {
        let toggle = toggle.to_rust(cr);
        let Abstract(lazy_frame) = lazy_frame.to_rust(cr);
        OCaml::box_value(cr, lazy_frame.with_streaming(toggle))
    }

    fn rust_lazy_frame_schema(cr, lazy_frame: OCamlRef<DynBox<LazyFrame>>) -> OCaml<Result<DynBox<Schema>,String>> {
        let Abstract(lazy_frame) = lazy_frame.to_rust(cr);
        lazy_frame.schema()
        .map(|schema| Abstract((*schema).clone()))
        .map_err(|err| err.to_string()).to_ocaml(cr)
    }

}

#[no_mangle]
pub extern "C" fn rust_lazy_frame_melt_bytecode(
    argv: *const ocaml_interop::RawOCaml,
) -> ocaml_interop::RawOCaml {
    unsafe {
        rust_lazy_frame_melt(
            *argv.offset(0),
            *argv.offset(1),
            *argv.offset(2),
            *argv.offset(3),
            *argv.offset(4),
            *argv.offset(5),
        )
    }
}
