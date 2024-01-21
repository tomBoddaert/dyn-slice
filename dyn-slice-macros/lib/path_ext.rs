//! The entire idea of this module is to try to add `super::` before any
//! paths that might need it.
//! This is useful when trying to access paths from a macro call within
//! a module.

use proc_macro2::Span;
use syn::{
    spanned::Spanned, AssocConst, AssocType, ConstParam, Constraint, Expr, ExprPath,
    GenericArgument, GenericParam, Generics, Ident, Macro, ParenthesizedGenericArguments, Path,
    PathArguments, PredicateType, QSelf, ReturnType, Type, TypeArray, TypeBareFn, TypeMacro,
    TypeParen, TypePath, TypePtr, TypeReference, TypeSlice, TypeTraitObject, TypeTuple,
    WherePredicate,
};

use crate::type_param_bound_select_trait;

pub const RESERVED: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "char", "bool", "f64", "core", "alloc", "std",
];

pub fn make_inner_path(path: &mut Path, generic_idents: &[String]) -> syn::Result<()> {
    path.segments.iter_mut().try_for_each(|segment| {
        make_inner_path_arguments(&mut segment.arguments, generic_idents)
    })?;

    // If the path starts with ::, do nothing
    if r#path.leading_colon.is_some() {
        return Ok(());
    }

    if r#path.segments.len() == 1
        && generic_idents
            .iter()
            .any(|generic| r#path.is_ident(generic))
    {
        return Ok(());
    }

    let path_span = path.span();
    let first = path
        .segments
        .first_mut()
        .ok_or_else(|| syn::Error::new(path_span, "empty path"))?;

    // If the path is the same as a generic ident or primative, do nothing
    if generic_idents.iter().any(|generic| first.ident == generic) {
        return Ok(());
    }

    let call_site = first.ident.span();

    // If the path starts with crate, skip it
    if first.ident == Ident::new("crate", call_site) {
        return Ok(());
    }

    // If the path starts with self, change self to super
    if first.ident == Ident::new("self", call_site) {
        first.ident = Ident::new("super", call_site);
        return Ok(());
    }

    // Otherwise, prefix the trait with super
    path.segments
        .insert(0, Ident::new("super", Span::call_site()).into());

    Ok(())
}

