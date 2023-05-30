use ocaml_interop::{ocaml_export, DynBox, OCaml, OCamlBytes, OCamlInt, OCamlRef, ToOCaml, BoxRoot};
use polars::prelude::LazyFrame;
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
