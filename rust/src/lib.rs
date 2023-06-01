use ocaml_interop::{
    ocaml_export, BoxRoot, DynBox, OCaml, OCamlBytes, OCamlInt, OCamlList, OCamlRef, ToOCaml,
};
use polars::prelude::*;
use std::{borrow::Borrow, path::Path};

// `ocaml_export` expands the function definitions by adding `pub` visibility and
// the required `#[no_mangle]` and `extern` declarations. It also takes care of
// acquiring the OCaml runtime handle and binding it to the name provided as
// the first parameter of the function.
ocaml_export! {
    // The first parameter is a name to which the GC frame handle will be bound to.
    // The remaining parameters must have type `OCamlRef<T>`, and the return
    // value `OCaml<T>`.
    fn rust_twice(cr, num: OCamlRef<OCamlInt>) -> OCaml<OCamlInt> {
        let num: i64 = num.to_rust(cr);
        unsafe { OCaml::of_i64_unchecked(num * 2) }
    }

    fn rust_series_new(cr, name: OCamlRef<String>, values: OCamlRef<OCamlList<OCamlInt>>) -> OCaml<DynBox<Series>> {
        let name: String = name.to_rust(cr);
        let values: Vec<i64> = values.to_rust(cr);
        OCaml::box_value(cr, Series::new(&name, values))
    }

    fn rust_series_to_string_hum(cr, series: OCamlRef<DynBox<Series>>) -> OCaml<String> {
        let series: OCaml<DynBox<Series>> = series.to_ocaml(cr);
        let series: &Series = Borrow::<Series>::borrow(&series);
        ToString::to_string(series).to_ocaml(cr)
    }

    fn rust_data_frame_new(cr, series: OCamlRef<OCamlList<DynBox<Series>>>) -> OCaml<Result<DynBox<DataFrame>,String>> {
        let mut series: OCaml<OCamlList<DynBox<Series>>> = series.to_ocaml(cr);

        let mut ret = Vec::new();
        while let Some((head, tail)) = series.uncons() {
            ret.push(Borrow::<Series>::borrow(&head).clone());

            series = tail;
        }

        match DataFrame::new(ret) {
            Err(err) => {
                Err::<BoxRoot<DynBox<DataFrame>>, _>(err.to_string()).to_ocaml(cr)
            },
            Ok(data_frame) => {
                let data_frame: BoxRoot<DynBox<DataFrame>> = OCaml::box_value(cr, data_frame).root();
                Ok::<_, String>(data_frame).to_ocaml(cr)
            }
        }
    }

    // TODO: properly return error type instead of a string
    fn rust_lazy_frame_of_parquet(cr, path: OCamlRef<OCamlBytes>) -> OCaml<Result<DynBox<LazyFrame>, String>>{
        let path:String = path.to_rust(cr);
        let path:&Path = Path::new(&path);

        match LazyFrame::scan_parquet(path, Default::default()) {
            Err(err) => {
                Err::<BoxRoot<DynBox<LazyFrame>>, _>(err.to_string()).to_ocaml(cr)
            },
            Ok(lazy_frame) => {
                let lazy_frame: BoxRoot<DynBox<LazyFrame>> = OCaml::box_value(cr, lazy_frame).root();
                Ok::<_, String>(lazy_frame).to_ocaml(cr)
            }
        }
    }

    fn rust_lazy_frame_to_dot(cr, lazy_frame: OCamlRef<DynBox<LazyFrame>>) -> OCaml<Result<String,String>>{
        let lazy_frame: OCaml<DynBox<LazyFrame>> = lazy_frame.to_ocaml(cr);

        // TODO: make configurable
        match Borrow::<LazyFrame>::borrow(&lazy_frame).to_dot(false) {
            Err(err) => {
                Err::<String, _>(err.to_string()).to_ocaml(cr)
            },
            Ok(dot) => {
                Ok::<_, String>(dot).to_ocaml(cr)
            }
        }
    }

    fn rust_lazy_frame_to_data_frame(cr, lazy_frame: OCamlRef<DynBox<LazyFrame>>)-> OCaml<Result<DynBox<DataFrame>, String>> {
        let lazy_frame: OCaml<DynBox<LazyFrame>> = lazy_frame.to_ocaml(cr);

        match Borrow::<LazyFrame>::borrow(&lazy_frame).clone().collect() {
            Err(err) => {
                Err::<BoxRoot<DynBox<DataFrame>>, _>(err.to_string()).to_ocaml(cr)
            },
            Ok(data_frame) => {
                let data_frame: BoxRoot<DynBox<DataFrame>> = OCaml::box_value(cr, data_frame).root();
                Ok::<_, String>(data_frame).to_ocaml(cr)
            }
        }
    }

    fn rust_data_frame_to_string_hum(cr, data_frame: OCamlRef<DynBox<DataFrame>>) -> OCaml<String> {
        let data_frame: OCaml<DynBox<DataFrame>> = data_frame.to_ocaml(cr);
        let data_frame: &DataFrame = Borrow::<DataFrame>::borrow(&data_frame);
        data_frame.to_string().to_ocaml(cr)
    }
}

//pub fn add(left: usize, right: usize) -> usize {
//    left + right
//}
//
//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    #[test]
//    fn it_works() {
//        let result = add(2, 2);
//        assert_eq!(result, 4);
//    }
//}
//