pub fn make_inner_path_arguments(
    arguments: &mut PathArguments,
    generic_idents: &[String],
) -> syn::Result<()> {
    match arguments {
        PathArguments::None => Ok(()),

        PathArguments::AngleBracketed(arguments) => arguments
            .args
            .iter_mut()
            .try_for_each(|arg| make_inner_path_generic_argument(arg, generic_idents)),

        PathArguments::Parenthesized(ParenthesizedGenericArguments { inputs, output, .. }) => {
            if let ReturnType::Type(_, r#type) = output {
                make_inner_path_type(r#type, generic_idents)?;
            }

            inputs
                .iter_mut()
                .try_for_each(|r#type| make_inner_path_type(r#type, generic_idents))
        }
    }
}

pub fn make_inner_path_generic_argument(
    argument: &mut GenericArgument,
    generic_idents: &[String],
) -> syn::Result<()> {
    match argument {
        GenericArgument::Type(r#type) => make_inner_path_type(r#type, generic_idents),

        // Only expand const paths because the alternative is too complex
        GenericArgument::Const(Expr::Path(ExprPath { qself, path, .. })) => {
            if let Some(QSelf { ty, .. }) = qself {
                make_inner_path_type(ty, generic_idents)?;
            }

            make_inner_path(path, generic_idents)
        }

        GenericArgument::AssocType(AssocType {
            generics: generic_arguments,
            ty,
            ..
        }) => {
            if let Some(arguments) = generic_arguments {
                arguments
                    .args
                    .iter_mut()
                    .try_for_each(|arg| make_inner_path_generic_argument(arg, generic_idents))?;
            }

            make_inner_path_type(ty, generic_idents)
        }

        GenericArgument::AssocConst(AssocConst {
            generics: generic_arguments,
            value,
            ..
        }) => {
            if let Some(arguments) = generic_arguments {
                arguments
                    .args
                    .iter_mut()
                    .try_for_each(|arg| make_inner_path_generic_argument(arg, generic_idents))?;
            }

            if let Expr::Path(ExprPath { qself, path, .. }) = value {
                if let Some(QSelf { ty, .. }) = qself {
                    make_inner_path_type(ty, generic_idents)?;
                }

                make_inner_path(path, generic_idents)?;
            }

            Ok(())
        }

        GenericArgument::Constraint(Constraint {
            generics: generic_arguments,
            bounds,
            ..
        }) => {
            if let Some(arguments) = generic_arguments {
                arguments
                    .args
                    .iter_mut()
                    .try_for_each(|arg| make_inner_path_generic_argument(arg, generic_idents))?;
            }

            bounds
                .iter_mut()
                .filter_map(type_param_bound_select_trait)
                .try_for_each(|bound| make_inner_path(&mut bound.path, generic_idents))
        }
        _ => Ok(()),
    }
}

pub fn make_inner_path_type(r#type: &mut Type, generic_idents: &[String]) -> syn::Result<()> {
    match r#type {
        Type::Array(TypeArray { elem, len, .. }) => {
            make_inner_path_type(elem, generic_idents)?;

            // Only expand const paths because the alternative is too complex
            if let Expr::Path(ExprPath { qself, path, .. }) = len {
                if let Some(QSelf { ty, .. }) = qself {
                    make_inner_path_type(ty, generic_idents)?;
                }

                make_inner_path(path, generic_idents)?;
            }

            Ok(())
        }

        Type::BareFn(TypeBareFn { inputs, output, .. }) => {
            inputs
                .iter_mut()
                .map(|input| &mut input.ty)
                .try_for_each(|r#type| make_inner_path_type(r#type, generic_idents))?;

            if let ReturnType::Type(_, r#type) = output {
                make_inner_path_type(r#type, generic_idents)?;
            }

            Ok(())
        }

        Type::Macro(TypeMacro {
            mac: Macro { path, .. },
        }) => make_inner_path(path, generic_idents),

        Type::Paren(TypeParen { elem, .. })
        | Type::Ptr(TypePtr { elem, .. })
        | Type::Reference(TypeReference { elem, .. })
        | Type::Slice(TypeSlice { elem, .. }) => make_inner_path_type(elem, generic_idents),

        Type::Path(TypePath { qself, path }) => {
            if let Some(QSelf { ty, .. }) = qself {
                make_inner_path_type(ty, generic_idents)?;
            }

            make_inner_path(path, generic_idents)
        }

        Type::TraitObject(TypeTraitObject { bounds, .. }) => bounds
            .iter_mut()
            .filter_map(type_param_bound_select_trait)
            .try_for_each(|bound| make_inner_path(&mut bound.path, generic_idents)),

        Type::Tuple(TypeTuple { elems, .. }) => elems
            .iter_mut()
            .try_for_each(|r#type| make_inner_path_type(r#type, generic_idents)),

        _ => Ok(()),
    }
}

pub fn make_generics_inner_path(
    generics: &mut Generics,
    generic_idents: &[String],
) -> syn::Result<()> {
    for param in &mut generics.params {
        match param {
            GenericParam::Lifetime(_) => {}

            GenericParam::Type(r#type) => r#type
                .bounds
                .iter_mut()
                .filter_map(type_param_bound_select_trait)
                .try_for_each(|bound| make_inner_path(&mut bound.path, generic_idents))?,

            GenericParam::Const(ConstParam { ty, default, .. }) => {
                make_inner_path_type(ty, generic_idents)?;

                // Only expand const paths because the alternative is too complex
                if let Some(Expr::Path(ExprPath { qself, path, .. })) = default {
                    if let Some(QSelf { ty, .. }) = qself {
                        make_inner_path_type(ty, generic_idents)?;
                    }

                    make_inner_path(path, generic_idents)?;
                }
            }
        }
    }

    if let Some(where_clause) = &mut generics.where_clause {
        for PredicateType {
            bounded_ty, bounds, ..
        } in where_clause.predicates.iter_mut().filter_map(|predicate| {
            if let WherePredicate::Type(predicate) = predicate {
                Some(predicate)
            } else {
                None
            }
        }) {
            make_inner_path_type(bounded_ty, generic_idents)?;

            for bound in bounds.iter_mut().filter_map(type_param_bound_select_trait) {
                make_inner_path(&mut bound.path, generic_idents)?;
            }
        }
    }

    Ok(())
}
