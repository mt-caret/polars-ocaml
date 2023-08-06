use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated};

fn ocaml_interop_export_implementation(item_fn: syn::ItemFn) -> TokenStream2 {
    let mut inputs_iter = item_fn.sig.inputs.iter().map(|fn_arg| match fn_arg {
        syn::FnArg::Receiver(_) => panic!("receiver not supported"),
        syn::FnArg::Typed(pat_type) => pat_type.clone(),
    });

    // The first argument to the function corresponds to the OCaml runtime.
    let runtime_name = match *inputs_iter.next().unwrap().pat {
        syn::Pat::Ident(pat_ident) => pat_ident.ident,
        _ => panic!("expected ident"),
    };

    // The remaining arguments are stripped of their types and converted to
    // `RawOCaml` values.
    let new_inputs: Punctuated<_, _> = inputs_iter
        .clone()
        .map(|pat_type| {
            syn::FnArg::Typed(syn::PatType {
                ty: syn::parse2(quote! {
                    ::ocaml_interop::RawOCaml
                })
                .unwrap(),
                ..pat_type
            })
        })
        .collect();
    let number_of_arguments = new_inputs.len();

    let signature = syn::Signature {
        inputs: new_inputs,
        output: syn::parse2(quote! {
            -> ::ocaml_interop::RawOCaml
        })
        .unwrap(),
        ..item_fn.sig.clone()
    };

    // We take each non-runtime argument to the function and convert them to the
    // appropriate Rust type.
    let locals = inputs_iter.map(|pat_type| match *pat_type.pat {
        syn::Pat::Ident(pat_ident) => {
            let ident = pat_ident.ident;
            let ty = pat_type.ty;
            quote! {
                let #ident: #ty = &::ocaml_interop::BoxRoot::new(unsafe {
                    OCaml::new(cr, #ident)
                });
            }
        }
        _ => panic!("expected ident"),
    });

    let return_type = match item_fn.sig.output.clone() {
        syn::ReturnType::Default => panic!("functions with no return type are not supported"),
        syn::ReturnType::Type(_, ty) => ty,
    };
    let block = item_fn.block.clone();

    let native_function = quote! {
        #[no_mangle]
        pub extern "C" #signature {
            let #runtime_name = unsafe { &mut ::ocaml_interop::OCamlRuntime::recover_handle() };

            #( #locals )*

            {
                let return_value: #return_type = #block;

                unsafe { return_value.raw() }
            }
        }
    };

    // We need to generate different functions for the bytecode and native
    // versions of the function if there is more than a certain number of arguments.
    // See https://v2.ocaml.org/manual/intfc.html#ss:c-prim-impl for details.
    if number_of_arguments > 5 {
        let native_function_name = item_fn.sig.ident;

        let bytecode_function_name = syn::Ident::new(
            &format!("{}_bytecode", native_function_name),
            Span::call_site(),
        );

        let arguments = (0..number_of_arguments).map(|i| {
            quote! {
                argv[#i]
            }
        });

        quote! {
            #native_function

            #[no_mangle]
            pub extern "C" fn #bytecode_function_name(
            argv: &[::ocaml_interop::RawOCaml],
            argn: isize,
            ) -> ::ocaml_interop::RawOCaml {
                if argn as usize != #number_of_arguments {
                    panic!("expected {} arguments, got {}", #number_of_arguments, argn);
                }

                #native_function_name(#( #arguments ),*)
            }
        }
    } else {
        native_function
    }
}

