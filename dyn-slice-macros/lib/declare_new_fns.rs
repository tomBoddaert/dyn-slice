use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Attribute, Error, Expr, ExprPath, GenericArgument, GenericParam, Generics, Ident, Lifetime,
    Meta, Path, PathSegment, Token, TypeParamBound, TypePath, Visibility, WhereClause,
};

use crate::{
    path_ext::{make_generics_inner_path, make_inner_path, RESERVED},
    stringify_basic_path, type_param_bound_select_trait,
};

/// A definition for a set of new functions for `DynSlice`s
pub struct DeclareNewFns {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub ident: Ident,
    pub generics: Generics,
    pub object_bounds: Punctuated<TypeParamBound, Token![+]>,
}

impl Parse for DeclareNewFns {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse the attributes for the module
        // This will mostly be docs
        let attrs = input.call(Attribute::parse_outer)?;

        // Parse the visibility for the module
        let vis = input.parse()?;
        // Parse the name of the module
        let ident = input.parse()?;

        // Optionally parse generics
        let mut generics = parse_optional_generics(input)?;

        // Parse the traits and lifetime bounds
        let object_bounds = input.call(Punctuated::parse_separated_nonempty)?;

        // Parse the where clause
        generics.where_clause = input.parse()?;

        Ok(Self {
            attrs,
            vis,
            ident,
            generics,
            object_bounds,
        })
    }
}

