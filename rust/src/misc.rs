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

    fn rust_test_panic(cr, error_message: OCamlRef<String>) -> OCaml<()> {
        let error_message: String = error_message.to_rust(cr);
        // TODO: quite hacky, but works for now
        if true {
            panic!("{}", error_message)
        }
        OCaml::unit()
    }

    fn rust_test_exception(cr, error_message: OCamlRef<String>) -> OCaml<()> {
        let error_message: String = error_message.to_rust(cr);
        let error_message =
            std::ffi::CString::new(error_message).expect("CString::new failed");
        unsafe {
            ocaml_sys::caml_failwith(error_message.as_ptr());
        }

        OCaml::unit()
    }

    fn rust_install_panic_hook(cr, suppress_backtrace: OCamlRef<bool>) -> OCaml<()> {
        let suppress_backtrace = suppress_backtrace.to_rust(cr);
        std::panic::set_hook(Box::new(move |panic_info| {
            let payload = panic_info.payload();
            let message = if let Some(s) = payload.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = payload.downcast_ref::<String>() {
                s.to_string()
            } else {
                "Box<Any>".to_string()
            };

            let message =
                if suppress_backtrace {
                    format!("Rust panic: {}", message)
                } else {
                    let backtrace = std::backtrace::Backtrace::force_capture();
                    format!("Rust panic: {}\nbacktrace:\n{}", message, backtrace.to_string())
                };
            let message =
                std::ffi::CString::new(message).expect("CString::new failed");

            unsafe {
                ocaml_sys::caml_failwith(message.as_ptr());
            }
        }));
        OCaml::unit()
    }
}
