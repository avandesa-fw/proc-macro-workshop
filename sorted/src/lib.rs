mod sorted;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = args.into();
    let item = parse_macro_input!(input as syn::Item);

    sorted::execute(args, item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