#[proc_macro_attribute]
pub fn ocaml_interop_export(_input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(annotated_item as syn::ItemFn);

    let expanded = ocaml_interop_export_implementation(item_fn);

    TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use proc_macro2::TokenStream as TokenStream2;

    fn pretty_print_item(item: &TokenStream2) -> String {
        let file: syn::File = syn::parse2(item.clone()).unwrap();

        prettyplease::unparse(&file)
    }

    fn apply_macro_and_pretty_print(input: TokenStream2) -> String {
        let item_fn = syn::parse2(input).unwrap();
        let expanded = ocaml_interop_export_implementation(item_fn);
        pretty_print_item(&expanded)
    }

    #[test]
    fn test_simple_function() {
        let macro_output = apply_macro_and_pretty_print(quote! {
            fn rust_expr_col(
                cr: &mut &mut OCamlRuntime,
                name: OCamlRef<String>
            ) -> OCaml<DynBox<Expr>> {
                let name: String = name.to_rust(cr);
                OCaml::box_value(cr, col(&name))
            }
        });

        expect![[r##"
            #[no_mangle]
            pub extern "C" fn rust_expr_col(
                name: ::ocaml_interop::RawOCaml,
            ) -> ::ocaml_interop::RawOCaml {
                let cr = unsafe { &mut ::ocaml_interop::OCamlRuntime::recover_handle() };
                let name: OCamlRef<String> = &::ocaml_interop::BoxRoot::new(unsafe {
                    OCaml::new(cr, name)
                });
                {
                    let return_value: OCaml<DynBox<Expr>> = {
                        let name: String = name.to_rust(cr);
                        OCaml::box_value(cr, col(&name))
                    };
                    unsafe { return_value.raw() }
                }
            }
        "##]]
        .assert_eq(&macro_output);
    }

    #[test]
    fn test_bytecode_generation() {
        let macro_output = apply_macro_and_pretty_print(quote! {
        fn rust_expr_sample_n(
            cr: &mut &mut OCamlRuntime,
            expr: OCamlRef<DynBox<Expr>>,
            n: OCamlRef<OCamlInt>,
            with_replacement: OCamlRef<bool>,
            shuffle: OCamlRef<bool>,
            seed: OCamlRef<Option<OCamlInt>>,
            fixed_seed: OCamlRef<bool>
        ) -> OCaml<DynBox<Expr>> {
            let Abstract(expr) = expr.to_rust(cr);
            let n = n.to_rust::<Coerce<_, i64, usize>>(cr).get();
            let with_replacement: bool = with_replacement.to_rust(cr);
            let shuffle: bool = shuffle.to_rust(cr);
            let seed = seed.to_rust::<Coerce<_, Option<i64>, Option<u64>>>(cr).get();
            let fixed_seed = fixed_seed.to_rust(cr);

            Abstract(expr.sample_n(n, with_replacement, shuffle, seed, fixed_seed)).to_ocaml(cr)
        }
        });

        expect![[r##"
            #[no_mangle]
            pub extern "C" fn rust_expr_sample_n(
                expr: ::ocaml_interop::RawOCaml,
                n: ::ocaml_interop::RawOCaml,
                with_replacement: ::ocaml_interop::RawOCaml,
                shuffle: ::ocaml_interop::RawOCaml,
                seed: ::ocaml_interop::RawOCaml,
                fixed_seed: ::ocaml_interop::RawOCaml,
            ) -> ::ocaml_interop::RawOCaml {
                let cr = unsafe { &mut ::ocaml_interop::OCamlRuntime::recover_handle() };
                let expr: OCamlRef<DynBox<Expr>> = &::ocaml_interop::BoxRoot::new(unsafe {
                    OCaml::new(cr, expr)
                });
                let n: OCamlRef<OCamlInt> = &::ocaml_interop::BoxRoot::new(unsafe {
                    OCaml::new(cr, n)
                });
                let with_replacement: OCamlRef<bool> = &::ocaml_interop::BoxRoot::new(unsafe {
                    OCaml::new(cr, with_replacement)
                });
                let shuffle: OCamlRef<bool> = &::ocaml_interop::BoxRoot::new(unsafe {
                    OCaml::new(cr, shuffle)
                });
                let seed: OCamlRef<Option<OCamlInt>> = &::ocaml_interop::BoxRoot::new(unsafe {
                    OCaml::new(cr, seed)
                });
                let fixed_seed: OCamlRef<bool> = &::ocaml_interop::BoxRoot::new(unsafe {
                    OCaml::new(cr, fixed_seed)
                });
                {
                    let return_value: OCaml<DynBox<Expr>> = {
                        let Abstract(expr) = expr.to_rust(cr);
                        let n = n.to_rust::<Coerce<_, i64, usize>>(cr).get();
                        let with_replacement: bool = with_replacement.to_rust(cr);
                        let shuffle: bool = shuffle.to_rust(cr);
                        let seed = seed.to_rust::<Coerce<_, Option<i64>, Option<u64>>>(cr).get();
                        let fixed_seed = fixed_seed.to_rust(cr);
                        Abstract(expr.sample_n(n, with_replacement, shuffle, seed, fixed_seed))
                            .to_ocaml(cr)
                    };
                    unsafe { return_value.raw() }
                }
            }
            #[no_mangle]
            pub extern "C" fn rust_expr_sample_n_bytecode(
                argv: &[::ocaml_interop::RawOCaml],
                argn: isize,
            ) -> ::ocaml_interop::RawOCaml {
                if argn as usize != 6usize {
                    panic!("expected {} arguments, got {}", 6usize, argn);
                }
                rust_expr_sample_n(
                    argv[0usize],
                    argv[1usize],
                    argv[2usize],
                    argv[3usize],
                    argv[4usize],
                    argv[5usize],
                )
            }
        "##]]
        .assert_eq(&macro_output);
    }
}
