//! Proc macros for the dyn-slice crate

#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::perf,
    clippy::cargo,
    clippy::alloc_instead_of_core,
    clippy::std_instead_of_alloc,
    clippy::std_instead_of_core,
    clippy::get_unwrap,
    clippy::panic_in_result_fn,
    clippy::pub_without_shorthand,
    clippy::redundant_type_annotations,
    clippy::todo,
    clippy::undocumented_unsafe_blocks
)]

mod declare_new_fns;
use declare_new_fns::DeclareNewFns;
mod path_ext;
use proc_macro2::TokenStream;
use syn::{spanned::Spanned, Path, TraitBound, TypeParamBound};

#[proc_macro]
pub fn declare_new_fns(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeclareNewFns = syn::parse_macro_input!(input);
    TokenStream::try_from(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn stringify_basic_path(path: &Path) -> syn::Result<String> {
    path.segments
        .iter()
        .map(|x| x.ident.to_string())
        .reduce(|mut acc, curr| {
            acc.push_str("::");
            acc.push_str(&curr);
            acc
        })
        .ok_or_else(|| syn::Error::new(path.span(), "empty path"))
}

fn type_param_bound_select_trait(bound: &mut TypeParamBound) -> Option<&mut TraitBound> {
    if let TypeParamBound::Trait(trait_bound) = bound {
        Some(trait_bound)
    } else {
        None
    }
}
