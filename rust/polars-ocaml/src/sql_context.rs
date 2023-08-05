use crate::utils::*;
use ocaml_interop::{ocaml_export, DynBox, OCaml, OCamlList, OCamlRef, ToOCaml};
use polars::prelude::*;
use polars_sql::SQLContext;
use std::cell::RefCell;
use std::rc::Rc;

type PolarsSQLContext = Rc<RefCell<SQLContext>>;

ocaml_export! {
    fn rust_sql_context_new(
        cr,
        unit: OCamlRef<()>,
    ) -> OCaml<DynBox<PolarsSQLContext>> {
        let () = unit.to_rust(cr);
        OCaml::box_value(cr, Rc::new(RefCell::new(SQLContext::new())))
    }

    fn rust_sql_context_get_tables(
        cr,
        sql_context: OCamlRef<DynBox<PolarsSQLContext>>,
    ) -> OCaml<OCamlList<String>> {
        let Abstract(sql_context) = sql_context.to_rust(cr);
        let tables = sql_context.borrow().get_tables();
        tables.to_ocaml(cr)
    }

    fn rust_sql_context_register(
        cr,
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

    fn rust_sql_context_unregister(
        cr,
        sql_context: OCamlRef<DynBox<PolarsSQLContext>>,
        name: OCamlRef<String>,
    ) -> OCaml<()> {
        let Abstract(sql_context) = sql_context.to_rust(cr);
        let name: String = name.to_rust(cr);

        sql_context.borrow_mut().unregister(&name);

        OCaml::unit()
    }

    fn rust_sql_context_execute(
        cr,
        sql_context: OCamlRef<DynBox<PolarsSQLContext>>,
        query: OCamlRef<String>,
    ) -> OCaml<Result<DynBox<LazyFrame>,String>> {
        let Abstract(sql_context) = sql_context.to_rust(cr);
        let query: String = query.to_rust(cr);

        let result = sql_context.borrow_mut().execute(&query);

        result.map(Abstract).map_err(|err| err.to_string())
        .to_ocaml(cr)
    }
}
