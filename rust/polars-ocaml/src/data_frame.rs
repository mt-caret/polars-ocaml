use crate::interop::*;
use crate::polars_types::*;
use ocaml_interop::{
    DynBox, OCaml, OCamlFloat, OCamlInt, OCamlList, OCamlRef, OCamlRuntime, ToOCaml,
};
use polars::prelude::*;
use polars_ocaml_macros::ocaml_interop_export;
use smartstring::{LazyCompact, SmartString};
use std::cell::RefCell;
use std::fs::File;
use std::rc::Rc;

use crate::series::PolarsSeries;

pub type PolarsDataFrame = Rc<RefCell<DataFrame>>;

#[ocaml_interop_export]
fn rust_data_frame_new(
    cr: &mut &mut OCamlRuntime,
    series: OCamlRef<OCamlList<DynBox<PolarsSeries>>>,
) -> OCaml<Result<DynBox<PolarsDataFrame>, String>> {
    let series: Vec<PolarsSeries> = unwrap_abstract_vec(series.to_rust(cr));
    let series: Vec<Series> = series.into_iter().map(|s| s.borrow().clone()).collect();

    DataFrame::new(series)
        .map(|df| Abstract(Rc::new(RefCell::new(df))))
        .map_err(|err| err.to_string())
        .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_data_frame_read_csv(
    cr: &mut &mut OCamlRuntime,
    path: OCamlRef<String>,
    schema: OCamlRef<Option<DynBox<Schema>>>,
    try_parse_dates: OCamlRef<Option<bool>>,
) -> OCaml<Result<DynBox<PolarsDataFrame>, String>> {
    let path: String = path.to_rust(cr);
    let schema = schema
        .to_rust::<Option<Abstract<Schema>>>(cr)
        .map(|Abstract(schema)| Arc::new(schema));
    let try_parse_dates: Option<bool> = try_parse_dates.to_rust(cr);

    CsvReader::from_path(path)
        .and_then(|csv_reader| {
            let csv_reader = csv_reader.with_dtypes(schema);
            match try_parse_dates {
                None => csv_reader,
                Some(try_parse_dates) => csv_reader.with_try_parse_dates(try_parse_dates),
            }
            .finish()
        })
        .map(|df| Abstract(Rc::new(RefCell::new(df))))
        .map_err(|err| err.to_string())
        .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_data_frame_write_csv(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
    path: OCamlRef<String>,
) -> OCaml<Result<(), String>> {
    let Abstract(data_frame) = data_frame.to_rust(cr);
    let path: String = path.to_rust(cr);

    File::create(path)
        .map_err(|err| err.to_string())
        .and_then(|file| {
            CsvWriter::new(&file)
                .finish(&mut data_frame.borrow_mut())
                .map_err(|err| err.to_string())
        })
        .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_data_frame_read_parquet(
    cr: &mut &mut OCamlRuntime,
    path: OCamlRef<String>,
) -> OCaml<Result<DynBox<PolarsDataFrame>, String>> {
    let path: String = path.to_rust(cr);

    File::open(path)
        .map_err(|err| err.to_string())
        .and_then(|file| {
            ParquetReader::new(file)
                .finish()
                .map_err(|err| err.to_string())
        })
        .map(|df| Abstract(Rc::new(RefCell::new(df))))
        .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_data_frame_write_parquet(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
    path: OCamlRef<String>,
) -> OCaml<Result<(), String>> {
    let Abstract(data_frame) = data_frame.to_rust(cr);
    let path: String = path.to_rust(cr);

    File::create(path)
        .map_err(|err| err.to_string())
        .and_then(|file| {
            ParquetWriter::new(file)
                .finish(&mut data_frame.borrow_mut())
                .map(|_file_size_in_bytes| ())
                .map_err(|err| err.to_string())
        })
        .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_data_frame_read_json(
    cr: &mut &mut OCamlRuntime,
    path: OCamlRef<String>,
) -> OCaml<Result<DynBox<PolarsDataFrame>, String>> {
    let path: String = path.to_rust(cr);

    File::open(path)
        .map_err(|err| err.to_string())
        .and_then(|file| {
            JsonReader::new(file)
                .finish()
                .map_err(|err| err.to_string())
        })
        .map(|df| Abstract(Rc::new(RefCell::new(df))))
        .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_data_frame_write_json(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
    path: OCamlRef<String>,
) -> OCaml<Result<(), String>> {
    let Abstract(data_frame) = data_frame.to_rust(cr);
    let path: String = path.to_rust(cr);

    File::create(path)
        .map_err(|err| err.to_string())
        .and_then(|file| {
            JsonWriter::new(file)
                .with_json_format(JsonFormat::Json)
                .finish(&mut data_frame.borrow_mut())
                .map_err(|err| err.to_string())
        })
        .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_data_frame_read_jsonl(
    cr: &mut &mut OCamlRuntime,
    path: OCamlRef<String>,
) -> OCaml<Result<DynBox<PolarsDataFrame>, String>> {
    let path: String = path.to_rust(cr);

    File::open(path)
        .map_err(|err| err.to_string())
        .and_then(|file| {
            JsonLineReader::new(file)
                .finish()
                .map_err(|err| err.to_string())
        })
        .map(|df| Abstract(Rc::new(RefCell::new(df))))
        .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_data_frame_write_jsonl(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
    path: OCamlRef<String>,
) -> OCaml<Result<(), String>> {
    let Abstract(data_frame) = data_frame.to_rust(cr);
    let path: String = path.to_rust(cr);

    File::create(path)
        .map_err(|err| err.to_string())
        .and_then(|file| {
            JsonWriter::new(file)
                .with_json_format(JsonFormat::JsonLines)
                .finish(&mut data_frame.borrow_mut())
                .map_err(|err| err.to_string())
        })
        .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_data_frame_clear(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
) -> OCaml<DynBox<PolarsDataFrame>> {
    dyn_box(cr, data_frame, |data_frame| {
        let data_frame = data_frame.borrow();
        Rc::new(RefCell::new(data_frame.clear()))
    })
}

#[ocaml_interop_export]
fn rust_data_frame_describe(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
    percentiles: OCamlRef<Option<OCamlList<OCamlFloat>>>,
) -> OCaml<Result<DynBox<PolarsDataFrame>, String>> {
    let Abstract(data_frame) = data_frame.to_rust(cr);
    let data_frame = data_frame.borrow();
    let percentiles: Option<Vec<f64>> = percentiles.to_rust(cr);

    // TODO: I'm not sure why I can't do this with something like
    // .map(|percentiles| percentiles.as_slice()
    match percentiles {
        None => data_frame.describe(None),
        Some(percentiles) => data_frame.describe(Some(percentiles.as_slice())),
    }
    .map(|df| Abstract(Rc::new(RefCell::new(df))))
    .map_err(|err| err.to_string())
    .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_data_frame_height(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
) -> OCaml<OCamlInt> {
    let Abstract(data_frame) = data_frame.to_rust(cr);
    let height = data_frame.borrow().height() as i64;
    height.to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_data_frame_estimated_size(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
) -> OCaml<OCamlInt> {
    let Abstract(data_frame) = data_frame.to_rust(cr);
    let estimated_size = data_frame.borrow().estimated_size() as i64;
    estimated_size.to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_data_frame_lazy(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
) -> OCaml<DynBox<LazyFrame>> {
    dyn_box(cr, data_frame, |data_frame| {
        match Rc::try_unwrap(data_frame) {
            Ok(data_frame) => data_frame.into_inner().lazy(),
            Err(data_frame) => data_frame.borrow().clone().lazy(),
        }
    })
}

#[ocaml_interop_export]
fn rust_data_frame_column(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
    name: OCamlRef<String>,
) -> OCaml<Result<DynBox<PolarsSeries>, String>> {
    let name: String = name.to_rust(cr);

    dyn_box_result(cr, data_frame, |data_frame| {
        let data_frame = data_frame.borrow();
        data_frame
            .column(&name)
            .cloned()
            .map(|s| Rc::new(RefCell::new(s)))
    })
}

#[ocaml_interop_export]
fn rust_data_frame_columns(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
    names: OCamlRef<OCamlList<String>>,
) -> OCaml<Result<OCamlList<DynBox<PolarsSeries>>, String>> {
    let Abstract(data_frame) = data_frame.to_rust(cr);
    let names: Vec<String> = names.to_rust(cr);
    let data_frame = data_frame.borrow();
    data_frame
        .columns(&names)
        .map(|series| {
            series
                .into_iter()
                .map(|series| Abstract(Rc::new(RefCell::new(series.clone()))))
                .collect::<Vec<Abstract<_>>>()
        })
        .map_err(|err| err.to_string())
        .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_data_frame_get_column_names(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
) -> OCaml<OCamlList<String>> {
    let Abstract(data_frame) = data_frame.to_rust(cr);
    let data_frame = data_frame.borrow();
    data_frame.get_column_names().to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_data_frame_vertical_concat(
    cr: &mut &mut OCamlRuntime,
    data_frames: OCamlRef<OCamlList<DynBox<PolarsDataFrame>>>,
) -> OCaml<Result<DynBox<PolarsDataFrame>, String>> {
    let data_frames = unwrap_abstract_vec(data_frames.to_rust(cr));

    let stack = || {
        let mut data_frames = data_frames.into_iter();
        let mut result = data_frames.next().ok_or(PolarsError::NoData(
            "No dataframes provided for vertical concatenation".into(),
        ))?;
        for data_frame in data_frames {
            // TODO: there has to be a more elegant way to do this that doesn't
            // involve creating a new Rc<RefCell<_>> on every iteration!
            result = Rc::new(RefCell::new({
                let borrowed_result = result.borrow();
                borrowed_result.vstack(&data_frame.borrow())
            }?));
        }
        Ok(Abstract(result))
    };

    stack()
        .map_err(|err: PolarsError| err.to_string())
        .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_data_frame_horizontal_concat(
    cr: &mut &mut OCamlRuntime,
    data_frames: OCamlRef<OCamlList<DynBox<PolarsDataFrame>>>,
) -> OCaml<Result<DynBox<PolarsDataFrame>, String>> {
    let data_frames: Vec<DataFrame> = unwrap_abstract_vec(data_frames.to_rust(cr))
        // TODO: This clone is probably avoidable
        .into_iter()
        .map(|df| df.borrow().clone())
        .collect();

    polars::functions::hor_concat_df(&data_frames)
        .map(|df| Abstract(Rc::new(RefCell::new(df))))
        .map_err(|err| err.to_string())
        .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_data_frame_diagonal_concat(
    cr: &mut &mut OCamlRuntime,
    data_frames: OCamlRef<OCamlList<DynBox<PolarsDataFrame>>>,
) -> OCaml<Result<DynBox<PolarsDataFrame>, String>> {
    let data_frames: Vec<DataFrame> = unwrap_abstract_vec(data_frames.to_rust(cr))
        // TODO: This clone is probably avoidable
        .into_iter()
        .map(|df| df.borrow().clone())
        .collect();

    polars::functions::diag_concat_df(&data_frames)
        .map(|df| Abstract(Rc::new(RefCell::new(df))))
        .map_err(|err| err.to_string())
        .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_data_frame_vstack(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
    other: OCamlRef<DynBox<PolarsDataFrame>>,
) -> OCaml<Result<DynBox<()>, String>> {
    dyn_box_result2(cr, data_frame, other, |data_frame, other| {
        let other = match Rc::try_unwrap(other) {
            Ok(data_frame) => data_frame.into_inner(),
            Err(data_frame) => data_frame.borrow().clone(),
        };

        let mut data_frame = data_frame.borrow_mut();
        data_frame
            .vstack_mut(&other)
            .map(|_| ())
            .map_err(|err| err.to_string())
    })
}

#[ocaml_interop_export]
fn rust_data_frame_as_single_chunk_par(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
) -> OCaml<()> {
    let Abstract(data_frame) = data_frame.to_rust(cr);
    data_frame.borrow_mut().as_single_chunk_par();
    OCaml::unit()
}

#[ocaml_interop_export]
fn rust_data_frame_pivot(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
    values: OCamlRef<OCamlList<String>>,
    index: OCamlRef<OCamlList<String>>,
    columns: OCamlRef<OCamlList<String>>,
    sort_columns: OCamlRef<bool>,
    agg_expr: OCamlRef<Option<DynBox<Expr>>>,
    separator: OCamlRef<Option<String>>,
    stable: OCamlRef<bool>,
) -> OCaml<Result<DynBox<PolarsDataFrame>, String>> {
    let values: Vec<String> = values.to_rust(cr);
    let index: Vec<String> = index.to_rust(cr);
    let columns: Vec<String> = columns.to_rust(cr);
    let sort_columns: bool = sort_columns.to_rust(cr);
    let agg_expr: Option<Expr> = agg_expr
        .to_rust::<Option<Abstract<Expr>>>(cr)
        .map(|Abstract(expr)| expr);
    let separator: Option<String> = separator.to_rust(cr);

    let stable: bool = stable.to_rust(cr);

    dyn_box_result(cr, data_frame, |data_frame| {
        let result = if stable {
            pivot::pivot_stable(
                &data_frame.borrow(),
                &values,
                &index,
                &columns,
                sort_columns,
                agg_expr,
                separator.as_deref(),
            )
        } else {
            pivot::pivot(
                &data_frame.borrow(),
                &values,
                &index,
                &columns,
                sort_columns,
                agg_expr,
                separator.as_deref(),
            )
        };

        result.map(|df| Rc::new(RefCell::new(df)))
    })
}

#[ocaml_interop_export]
fn rust_data_frame_melt(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
    id_vars: OCamlRef<OCamlList<String>>,
    value_vars: OCamlRef<OCamlList<String>>,
    variable_name: OCamlRef<Option<String>>,
    value_name: OCamlRef<Option<String>>,
    streamable: OCamlRef<bool>,
) -> OCaml<Result<DynBox<PolarsDataFrame>, String>> {
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

    dyn_box_result(cr, data_frame, |data_frame| {
        let data_frame = data_frame.borrow();
        data_frame
            .melt2(melt_args)
            .map(|df| Rc::new(RefCell::new(df)))
    })
}

#[ocaml_interop_export]
fn rust_data_frame_sort(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
    by_column: OCamlRef<OCamlList<String>>,
    descending: OCamlRef<OCamlList<bool>>,
    maintain_order: OCamlRef<bool>,
) -> OCaml<Result<DynBox<PolarsDataFrame>, String>> {
    let by_column: Vec<String> = by_column.to_rust(cr);
    let descending: Vec<bool> = descending.to_rust(cr);
    let maintain_order: bool = maintain_order.to_rust(cr);

    dyn_box_result(cr, data_frame, |data_frame| {
        let data_frame = data_frame.borrow();
        data_frame
            .sort(by_column, descending, maintain_order)
            .map(|df| Rc::new(RefCell::new(df)))
    })
}

#[ocaml_interop_export(raise_on_err)]
fn rust_data_frame_head(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
    length: OCamlRef<Option<OCamlInt>>,
) -> OCaml<DynBox<PolarsDataFrame>> {
    let length = length
        .to_rust::<Coerce<_, Option<i64>, Option<usize>>>(cr)
        .get()?;

    dyn_box(cr, data_frame, |data_frame| {
        let data_frame = data_frame.borrow();
        Rc::new(RefCell::new(data_frame.head(length)))
    })
}

#[ocaml_interop_export(raise_on_err)]
fn rust_data_frame_tail(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
    length: OCamlRef<Option<OCamlInt>>,
) -> OCaml<DynBox<PolarsDataFrame>> {
    let length = length
        .to_rust::<Coerce<_, Option<i64>, Option<usize>>>(cr)
        .get()?;

    dyn_box(cr, data_frame, |data_frame| {
        let data_frame = data_frame.borrow();
        Rc::new(RefCell::new(data_frame.tail(length)))
    })
}

#[ocaml_interop_export(raise_on_err)]
fn rust_data_frame_sample_n(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
    n: OCamlRef<OCamlInt>,
    with_replacement: OCamlRef<bool>,
    shuffle: OCamlRef<bool>,
    seed: OCamlRef<Option<OCamlInt>>,
) -> OCaml<Result<DynBox<PolarsDataFrame>, String>> {
    let n = n.to_rust::<Coerce<_, i64, usize>>(cr).get()?;
    let with_replacement: bool = with_replacement.to_rust(cr);
    let shuffle: bool = shuffle.to_rust(cr);
    let seed: Option<u64> = seed
        .to_rust::<Coerce<_, Option<i64>, Option<u64>>>(cr)
        .get()?;

    dyn_box_result(cr, data_frame, |data_frame| {
        let data_frame = data_frame.borrow();
        data_frame
            .sample_n(n, with_replacement, shuffle, seed)
            .map(|df| Rc::new(RefCell::new(df)))
    })
}

dyn_box_op!(rust_data_frame_sum, PolarsDataFrame, |data_frame| {
    let data_frame = data_frame.borrow();
    Rc::new(RefCell::new(data_frame.sum()))
});
dyn_box_op!(rust_data_frame_mean, PolarsDataFrame, |data_frame| {
    let data_frame = data_frame.borrow();
    Rc::new(RefCell::new(data_frame.mean()))
});
dyn_box_op!(rust_data_frame_median, PolarsDataFrame, |data_frame| {
    let data_frame = data_frame.borrow();
    Rc::new(RefCell::new(data_frame.median()))
});
// TODO: mode is missing for dataframes
dyn_box_op!(rust_data_frame_null_count, PolarsDataFrame, |data_frame| {
    let data_frame = data_frame.borrow();
    Rc::new(RefCell::new(data_frame.null_count()))
});

#[ocaml_interop_export]
fn rust_data_frame_fill_null_with_strategy(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
    strategy: OCamlRef<FillNullStrategy>,
) -> OCaml<Result<DynBox<PolarsDataFrame>, String>> {
    let PolarsFillNullStrategy(strategy) = strategy.to_rust(cr);

    dyn_box_result(cr, data_frame, |data_frame| {
        let data_frame = data_frame.borrow();
        data_frame
            .fill_null(strategy)
            .map(|df| Rc::new(RefCell::new(df)))
    })
}

#[ocaml_interop_export]
fn rust_data_frame_interpolate(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
    method: OCamlRef<InterpolationMethod>,
) -> OCaml<Result<DynBox<PolarsDataFrame>, String>> {
    let PolarsInterpolationMethod(method) = method.to_rust(cr);

    dyn_box_result(cr, data_frame, |data_frame| {
        let data_frame = data_frame.borrow();

        let series = data_frame
            .get_columns()
            .iter()
            .map(|series| interpolate(series, method))
            .collect::<Vec<_>>();

        DataFrame::new(series).map(|df| Rc::new(RefCell::new(df)))
    })
}

#[ocaml_interop_export]
fn rust_data_frame_upsample(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
    by: OCamlRef<OCamlList<String>>,
    time_column: OCamlRef<String>,
    every: OCamlRef<String>,
    offset: OCamlRef<String>,
    stable: OCamlRef<bool>,
) -> OCaml<Result<DynBox<PolarsDataFrame>, String>> {
    let by: Vec<String> = by.to_rust(cr);
    let time_column: String = time_column.to_rust(cr);
    let every: String = every.to_rust(cr);
    let offset: String = offset.to_rust(cr);
    let stable: bool = stable.to_rust(cr);

    dyn_box_result(cr, data_frame, |data_frame| {
        let result = if stable {
            data_frame.borrow().upsample_stable(
                &by,
                &time_column,
                Duration::parse(&every),
                Duration::parse(&offset),
            )
        } else {
            data_frame.borrow().upsample(
                &by,
                &time_column,
                Duration::parse(&every),
                Duration::parse(&offset),
            )
        };

        result.map(|df| Rc::new(RefCell::new(df)))
    })
}

#[ocaml_interop_export]
fn rust_data_frame_explode(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
    columns: OCamlRef<OCamlList<String>>,
) -> OCaml<Result<DynBox<PolarsDataFrame>, String>> {
    let columns: Vec<String> = columns.to_rust(cr);

    dyn_box_result(cr, data_frame, |data_frame| {
        let data_frame = data_frame.borrow();
        data_frame
            .explode(columns)
            .map(|df| Rc::new(RefCell::new(df)))
    })
}

#[ocaml_interop_export]
fn rust_data_frame_schema(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
) -> OCaml<DynBox<Schema>> {
    dyn_box(cr, data_frame, |data_frame| {
        let data_frame = data_frame.borrow();
        data_frame.schema()
    })
}

#[ocaml_interop_export]
fn rust_data_frame_to_string_hum(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
) -> OCaml<String> {
    let Abstract(data_frame) = data_frame.to_rust(cr);
    let data_frame = data_frame.borrow();
    data_frame.to_string().to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_data_frame_partition_by(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
    by: OCamlRef<OCamlList<String>>,
    maintain_order: OCamlRef<bool>,
) -> OCaml<Result<OCamlList<DynBox<PolarsDataFrame>>, String>> {
    let Abstract(data_frame) = data_frame.to_rust(cr);
    let data_frame = data_frame.borrow();
    let cols: Vec<String> = by.to_rust(cr);
    let partitioned = if maintain_order.to_rust(cr) {
        data_frame.partition_by_stable(cols, true)
    } else {
        data_frame.partition_by(cols, true)
    };
    partitioned
        .map(|v| {
            v.into_iter()
                .map(|df| Abstract(Rc::new(RefCell::new(df))))
                .collect::<Vec<_>>()
        })
        .map_err(|err| err.to_string())
        .to_ocaml(cr)
}

fn modify_series_at_chunk_index(
    cr: &&mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
    data_type: OCamlRef<GADTDataType>,
    series_index: OCamlRef<OCamlInt>,
    chunk_index: OCamlRef<OCamlInt>,
    indices_and_values: OCamlRef<OCamlList<(OCamlInt, DummyBoxRoot)>>,
    are_values_options: bool,
) -> Result<(), String> {
    let Abstract(data_frame) = data_frame.to_rust(cr);
    let mut data_frame = data_frame.borrow_mut();
    let columns = unsafe { data_frame.get_columns_mut() };
    let series_index = series_index.to_rust::<Coerce<_, i64, usize>>(cr).get()?;
    let series = match columns.get_mut(series_index) {
        Some(series) => Ok(series),
        None => Err(format!("Column index out of bounds: {}", series_index)),
    }?;
    crate::series::modify_series_at_chunk_index(
        cr,
        series,
        data_type,
        chunk_index,
        indices_and_values,
        are_values_options,
    )
    .map_err(|err| err.to_string())
}

#[ocaml_interop_export]
fn rust_data_frame_modify_series_at_chunk_index(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
    data_type: OCamlRef<GADTDataType>,
    series_index: OCamlRef<OCamlInt>,
    chunk_index: OCamlRef<OCamlInt>,
    indices_and_values: OCamlRef<OCamlList<(OCamlInt, DummyBoxRoot)>>,
) -> OCaml<Result<DynBox<()>, String>> {
    modify_series_at_chunk_index(
        cr,
        data_frame,
        data_type,
        series_index,
        chunk_index,
        indices_and_values,
        false,
    )
    .map(Abstract)
    .to_ocaml(cr)
}

#[ocaml_interop_export]
fn rust_data_frame_modify_optional_series_at_chunk_index(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
    data_type: OCamlRef<GADTDataType>,
    series_index: OCamlRef<OCamlInt>,
    chunk_index: OCamlRef<OCamlInt>,
    indices_and_values: OCamlRef<OCamlList<(OCamlInt, DummyBoxRoot)>>,
) -> OCaml<Result<DynBox<()>, String>> {
    modify_series_at_chunk_index(
        cr,
        data_frame,
        data_type,
        series_index,
        chunk_index,
        indices_and_values,
        true,
    )
    .map(Abstract)
    .to_ocaml(cr)
}

#[ocaml_interop_export(raise_on_err)]
fn rust_data_frame_clear_mut(
    cr: &mut &mut OCamlRuntime,
    data_frame: OCamlRef<DynBox<PolarsDataFrame>>,
) -> OCaml<()> {
    let Abstract(data_frame) = data_frame.to_rust(cr);
    let mut data_frame = data_frame.borrow_mut();
    *data_frame = data_frame.clear();

    OCaml::unit()
}