fn parse_optional_generics(input: ParseStream) -> syn::Result<Generics> {
    // This function is adapted from part of the parse_impl function in the syn crate
    // https://docs.rs/syn/2.0.42/src/syn/item.rs.html#2469-2571
    //
    // syn (https://github.com/dtolnay/syn) by David Tolnay (dtolnay, https://github.com/dtolnay)
    // is licensed under MIT OR Apache-2.0

    let has_generics = input.peek(Token![<])
        && (input.peek2(Token![>])
            || input.peek2(Token![#])
            || (input.peek2(Ident) || input.peek2(Lifetime))
                && (input.peek3(Token![:])
                    || input.peek3(Token![,])
                    || input.peek3(Token![>])
                    || input.peek3(Token![=]))
            || input.peek2(Token![const]));

    // Parse the generics if detected
    if has_generics {
        input.parse()
    } else {
        Ok(Generics::default())
    }
}

impl From<DeclareNewFns> for TokenStream {
    fn from(value: DeclareNewFns) -> Self {
        let DeclareNewFns {
            mut attrs,
            vis,
            ident,
            mut generics,
            mut object_bounds,
        } = value;

        // Get the dyn-slice crate path
        let crate_ = match get_crate(&mut attrs) {
            Ok(path) => path,
            Err(err) => return err.into_compile_error(),
        };

        let mut generic_idents: Vec<String> =
            RESERVED.iter().copied().map(ToOwned::to_owned).collect();
        generic_idents.extend(generics.params.iter().filter_map(|param| match param {
            GenericParam::Type(r#type) => Some(r#type.ident.to_string()),
            GenericParam::Const(r#const) => Some(r#const.ident.to_string()),
            GenericParam::Lifetime(_) => None,
        }));

        // Create a clone before editing
        // let outer_trait_paths = traits.clone();
        let outer_trait_object = object_bounds.clone();

        // Make paths inner paths
        // traits
        //     .iter_mut()
        //     .for_each(|path| make_inner_path(path, &generic_idents));
        for bound in &mut object_bounds
            .iter_mut()
            .filter_map(type_param_bound_select_trait)
        {
            make_inner_path(&mut bound.path, &generic_idents);
        }

        make_generics_inner_path(&mut generics, &generic_idents);

        // Get the path of the trait for documentation
        // This is done as a string rather than using `r#trait` in the quote
        // directly because syn puts spaces around the :: delimiters, which breaks
        // rustdoc linking
        let mut outer_trait_paths: Vec<String> = outer_trait_object
            .iter()
            .filter_map(|bound| {
                if let TypeParamBound::Trait(trait_bound) = bound {
                    Some(&trait_bound.path)
                } else {
                    None
                }
            })
            .map(stringify_basic_path)
            .collect();
        let mut inner_trait_paths: Vec<String> = object_bounds
            .iter()
            .filter_map(|bound| {
                if let TypeParamBound::Trait(trait_bound) = bound {
                    Some(&trait_bound.path)
                } else {
                    None
                }
            })
            .map(stringify_basic_path)
            .collect();
        // let mut inner_trait_paths: Vec<String> = traits.iter().map(stringify_basic_path).collect();

        // Get the trait names for documentation
        let mut trait_names: Vec<String> = object_bounds
            .iter()
            .filter_map(|bound| {
                if let TypeParamBound::Trait(trait_bound) = bound {
                    Some(&trait_bound.path)
                } else {
                    None
                }
            })
            .map(|r#trait| {
                r#trait
                    .segments
                    .last()
                    .expect("empty trait path")
                    .ident
                    .to_string()
            })
            .collect();

        // Get the first of the trait documentation to put before the first '+'
        let trait_docs = TraitDocs {
            name: trait_names.remove(0),
            outer_path: outer_trait_paths.remove(0),
            inner_path: inner_trait_paths.remove(0),
        };

        let auto_trait_docs = TraitDocs {
            name: trait_names.as_slice(),
            outer_path: outer_trait_paths.as_slice(),
            inner_path: inner_trait_paths.as_slice(),
        };

        let data = Data {
            attrs,
            vis,
            ident,
            generics,
            object_bounds,
        };

        declare_new_fns_quote(data, &crate_, trait_docs, auto_trait_docs)
    }
}

fn get_crate(attrs: &mut Vec<Attribute>) -> syn::Result<Path> {
    // Make the crate name `dyn_slice` by default
    let mut crate_ = Path::from(PathSegment::from(Ident::new(
        "dyn_slice",
        Span::mixed_site(),
    )));

    // Check for a `crate = <path>` attribute macro
    if let Some((i, value)) = attrs
        .iter()
        .enumerate()
        .find_map(|(i, Attribute { meta, .. })| {
            let Meta::NameValue(name_value) = meta else {
                return None;
            };

            if !name_value
                .path
                .is_ident(&Ident::new("crate", Span::call_site()))
            {
                return None;
            }

            Some((i, &name_value.value))
        })
    {
        let Expr::Path(crate_new) = value else {
            return Err(Error::new(
                Span::call_site(),
                "'crate' attribute value must be the crate path",
            ));
        };
        // Set the crate path
        crate_ = crate_new.path.clone();

        // Remove the crate attribute
        attrs.remove(i);
    }

    Ok(crate_)
}

#[derive(Clone)]
struct Data {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    object_bounds: Punctuated<TypeParamBound, Token![+]>,
}

#[derive(Clone, Copy, Debug)]
struct TraitDocs<T> {
    name: T,
    outer_path: T,
    inner_path: T,
}

fn declare_new_fns_quote(
    data: Data,
    crate_: &Path,
    trait_docs: TraitDocs<String>,
    auto_trait_docs: TraitDocs<&[String]>,
) -> TokenStream {
    let Data {
        attrs,
        vis,
        ident,
        mut generics,
        object_bounds,
    } = data;

    let TraitDocs {
        name: trait_name,
        outer_path: trait_outer_path,
        inner_path: trait_inner_path,
    } = trait_docs;

    let TraitDocs {
        name: auto_trait_names,
        outer_path: auto_trait_outer_paths,
        inner_path: auto_trait_inner_paths,
    } = auto_trait_docs;

    let where_predicates =
        generics
            .where_clause
            .take()
            .map(|WhereClause { mut predicates, .. }| {
                if !predicates.empty_or_trailing() {
                    predicates.push_punct(<Token![,]>::default());
                }

                predicates
            });

    if !generics.params.empty_or_trailing() {
        generics.params.push_punct(<Token![,]>::default());
    }

    // Get generics without brackets
    let full_generics = &generics.params;
    // Create generics without bounds for type aliases
    let stripped_generics = remove_generic_bounds(full_generics);
    // Get arguments to Dyn
    let arguments = get_arguments(full_generics);

    quote! {
        #[doc = concat!("new functions for `&dyn [`[`", #trait_name, "`](", #trait_outer_path, ")", #( "` + `[`", #auto_trait_names, "`](", #auto_trait_outer_paths, ")" ,)* "`]`")]
        #( #attrs )*
        #vis mod #ident {
            use core::{
                mem::transmute,
                ptr::{metadata, null, DynMetadata, Pointee},
            };

            use #crate_ as dyn_slice;
            use dyn_slice::{DynSlice, DynSliceMut};

            #[doc = concat!("An alias for `dyn `[`", #trait_name, "`](", #trait_inner_path, ")" #(, "` + `[`", #auto_trait_names, "`](", #auto_trait_inner_paths, ")" )*)]
            pub type Dyn<#stripped_generics> = dyn #object_bounds;

            #[doc = concat!("An alias for `&dyn [`[`", #trait_name, "`](", #trait_inner_path, ")", #( "` + `[`", #auto_trait_names, "`](", #auto_trait_inner_paths, ")" ,)* "`]` ([`DynSlice<Dyn>`])")]
            pub type Slice<'__slice, #stripped_generics> = DynSlice<'__slice, Dyn<#arguments>>;

            #[doc = concat!("An alias for `&mut dyn [`[`", #trait_name, "`](", #trait_inner_path, ")", #( "` + `[`", #auto_trait_names, "`](", #auto_trait_inner_paths, ")" ,)* "`]` ([`DynSliceMut<Dyn>`])")]
            pub type SliceMut<'__slice, #stripped_generics> = DynSliceMut<'__slice, Dyn<#arguments>>;

            #[allow(unused)]
            #[must_use]
            #[doc = concat!("Create a dyn slice from a slice of a type that implements [`", #trait_name, "`](", #trait_inner_path, ")" #(, "` + `[`", #auto_trait_names, "`](", #auto_trait_inner_paths, ")" )*)]
            pub fn new<#full_generics DynSliceFromType>(value: &[DynSliceFromType]) -> Slice<'_, #arguments>
            where
                Dyn<#arguments>: Pointee<Metadata = DynMetadata<Dyn<#arguments>>>,
                #where_predicates
                DynSliceFromType: 'static + #object_bounds,
            {
                // SAFETY:
                // DynMetadata contains a single pointer to the vtable, and the layout is the same as *const (),
                // so it can be transmuted.
                unsafe {
                    // Get the dyn metadata from the first element of value
                    // If value is empty, the metadata should never be accessed, so set it to a null pointer
                    let vtable_ptr = value.get(0).map_or(
                        null::<()>(),
                        |example| {
                            transmute(metadata(example as &Dyn<#arguments>))
                        }
                    );

                    DynSlice::with_vtable_ptr(value, vtable_ptr)
                }
            }

            #[allow(unused)]
            #[must_use]
            #[doc = concat!("Create a mutable dyn slice from a mutable slice of a type that implements [`", #trait_name, "`](", #trait_inner_path, ")" #(, "` + `[`", #auto_trait_names, "`](", #auto_trait_inner_paths, ")" )*)]
            pub fn new_mut<#full_generics DynSliceFromType>(value: &mut [DynSliceFromType]) -> SliceMut<'_, #arguments>
            where
                Dyn<#arguments>: Pointee<Metadata = DynMetadata<Dyn<#arguments>>>,
                #where_predicates
                DynSliceFromType: 'static + #object_bounds,
            {
                // SAFETY:
                // DynMetadata contains a single pointer to the vtable, and the layout is the same as *const (),
                // so it can be transmuted.
                unsafe {
                    // Get the dyn metadata from the first element of value
                    // If value is empty, the metadata should never be accessed, so set it to a null pointer
                    let vtable_ptr = value.get(0).map_or(
                        null::<()>(),
                        |example| {
                            transmute(metadata(example as &Dyn<#arguments>))
                        }
                    );

                    DynSliceMut::with_vtable_ptr(value, vtable_ptr)
                }
            }
        }
    }
}

fn remove_generic_bounds(
    generics: &Punctuated<GenericParam, Token![,]>,
) -> Punctuated<GenericParam, Token![,]> {
    let mut stripped_generics = generics.clone();

    for param in &mut stripped_generics {
        match param {
            GenericParam::Lifetime(lifetime) => {
                lifetime.bounds.clear();
            }
            GenericParam::Type(r#type) => {
                r#type.bounds.clear();
            }
            GenericParam::Const(_) => {}
        }
    }

    stripped_generics
}

fn get_arguments(
    generics: &Punctuated<GenericParam, Token![,]>,
) -> Punctuated<GenericArgument, Token![,]> {
    let mut arguments = Punctuated::new();

    for param in generics {
        let argument: GenericArgument = match param {
            GenericParam::Lifetime(lifetime) => {
                GenericArgument::Lifetime(lifetime.lifetime.clone())
            }
            GenericParam::Type(r#type) => GenericArgument::Type(
                TypePath {
                    qself: None,
                    path: r#type.ident.clone().into(),
                }
                .into(),
            ),
            GenericParam::Const(r#const) => GenericArgument::Const(
                ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path: r#const.ident.clone().into(),
                }
                .into(),
            ),
        };

        arguments.push(argument);
    }

    arguments
}
