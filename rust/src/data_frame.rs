use crate::utils::*;
use ocaml_interop::{
    ocaml_export, DynBox, OCaml, OCamlFloat, OCamlInt, OCamlList, OCamlRef, ToOCaml,
};
use polars::prelude::*;
use smartstring::{LazyCompact, SmartString};

ocaml_export! {
    fn rust_data_frame_new(cr, series: OCamlRef<OCamlList<DynBox<Series>>>) -> OCaml<Result<DynBox<DataFrame>,String>> {
        let series: Vec<Series> = unwrap_abstract_vec(series.to_rust(cr));

        DataFrame::new(series).map(Abstract).map_err(|err| err.to_string()).to_ocaml(cr)
    }

    fn rust_data_frame_read_csv(cr, path: OCamlRef<String>, schema: OCamlRef<Option<DynBox<Schema>>>, try_parse_dates: OCamlRef<Option<bool>>) -> OCaml<Result<DynBox<DataFrame>,String>> {
        let path: String = path.to_rust(cr);
        let schema = schema.to_rust::<Option<Abstract<Schema>>>(cr).map(|Abstract(schema)| Arc::new(schema));
        let try_parse_dates: Option<bool> = try_parse_dates.to_rust(cr);

        CsvReader::from_path(&path)
        .and_then(|csv_reader| {
            let csv_reader = csv_reader.with_dtypes(schema);
            match try_parse_dates {
                None => csv_reader,
                Some(try_parse_dates) => csv_reader.with_try_parse_dates(try_parse_dates),
            }.finish()
        })
        .map(Abstract).map_err(|err| err.to_string()).to_ocaml(cr)
    }

   fn rust_data_frame_describe(cr, data_frame: OCamlRef<DynBox<DataFrame>>, percentiles: OCamlRef<Option<OCamlList<OCamlFloat>>>) -> OCaml<Result<DynBox<DataFrame>,String>> {
        let Abstract(data_frame) = data_frame.to_rust(cr);
        let percentiles: Option<Vec<f64>> = percentiles.to_rust(cr);

        // TODO: I'm not sure why I can't do this with something like
        // .map(|percentiles| percentiles.as_slice()
        match percentiles {
            None => data_frame.describe(None),
            Some(percentiles) => data_frame.describe(Some(percentiles.as_slice()))
        }
        .map(Abstract).map_err(|err| err.to_string()).to_ocaml(cr)
    }

   fn rust_data_frame_lazy(cr, data_frame: OCamlRef<DynBox<DataFrame>>) -> OCaml<DynBox<LazyFrame>> {
        let Abstract(data_frame) = data_frame.to_rust(cr);
        OCaml::box_value(cr, data_frame.lazy())
    }

    fn rust_data_frame_column(cr, data_frame: OCamlRef<DynBox<DataFrame>>, name: OCamlRef<String>) -> OCaml<Result<DynBox<Series>,String>> {
        let Abstract(data_frame) = data_frame.to_rust(cr);
        let name: String = name.to_rust(cr);
        data_frame.column(&name)
        .map(|series| Abstract(series.clone()))
        .map_err(|err| err.to_string())
        .to_ocaml(cr)
    }

    fn rust_data_frame_columns(cr, data_frame: OCamlRef<DynBox<DataFrame>>, names: OCamlRef<OCamlList<String>>) -> OCaml<Result<OCamlList<DynBox<Series>>,String>> {
        let Abstract(data_frame) = data_frame.to_rust(cr);
        let names: Vec<String> = names.to_rust(cr);
        data_frame.columns(&names)
        .map(|series|
            series
            .into_iter()
            .map(|series| Abstract(series.clone()))
            .collect::<Vec<Abstract<_>>>())
        .map_err(|err| err.to_string())
        .to_ocaml(cr)
    }

    fn rust_data_frame_vertical_concat(cr, data_frames: OCamlRef<OCamlList<DynBox<DataFrame>>>) -> OCaml<Result<DynBox<DataFrame>,String>> {
        let data_frames = unwrap_abstract_vec(data_frames.to_rust(cr));

        let result: PolarsResult<_> = try {
            let mut data_frames = data_frames.into_iter();
            let first = data_frames.next().ok_or(PolarsError::NoData("No dataframes provided for vertical concatenation".into()))?;
            let mut result = first.clone();
            for data_frame in data_frames {
                result = result.vstack(&data_frame)?;
            }
            result
        };
        result.map(Abstract).map_err(|err| err.to_string()).to_ocaml(cr)
    }

    fn rust_data_frame_horizontal_concat(cr, data_frames: OCamlRef<OCamlList<DynBox<DataFrame>>>) -> OCaml<Result<DynBox<DataFrame>,String>> {
        let data_frames = unwrap_abstract_vec(data_frames.to_rust(cr));

        polars::functions::hor_concat_df(&data_frames).map(Abstract).map_err(|err| err.to_string()).to_ocaml(cr)
    }

    fn rust_data_frame_diagonal_concat(cr, data_frames: OCamlRef<OCamlList<DynBox<DataFrame>>>) -> OCaml<Result<DynBox<DataFrame>,String>> {
        let data_frames = unwrap_abstract_vec(data_frames.to_rust(cr));

        polars::functions::diag_concat_df(&data_frames).map(Abstract).map_err(|err| err.to_string()).to_ocaml(cr)
    }

    fn rust_data_frame_pivot(cr,
        data_frame: OCamlRef<DynBox<DataFrame>>,
        values: OCamlRef<OCamlList<String>>,
        index: OCamlRef<OCamlList<String>>,
        columns: OCamlRef<OCamlList<String>>,
        sort_columns: OCamlRef<bool>,
        agg_expr: OCamlRef<Option<DynBox<Expr>>>,
        separator: OCamlRef<Option<String>>,
        stable: OCamlRef<bool>
    )
    -> OCaml<Result<DynBox<DataFrame>,String>> {
        let Abstract(data_frame) = data_frame.to_rust(cr);

        let values: Vec<String> = values.to_rust(cr);
        let index: Vec<String> = index.to_rust(cr);
        let columns: Vec<String> = columns.to_rust(cr);
        let sort_columns: bool = sort_columns.to_rust(cr);
        let agg_expr: Option<Expr> = agg_expr.to_rust::<Option<Abstract<Expr>>>(cr).map(|Abstract(expr)| expr);
        let separator: Option<String> = separator.to_rust(cr);

        let stable: bool = stable.to_rust(cr);

        if stable {
            pivot::pivot_stable(&data_frame, &values, &index, &columns, sort_columns, agg_expr, separator.as_deref())
        } else {
            pivot::pivot(&data_frame, &values, &index, &columns, sort_columns, agg_expr, separator.as_deref())
        }.map(Abstract).map_err(|err| err.to_string()).to_ocaml(cr)
    }

    fn rust_data_frame_melt(cr,
        data_frame: OCamlRef<DynBox<DataFrame>>,
        id_vars: OCamlRef<OCamlList<String>>,
        value_vars: OCamlRef<OCamlList<String>>,
        variable_name: OCamlRef<Option<String>>,
        value_name: OCamlRef<Option<String>>,
        streamable: OCamlRef<bool>,
    ) -> OCaml<Result<DynBox<DataFrame>,String>> {
        let Abstract(data_frame) = data_frame.to_rust(cr);

        let id_vars: Vec<SmartString<LazyCompact>> = id_vars.to_rust::<Vec<String>>(cr).into_iter().map(|s| s.into()).collect();
        let value_vars: Vec<SmartString<LazyCompact>> = value_vars.to_rust::<Vec<String>>(cr).into_iter().map(|s| s.into()).collect();
        let variable_name: Option<SmartString<LazyCompact>> = variable_name.to_rust::<Option<String>>(cr).map(|s| s.into());
        let value_name: Option<SmartString<LazyCompact>> = value_name.to_rust::<Option<String>>(cr).map(|s| s.into());
        let streamable: bool = streamable.to_rust(cr);

        let melt_args = MeltArgs {
            id_vars, value_vars, variable_name, value_name, streamable
        };
        data_frame.melt2(melt_args).map(Abstract).map_err(|err| err.to_string()).to_ocaml(cr)
    }

    fn rust_data_frame_head(cr, data_frame: OCamlRef<DynBox<DataFrame>>, length: OCamlRef<Option<OCamlInt>>) -> OCaml<Option<DynBox<DataFrame>>> {
        let Abstract(data_frame) = data_frame.to_rust(cr);
        let length: Option<i64> = length.to_rust(cr);

        match length.map(|length| length.try_into().ok()) {
            None => Some(Abstract(data_frame.head(None))),
            Some(None) => None,
            Some(Some(length)) => Some(Abstract(data_frame.head(Some(length)))),
        }.to_ocaml(cr)
    }

    fn rust_data_frame_tail(cr, data_frame: OCamlRef<DynBox<DataFrame>>, length: OCamlRef<Option<OCamlInt>>) -> OCaml<Option<DynBox<DataFrame>>> {
        let Abstract(data_frame) = data_frame.to_rust(cr);
        let length: Option<i64> = length.to_rust(cr);

        match length.map(|length| length.try_into().ok()) {
            None => Some(Abstract(data_frame.tail(None))),
            Some(None) => None,
            Some(Some(length)) => Some(Abstract(data_frame.tail(Some(length)))),
        }.to_ocaml(cr)
    }

    fn rust_data_frame_sample_n(cr, data_frame: OCamlRef<DynBox<DataFrame>>, n: OCamlRef<OCamlInt>, with_replacement: OCamlRef<bool>, shuffle: OCamlRef<bool>, seed: OCamlRef<Option<OCamlInt>>) -> OCaml<Option<Result<DynBox<DataFrame>,String>>> {
        let result: Option<_> = try {
            let Abstract(data_frame) = data_frame.to_rust(cr);
            let n: usize = n.to_rust::<i64>(cr).try_into().ok()?;
            let with_replacement: bool = with_replacement.to_rust(cr);
            let shuffle: bool = shuffle.to_rust(cr);
            let seed: Option<Result<u64,_>> = seed.to_rust::<Option<i64>>(cr).map(|seed| seed.try_into());
            let seed: Option<u64> = seed.map_or(Ok(None), |seed| seed.map(Some)).ok()?;

            data_frame.sample_n(n, with_replacement, shuffle, seed)
            .map(Abstract).map_err(|err| err.to_string())
        };
        result.to_ocaml(cr)
    }

    fn rust_data_frame_sum(cr, data_frame: OCamlRef<DynBox<DataFrame>>) -> OCaml<DynBox<DataFrame>> {
        let Abstract(data_frame) = data_frame.to_rust(cr);
        Abstract(data_frame.sum()).to_ocaml(cr)
    }

    fn rust_data_frame_mean(cr, data_frame: OCamlRef<DynBox<DataFrame>>) -> OCaml<DynBox<DataFrame>> {
        let Abstract(data_frame) = data_frame.to_rust(cr);
        Abstract(data_frame.mean()).to_ocaml(cr)
    }

    fn rust_data_frame_median(cr, data_frame: OCamlRef<DynBox<DataFrame>>) -> OCaml<DynBox<DataFrame>> {
        let Abstract(data_frame) = data_frame.to_rust(cr);
        Abstract(data_frame.median()).to_ocaml(cr)
    }

    fn rust_data_frame_null_count(cr, data_frame: OCamlRef<DynBox<DataFrame>>) -> OCaml<DynBox<DataFrame>> {
        let Abstract(data_frame) = data_frame.to_rust(cr);
        Abstract(data_frame.null_count()).to_ocaml(cr)
    }

    fn rust_data_frame_explode(cr, data_frame: OCamlRef<DynBox<DataFrame>>, columns: OCamlRef<OCamlList<String>>) -> OCaml<Result<DynBox<DataFrame>, String>> {
        let Abstract(data_frame) = data_frame.to_rust(cr);
        let columns: Vec<String> = columns.to_rust(cr);

        data_frame.explode(columns)
        .map(Abstract)
        .map_err(|err| err.to_string())
        .to_ocaml(cr)
    }

    fn rust_data_frame_schema(cr, data_frame: OCamlRef<DynBox<DataFrame>>) -> OCaml<DynBox<Schema>> {
        let Abstract(data_frame) = data_frame.to_rust(cr);
        OCaml::box_value(cr, data_frame.schema())
    }

    fn rust_data_frame_to_string_hum(cr, data_frame: OCamlRef<DynBox<DataFrame>>) -> OCaml<String> {
        let Abstract(data_frame) = data_frame.to_rust(cr);
        data_frame.to_string().to_ocaml(cr)
    }
}

#[no_mangle]
pub extern "C" fn rust_data_frame_pivot_bytecode(
    argv: *const ocaml_interop::RawOCaml,
) -> ocaml_interop::RawOCaml {
    unsafe {
        rust_data_frame_pivot(
            *argv.offset(0),
            *argv.offset(1),
            *argv.offset(2),
            *argv.offset(3),
            *argv.offset(4),
            *argv.offset(5),
            *argv.offset(6),
            *argv.offset(7),
        )
    }
}

#[no_mangle]
pub extern "C" fn rust_data_frame_melt_bytecode(
    argv: *const ocaml_interop::RawOCaml,
) -> ocaml_interop::RawOCaml {
    unsafe {
        rust_data_frame_melt(
            *argv.offset(0),
            *argv.offset(1),
            *argv.offset(2),
            *argv.offset(3),
            *argv.offset(4),
            *argv.offset(5),
        )
    }
}
