use ocaml_interop::{
    ocaml_export, BoxRoot, DynBox, OCaml, OCamlBytes, OCamlFloat, OCamlInt, OCamlList, OCamlRef,
    OCamlRuntime, ToOCaml,
};
use polars::prelude::*;
use std::fmt::Display;
use std::{borrow::Borrow, path::Path};

fn ocaml_list_to_vec<T: 'static + Clone>(mut list: OCaml<OCamlList<DynBox<T>>>) -> Vec<T> {
    let mut ret = Vec::new();

    while let Some((head, tail)) = list.uncons() {
        ret.push((*Borrow::<T>::borrow(&head)).clone());
        list = tail;
    }

    ret
}

fn box_result<'a, T: 'static, E: Display>(
    cr: &'a mut &'a mut OCamlRuntime,
    result: Result<T, E>,
) -> OCaml<'a, Result<DynBox<T>, String>> {
    match result {
        Err(err) => Err::<BoxRoot<DynBox<_>>, _>(err.to_string()).to_ocaml(cr),
        Ok(v) => {
            let v: BoxRoot<DynBox<T>> = OCaml::box_value(cr, v).root();
            Ok::<_, String>(v).to_ocaml(cr)
        }
    }
}

fn expr_binary_op<'a>(
    cr: &'a mut &'a mut OCamlRuntime,
    expr: OCamlRef<'a, DynBox<Expr>>,
    other: OCamlRef<'a, DynBox<Expr>>,
    f: impl Fn(Expr, Expr) -> Expr,
) -> OCaml<'a, DynBox<Expr>> {
    let expr: Expr = Borrow::<Expr>::borrow(&expr.to_ocaml(cr)).clone();
    let other: Expr = Borrow::<Expr>::borrow(&other.to_ocaml(cr)).clone();
    OCaml::box_value(cr, f(expr, other))
}

