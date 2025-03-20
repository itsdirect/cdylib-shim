use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{
    Attribute, Error, Ident, Item, ItemFn, ItemMod, LitStr, parse_macro_input, parse_quote,
    spanned::Spanned,
};

#[proc_macro_attribute]
pub fn shim(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _library = parse_macro_input!(attr as LitStr);
    let mut module = parse_macro_input!(item as ItemMod);

    let mut ctx = Context {
        hook_fns: Vec::new(),
        load_fn: None,
        unload_fn: None,
    };

    let Some((_, content)) = &mut module.content else {
        return module.into_token_stream().into();
    };

    for item in content.iter_mut() {
        let result = match item {
            Item::Fn(item_fn) => handle_item_fn(&mut ctx, item_fn),
            _ => Ok(()),
        };

        if let Err(errors) = result {
            return errors
                .into_iter()
                .map(|error| TokenStream::from(error.to_compile_error()))
                .collect();
        }
    }

    module.into_token_stream().into()
}

struct Context {
    hook_fns: Vec<Ident>,
    load_fn: Option<Ident>,
    unload_fn: Option<Ident>,
}

fn handle_item_fn(ctx: &mut Context, item_fn: &mut ItemFn) -> Result<(), Vec<Error>> {
    let (special_attrs, attrs) = std::mem::take(&mut item_fn.attrs)
        .into_iter()
        .partition::<Vec<_>, _>(is_special_attr);

    item_fn.attrs = attrs;

    let [special_attr, excess_attrs @ ..] = special_attrs.as_slice() else {
        return Ok(());
    };

    if !excess_attrs.is_empty() {
        return Err(excess_attrs
            .iter()
            .map(|attr| {
                Error::new(
                    attr.span(),
                    "Only one of `load`, `unload`, or `hook` attributes are allowed per function",
                )
            })
            .collect());
    }

    if let Some(ident) = special_attr.path().get_ident() {
        if ident == "hook" {
            item_fn.attrs.push(parse_quote!(#[unsafe(no_mangle)]));
            ctx.hook_fns.push(item_fn.sig.ident.clone());
        } else if ident == "load" {
            item_fn.attrs.push(parse_quote!(#[allow(dead_code)]));
            ctx.load_fn = Some(item_fn.sig.ident.clone());
        } else if ident == "unload" {
            item_fn.attrs.push(parse_quote!(#[allow(dead_code)]));
            ctx.unload_fn = Some(item_fn.sig.ident.clone());
        }
    }

    Ok(())
}

fn is_special_attr(attr: &Attribute) -> bool {
    let Some(ident) = attr.path().get_ident() else {
        return false;
    };

    ident == "hook" || ident == "load" || ident == "unload"
}
