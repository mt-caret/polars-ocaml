use crate::interop::*;
use ocaml_interop::{DynBox, OCaml, OCamlList, OCamlRef, ToOCaml};
use polars::prelude::*;
use polars_ocaml_macros::ocaml_interop_export;
use polars_sql::SQLContext;
use std::cell::RefCell;
use std::rc::Rc;

type PolarsSQLContext = Rc<RefCell<SQLContext>>;

#[ocaml_interop_export]
fn rust_sql_context_new(
    cr: &mut &mut OCamlRuntime,
    unit: OCamlRef<()>,
) -> OCaml<DynBox<PolarsSQLContext>> {
    let () = unit.to_rust(cr);
    OCaml::box_value(cr, Rc::new(RefCell::new(SQLContext::new())))
}

#[ocaml_interop_export]
fn rust_sql_context_get_tables(
    cr: &mut &mut OCamlRuntime,
    sql_context: OCamlRef<DynBox<PolarsSQLContext>>,
) -> OCaml<OCamlList<String>> {
    let Abstract(sql_context) = sql_context.to_rust(cr);
    let tables = sql_context.borrow().get_tables();
    tables.to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_sql_context_register(
    cr: &mut &mut OCamlRuntime,
    sql_context: OCamlRef<DynBox<PolarsSQLContext>>,
    name: OCamlRef<String>,
    lf: OCamlRef<DynBox<LazyFrame>>,
) -> OCaml<()> {
    let Abstract(sql_context) = sql_context.to_rust(cr);
    let name: String = name.to_rust(cr);
    let Abstract(lf) = lf.to_rust(cr);

    sql_context.borrow_mut().register(&name, lf);

    OCaml::unit()
}

#[ocaml_interop_export]
fn rust_sql_context_execute_with_data_frames(
    cr: &mut &mut OCamlRuntime,
    names_and_data_frames: OCamlRef<
        OCamlList<(String, DynBox<crate::data_frame::PolarsDataFrame>)>,
    >,
    query: OCamlRef<String>,
) -> OCaml<Result<DynBox<crate::data_frame::PolarsDataFrame>, String>> {
    let names_and_data_frames: Vec<(String, Abstract<crate::data_frame::PolarsDataFrame>)> =
        names_and_data_frames.to_rust(cr);
    let query: String = query.to_rust(cr);

    let mut sql_context = SQLContext::new();
    for (name, Abstract(data_frame)) in names_and_data_frames {
        sql_context.register(&name, data_frame.borrow().clone().lazy());
    }

    cr.releasing_runtime(|| {
        sql_context
            .execute(&query)
            .and_then(|query_result| query_result.collect())
            .map(|df| Abstract(Rc::new(RefCell::new(df))))
            .map_err(|err| err.to_string())
    }).to_ocaml(cr)
}

fn sql_context_vstack_and_execute(
    query: String,
    names_and_data_frames: Vec<(String, Vec<Abstract<crate::data_frame::PolarsDataFrame>>)>,
) -> Result<LazyFrame, String> {
    let mut sql_context = SQLContext::new();
    for (name, data_frames) in names_and_data_frames {
        let data_frame = match data_frames
            .iter()
            .map(|Abstract(data_frame)| Ok(data_frame.borrow().clone()))
            .reduce(|accum, next| match (accum, next) {
                (Ok(accum), Ok(next)) => accum.vstack(&next),
                (Err(err), _) => Err(err),
                (_, Err(err)) => Err(err),
            }) {
            Some(ok) => ok.map_err(|err| err.to_string()),
            None => Err("Received an empty list of dataframes".to_string()),
        }?;
        sql_context.register(&name, data_frame.lazy());
    }

    sql_context
        .execute(&query)
        .map_err(|err| err.to_string())
}

#[ocaml_interop_export]
fn rust_sql_context_vstack_and_execute(
    cr: &mut &mut OCamlRuntime,
    names_and_data_frames: OCamlRef<
        OCamlList<(
            String,
            OCamlList<DynBox<crate::data_frame::PolarsDataFrame>>,
        )>,
    >,
    query: OCamlRef<String>,
) -> OCaml<Result<DynBox<LazyFrame>, String>> {
    let names_and_data_frames: Vec<(String, Vec<Abstract<crate::data_frame::PolarsDataFrame>>)> =
        names_and_data_frames.to_rust(cr);
    let query: String = query.to_rust(cr);

    sql_context_vstack_and_execute(query, names_and_data_frames)
        .map(|df| Abstract(df))
        .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_sql_context_vstack_and_collect(
    cr: &mut &mut OCamlRuntime,
    names_and_data_frames: OCamlRef<
        OCamlList<(
            String,
            OCamlList<DynBox<crate::data_frame::PolarsDataFrame>>,
        )>,
    >,
    query: OCamlRef<String>,
) -> OCaml<Result<DynBox<crate::data_frame::PolarsDataFrame>, String>> {
    let names_and_data_frames: Vec<(String, Vec<Abstract<crate::data_frame::PolarsDataFrame>>)> =
        names_and_data_frames.to_rust(cr);
    let query: String = query.to_rust(cr);

    cr.releasing_runtime(|| {
        sql_context_vstack_and_execute(query, names_and_data_frames)
            .and_then(|lazy_frame| lazy_frame.collect().map_err(|err| err.to_string()))
            .map(|df| Abstract(Rc::new(RefCell::new(df))))
    }).to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_sql_context_unregister(
    cr: &mut &mut OCamlRuntime,
    sql_context: OCamlRef<DynBox<PolarsSQLContext>>,
    name: OCamlRef<String>,
) -> OCaml<()> {
    let Abstract(sql_context) = sql_context.to_rust(cr);
    let name: String = name.to_rust(cr);

    sql_context.borrow_mut().unregister(&name);

    OCaml::unit()
}

#[ocaml_interop_export]
fn rust_sql_context_execute(
    cr: &mut &mut OCamlRuntime,
    sql_context: OCamlRef<DynBox<PolarsSQLContext>>,
    query: OCamlRef<String>,
) -> OCaml<Result<DynBox<LazyFrame>, String>> {
    let query: String = query.to_rust(cr);

    dyn_box_result_with_cr(cr, sql_context, |cr, sql_context| {
        cr.releasing_runtime(|| {
            let result = sql_context.borrow_mut().execute(&query);
            result
        })
    })
}
