extern crate proc_macro;
use proc_macro::TokenStream;
use syn::spanned::Spanned;

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as syn::ItemFn);
    let span = item.span();

    let fn_name = item.sig.ident.clone();
    let Some(_) = item.sig.asyncness else {
        return syn::Error::new(span, "entry point function must be async")
            .into_compile_error()
            .into();
    };

    if item.sig.inputs.len() != 0 {
        return syn::Error::new(span, "entry point function must not have any parameters")
            .into_compile_error()
            .into();
    }

    let result = quote::quote! {
        fn main() {
            #item
            let mut ex = oxic::prelude::Executor::new();
            ex.block_on(
            #fn_name()
                );
        }
    };

    return result.into();
}
