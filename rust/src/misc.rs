use crate::utils::PolarsDataType;
use crate::utils::*;
use chrono::naive::{NaiveDate, NaiveDateTime};
use ocaml_interop::{ocaml_export, DynBox, OCaml, OCamlInt, OCamlList, OCamlRef, ToOCaml};
use polars::prelude::*;

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
