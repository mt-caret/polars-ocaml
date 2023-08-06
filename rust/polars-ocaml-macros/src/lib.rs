use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::parse_macro_input;

fn ocaml_interop_export_implementation(item_fn: syn::ItemFn) -> TokenStream2 {
    let mut inputs_iter = item_fn.sig.inputs.iter().map(|fn_arg| match fn_arg {
        syn::FnArg::Receiver(_) => panic!("receiver not supported"),
        syn::FnArg::Typed(pat_type) => pat_type.clone(),
    });

    let runtime_name = match *inputs_iter.next().unwrap().pat {
        syn::Pat::Ident(pat_ident) => pat_ident.ident,
        _ => panic!("expected ident"),
    };

    let new_inputs = inputs_iter
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

    let signature = syn::Signature {
        inputs: new_inputs,
        output: syn::parse2(quote! {
            -> ::ocaml_interop::RawOCaml
        })
        .unwrap(),
        ..item_fn.sig
    };

    let locals = inputs_iter.map(|pat_type| match *pat_type.pat {
        syn::Pat::Ident(pat_ident) => {
            let ident = pat_ident.ident;
            let ty = pat_type.ty;
            quote! {
                let #ident: #ty = &::caml_interop::BoxRoot::new(unsafe {
                    OCaml::new(cr, names)
                });
            }
        }
        _ => panic!("expected ident"),
    });

    let return_type = match item_fn.sig.output.clone() {
        syn::ReturnType::Default => quote! { () },
        syn::ReturnType::Type(_, ty) => ty.into_token_stream(),
    };
    let block = item_fn.block.clone();

    quote! {
        #[no_mangle]
        pub extern "C" #signature {
            let #runtime_name = unsafe { &mut ::ocaml_interop::OCamlRuntime::recover_handle() };

            #( #locals )*

            {
                let return_value: #return_type = #block;

                unsafe { return_value.raw() }
            }
        }
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

    fn example_tokens() -> TokenStream2 {
        quote! {
            fn rust_expr_col(
                cr: &mut &mut OCamlRuntime,
                name: OCamlRef<String>
            ) -> OCaml<DynBox<Expr>> {
                let name: String = name.to_rust(cr);
                OCaml::box_value(cr, col(&name))
            }
        }
    }

    fn pretty_print_item(item: &TokenStream2) -> String {
        let item = syn::parse2(item.clone()).unwrap();
        let file = syn::File {
            attrs: vec![],
            items: vec![item],
            shebang: None,
        };

        prettyplease::unparse(&file)
    }

    #[test]
    fn example_run() {
        let example_tokens = example_tokens();

        expect![[r#"
            fn rust_expr_col(
                cr: &mut &mut OCamlRuntime,
                name: OCamlRef<String>,
            ) -> OCaml<DynBox<Expr>> {
                let name: String = name.to_rust(cr);
                OCaml::box_value(cr, col(&name))
            }
        "#]]
        .assert_eq(&pretty_print_item(&example_tokens));

        expect![[r##"
            #[no_mangle]
            pub extern "C" fn rust_expr_col(
                name: ::ocaml_interop::RawOCaml,
            ) -> ::ocaml_interop::RawOCaml {
                let cr = unsafe { &mut ::ocaml_interop::OCamlRuntime::recover_handle() };
                let name: OCamlRef<String> = &::caml_interop::BoxRoot::new(unsafe {
                    OCaml::new(cr, names)
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
        .assert_eq(&pretty_print_item(&TokenStream2::from(
            ocaml_interop_export_implementation(syn::parse2(example_tokens).unwrap()),
        )));
    }
}
