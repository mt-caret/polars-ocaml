use crate::utils::*;
use ocaml_interop::{
    ocaml_export, DynBox, OCaml, OCamlFloat, OCamlInt, OCamlList, OCamlRef, ToOCaml,
};
use polars::prelude::*;

ocaml_export! {
    fn rust_data_frame_new(cr, series: OCamlRef<OCamlList<DynBox<Series>>>) -> OCaml<Result<DynBox<DataFrame>,String>> {
        let series: Vec<Series> = unwrap_abstract_vec(series.to_rust(cr));

        DataFrame::new(series).map(Abstract).map_err(|err| err.to_string()).to_ocaml(cr)
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

    fn rust_data_frame_schema(cr, data_frame: OCamlRef<DynBox<DataFrame>>) -> OCaml<DynBox<Schema>> {
        let Abstract(data_frame) = data_frame.to_rust(cr);
        OCaml::box_value(cr, data_frame.schema())
    }

    fn rust_data_frame_to_string_hum(cr, data_frame: OCamlRef<DynBox<DataFrame>>) -> OCaml<String> {
        let Abstract(data_frame) = data_frame.to_rust(cr);
        data_frame.to_string().to_ocaml(cr)
    }
}
