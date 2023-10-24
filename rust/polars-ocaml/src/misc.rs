use crate::utils::PolarsDataType;
use crate::utils::*;
use ocaml_interop::{DynBox, OCaml, OCamlList, OCamlRef, ToOCaml};
use polars::prelude::*;
use polars_ocaml_macros::ocaml_interop_export;

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
