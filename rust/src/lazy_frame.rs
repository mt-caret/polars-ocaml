use crate::utils::*;
use ocaml_interop::{ocaml_export, DynBox, OCaml, OCamlBytes, OCamlList, OCamlRef, ToOCaml};
use polars::prelude::*;
use std::path::Path;

ocaml_export! {
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

}