// `ocaml_export` expands the function definitions by adding `pub` visibility and
// the required `#[no_mangle]` and `extern` declarations. It also takes care of
// acquiring the OCaml runtime handle and binding it to the name provided as
// the first parameter of the function.
ocaml_export! {
    fn rust_expr_col(cr, name: OCamlRef<String>) -> OCaml<DynBox<Expr>> {
        let name: String = name.to_rust(cr);
        OCaml::box_value(cr, col(&name))
    }

    fn rust_expr_int(cr, value: OCamlRef<OCamlInt>) -> OCaml<DynBox<Expr>> {
        let value: i64 = value.to_rust(cr);
        OCaml::box_value(cr, lit(value))
    }

    fn rust_expr_float(cr, value: OCamlRef<OCamlFloat>) -> OCaml<DynBox<Expr>> {
        let value: f64 = value.to_rust(cr);
        OCaml::box_value(cr, lit(value))
    }

    fn rust_expr_sort(cr, expr: OCamlRef<DynBox<Expr>>, descending: OCamlRef<bool>) -> OCaml<DynBox<Expr>> {
        let expr: Expr = Borrow::<Expr>::borrow(&expr.to_ocaml(cr)).clone();
        let descending: bool = descending.to_rust(cr);
        OCaml::box_value(cr, expr.sort(descending))
    }

    fn rust_expr_head(cr, expr: OCamlRef<DynBox<Expr>>, length: OCamlRef<Option<OCamlInt>>) -> OCaml<Option<DynBox<Expr>>> {
        let expr: Expr = Borrow::<Expr>::borrow(&expr.to_ocaml(cr)).clone();
        let length: Option<i64> = length.to_rust(cr);

        match length {
            None => {
                let expr: BoxRoot<DynBox<Expr>> = OCaml::box_value(cr, expr.head(None)).root();
                Some(expr).to_ocaml(cr)
            },
            Some(length) => {
                match length.try_into().ok() {
                    // TODO: this should probably be an error instead of none
                    None => OCaml::none(),
                    Some(length) => {
                        let expr: BoxRoot<DynBox<Expr>> = OCaml::box_value(cr, expr.head(Some(length))).root();
                        Some(expr).to_ocaml(cr)
                    },
                }
            }
        }
    }

    fn rust_expr_filter(cr, expr: OCamlRef<DynBox<Expr>>, predicate: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        let expr: Expr = Borrow::<Expr>::borrow(&expr.to_ocaml(cr)).clone();
        let predicate: Expr = Borrow::<Expr>::borrow(&predicate.to_ocaml(cr)).clone();
        OCaml::box_value(cr, expr.filter(predicate))
    }

    fn rust_expr_sum(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        let expr: Expr = Borrow::<Expr>::borrow(&expr.to_ocaml(cr)).clone();
        OCaml::box_value(cr, expr.sum())
    }

    fn rust_expr_alias(cr, expr: OCamlRef<DynBox<Expr>>, name: OCamlRef<String>) -> OCaml<DynBox<Expr>> {
        let expr: Expr = Borrow::<Expr>::borrow(&expr.to_ocaml(cr)).clone();
        let name: String = name.to_rust(cr);
        OCaml::box_value(cr, expr.alias(&name))
    }

    fn rust_expr_eq(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a.eq(b))
    }

    fn rust_expr_neq(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a.neq(b))
    }

    fn rust_expr_gt(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a.gt(b))
    }

    fn rust_expr_gt_eq(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a.gt_eq(b))
    }

    fn rust_expr_lt(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a.lt(b))
    }

    fn rust_expr_lt_eq(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a.lt_eq(b))
    }

    fn rust_expr_not(cr, expr: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        let expr: Expr = Borrow::<Expr>::borrow(&expr.to_ocaml(cr)).clone();
        OCaml::box_value(cr, expr.not())
    }

    fn rust_expr_and(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a.and(b))
    }

    fn rust_expr_or(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a.or(b))
    }

    fn rust_expr_xor(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a.xor(b))
    }

    fn rust_expr_add(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a + b)
    }

    fn rust_expr_sub(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a - b)
    }

    fn rust_expr_mul(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a * b)
    }

    fn rust_expr_div(cr, expr: OCamlRef<DynBox<Expr>>, other: OCamlRef<DynBox<Expr>>) -> OCaml<DynBox<Expr>> {
        expr_binary_op(cr, expr, other, |a, b| a / b)
    }

    fn rust_series_new_int(cr, name: OCamlRef<String>, values: OCamlRef<OCamlList<OCamlInt>>) -> OCaml<DynBox<Series>> {
        let name: String = name.to_rust(cr);
        let values: Vec<i64> = values.to_rust(cr);
        OCaml::box_value(cr, Series::new(&name, values))
    }

    fn rust_series_new_int_option(cr, name: OCamlRef<String>, values: OCamlRef<OCamlList<Option<OCamlInt>>>) -> OCaml<DynBox<Series>> {
        let name: String = name.to_rust(cr);
        let values: Vec<Option<i64>> = values.to_rust(cr);
        OCaml::box_value(cr, Series::new(&name, values))
    }

    fn rust_series_new_float(cr, name: OCamlRef<String>, values: OCamlRef<OCamlList<OCamlFloat>>) -> OCaml<DynBox<Series>> {
        let name: String = name.to_rust(cr);
        let values: Vec<f64> = values.to_rust(cr);
        OCaml::box_value(cr, Series::new(&name, values))
    }

    fn rust_series_new_float_option(cr, name: OCamlRef<String>, values: OCamlRef<OCamlList<Option<OCamlFloat>>>) -> OCaml<DynBox<Series>> {
        let name: String = name.to_rust(cr);
        let values: Vec<Option<f64>> = values.to_rust(cr);
        OCaml::box_value(cr, Series::new(&name, values))
    }

    fn rust_series_new_string(cr, name: OCamlRef<String>, values: OCamlRef<OCamlList<String>>) -> OCaml<DynBox<Series>> {
        let name: String = name.to_rust(cr);
        let values: Vec<String> = values.to_rust(cr);
        OCaml::box_value(cr, Series::new(&name, values))
    }

    fn rust_series_new_string_option(cr, name: OCamlRef<String>, values: OCamlRef<OCamlList<Option<String>>>) -> OCaml<DynBox<Series>> {
        let name: String = name.to_rust(cr);
        let values: Vec<Option<String>> = values.to_rust(cr);
        OCaml::box_value(cr, Series::new(&name, values))
    }

    fn rust_series_to_string_hum(cr, series: OCamlRef<DynBox<Series>>) -> OCaml<String> {
        let series: OCaml<DynBox<Series>> = series.to_ocaml(cr);
        let series: &Series = Borrow::<Series>::borrow(&series);
        ToString::to_string(series).to_ocaml(cr)
    }

    fn rust_data_frame_new(cr, series: OCamlRef<OCamlList<DynBox<Series>>>) -> OCaml<Result<DynBox<DataFrame>,String>> {
        let series = ocaml_list_to_vec(series.to_ocaml(cr));

        match DataFrame::new(series) {
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

    fn rust_data_frame_lazy(cr, data_frame: OCamlRef<DynBox<DataFrame>>) -> OCaml<DynBox<LazyFrame>> {
        let data_frame: OCaml<DynBox<DataFrame>> = data_frame.to_ocaml(cr);
        let data_frame: DataFrame = Borrow::<DataFrame>::borrow(&data_frame).clone();
        OCaml::box_value(cr, data_frame.lazy())
    }

    // TODO: properly return error type instead of a string
    fn rust_lazy_frame_scan_parquet(cr, path: OCamlRef<OCamlBytes>) -> OCaml<Result<DynBox<LazyFrame>, String>>{
        let path:String = path.to_rust(cr);
        let path:&Path = Path::new(&path);

        box_result(cr, LazyFrame::scan_parquet(path, Default::default()))
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

    fn rust_lazy_frame_collect(cr, lazy_frame: OCamlRef<DynBox<LazyFrame>>)-> OCaml<Result<DynBox<DataFrame>, String>> {
        let lazy_frame: OCaml<DynBox<LazyFrame>> = lazy_frame.to_ocaml(cr);
        let lazy_frame = Borrow::<LazyFrame>::borrow(&lazy_frame).clone();

        box_result(cr, lazy_frame.collect())
    }

    fn rust_lazy_frame_select(cr, lazy_frame: OCamlRef<DynBox<LazyFrame>>, exprs: OCamlRef<OCamlList<DynBox<Expr>>>) -> OCaml<DynBox<LazyFrame>> {
        let exprs = ocaml_list_to_vec(exprs.to_ocaml(cr));

        let lazy_frame: OCaml<DynBox<LazyFrame>> = lazy_frame.to_ocaml(cr);
        let lazy_frame:LazyFrame =  Borrow::<LazyFrame>::borrow(&lazy_frame).clone();

        OCaml::box_value(cr, lazy_frame.select(&exprs))
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
