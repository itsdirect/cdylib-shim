use std::{collections::HashSet, path::Path, str::Utf8Error};

use convert_case::{Case, Casing};
use itertools::{Either, Itertools};
use object::Object;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Attribute, Error, FnArg, Ident, Item, ItemFn, ItemMod, LitStr, PatType, Signature,
    parse_macro_input, parse_quote, spanned::Spanned,
};

#[proc_macro_attribute]
pub fn shim(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let name = parse_macro_input!(attr as LitStr);

    let Some(library) = Library::load(name.value()) else {
        panic!("Failed to load library");
    };

    let mut module = parse_macro_input!(item as ItemMod);

    let mut ctx = Context {
        library,
        init_fn: None,
        hook_fns: Vec::new(),
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
                .map(|error| error.to_compile_error())
                .collect::<TokenStream>()
                .into();
        }
    }

    content.push({
        let original_mod = OriginalModule { ctx: &ctx };
        parse_quote! { #original_mod }
    });

    module.into_token_stream().into()
}

struct Context {
    library: Library,
    init_fn: Option<InitFn>,
    hook_fns: Vec<HookFn>,
}

fn handle_item_fn(ctx: &mut Context, item_fn: &mut ItemFn) -> Result<(), Vec<Error>> {
    let Some((kind, attr)) = parse_attrs(item_fn)? else {
        return Ok(());
    };

    match kind {
        AttributeKind::Init => handle_init_fn(ctx, item_fn, &attr),
        AttributeKind::Hook => handle_hook_fn(ctx, item_fn),
    }
}

fn parse_attrs(item_fn: &mut ItemFn) -> Result<Option<(AttributeKind, Attribute)>, Vec<Error>> {
    let (parsed_attrs, attrs): (Vec<_>, Vec<_>) = std::mem::take(&mut item_fn.attrs)
        .into_iter()
        .partition_map(|attr| match AttributeKind::try_from(&attr) {
            Ok(kind) => Either::Left((kind, attr)),
            Err(_) => Either::Right(attr),
        });

    item_fn.attrs = attrs;
    let mut parsed_attrs = parsed_attrs.into_iter();

    let Some(parsed_attr) = parsed_attrs.next() else {
        return Ok(None);
    };

    let errors: Vec<_> = parsed_attrs
        .map(|(_, attr)| {
            Error::new(
                attr.span(),
                "Only one `init` or `hook` attribute is allowed per function",
            )
        })
        .collect();

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(Some(parsed_attr))
}

fn handle_init_fn(
    ctx: &mut Context,
    item_fn: &mut ItemFn,
    attr: &Attribute,
) -> Result<(), Vec<Error>> {
    if ctx.init_fn.is_some() {
        return Err(vec![Error::new(
            attr.span(),
            "There can only be one `init` function",
        )]);
    }

    item_fn.attrs.push(parse_quote!(#[allow(dead_code)]));

    ctx.init_fn = Some(InitFn {
        sig: item_fn.sig.clone(),
    });

    Ok(())
}

fn handle_hook_fn(ctx: &mut Context, item_fn: &mut ItemFn) -> Result<(), Vec<Error>> {
    let export = item_fn.sig.ident.to_string().as_str().into();

    if !ctx.library.exports.contains(&export) {
        return Err(vec![Error::new(
            item_fn.sig.ident.span(),
            format!("Function is not an exported symbol in {}", ctx.library.name),
        )]);
    }

    item_fn.attrs.push(parse_quote!(#[unsafe(no_mangle)]));
    item_fn.attrs.push(parse_quote!(#[allow(non_snake_case)]));

    ctx.hook_fns.push(HookFn {
        sig: item_fn.sig.clone(),
        export,
    });

    Ok(())
}

enum AttributeKind {
    Init,
    Hook,
}

impl TryFrom<&Attribute> for AttributeKind {
    type Error = ();

    fn try_from(value: &Attribute) -> Result<Self, Self::Error> {
        if value.path().is_ident("init") {
            Ok(Self::Init)
        } else if value.path().is_ident("hook") {
            Ok(Self::Hook)
        } else {
            Err(())
        }
    }
}

struct Library {
    name: String,
    exports: HashSet<Export>,
}

impl Library {
    fn load(name: String) -> Option<Self> {
        let separator = if cfg!(windows) { ';' } else { ':' };

        let path = std::env::var("PATH")
            .ok()?
            .split(separator)
            .map(|directory| Path::new(directory).join(&name))
            .find(|path| path.exists())?;

        let data = std::fs::read(&path).ok()?;

        let exports = object::File::parse(data.as_slice())
            .ok()?
            .exports()
            .ok()?
            .into_iter()
            .filter_map(|export| Export::try_from(&export).ok())
            .collect();

        Some(Self { name, exports })
    }

    fn lit_str(&self, span: Span) -> LitStr {
        LitStr::new(&self.name, span)
    }
}

#[derive(PartialEq, Eq, Hash)]
struct Export {
    name: String,
}

impl Export {
    fn ident(&self, span: Span) -> Ident {
        Ident::new(&self.name, span)
    }

    fn lit_str(&self, span: Span) -> LitStr {
        LitStr::new(&self.name, span)
    }

    fn address(&self) -> ExportAddress {
        ExportAddress { export: self }
    }
}

impl From<&str> for Export {
    fn from(value: &str) -> Self {
        Self { name: value.into() }
    }
}

impl TryFrom<&object::Export<'_>> for Export {
    type Error = Utf8Error;

    fn try_from(value: &object::Export) -> Result<Self, Self::Error> {
        std::str::from_utf8(value.name()).map(Into::into)
    }
}

struct ExportAddress<'a> {
    export: &'a Export,
}

impl ExportAddress<'_> {
    fn ident(&self, span: Span) -> Ident {
        Ident::new(
            &format!("{}_ADDRESS", self.export.name.to_case(Case::UpperSnake)),
            span,
        )
    }
}

impl ToTokens for ExportAddress<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = self.ident(Span::call_site());

        tokens.extend(quote! {
            static mut #ident: usize = 0;
        });
    }
}

struct ShimFn<'a> {
    export: &'a Export,
}

impl ToTokens for ShimFn<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = self.export.ident(Span::call_site());
        let address_ident = self.export.address().ident(Span::call_site());

        tokens.extend(quote! {
            #[naked]
            #[unsafe(no_mangle)]
            unsafe extern "system" fn #ident() {
                std::arch::naked_asm!("jmp [rip + {}]", sym #address_ident)
            }
        });
    }
}

struct LoadLibraryFn<'a> {
    library: &'a Library,
}

impl LoadLibraryFn<'_> {
    fn ident(&self, span: Span) -> Ident {
        Ident::new("load_library", span)
    }

    fn to_call_tokens(&self) -> TokenStream {
        let ident = self.ident(Span::call_site());
        quote! { #ident() }
    }
}

impl ToTokens for LoadLibraryFn<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = self.ident(Span::call_site());
        let library_name = self.library.lit_str(Span::call_site());

        let load_exports = self.library.exports.iter().map(|export| {
            let address_ident = export.address().ident(Span::call_site());
            let export_name = export.lit_str(Span::call_site());
            quote! { #address_ident = library.get(#export_name).unwrap().addr().get(); }
        });

        tokens.extend(quote! {
            fn #ident() {
                unsafe {
                    static mut LIBRARY: Option<cdylib_shim::__private::Library> = None;
                    let library = LIBRARY.insert(cdylib_shim::__private::Library::load_system(#library_name).unwrap());
                    #(#load_exports)*
                }
            }
        });
    }
}

struct InitFn {
    sig: Signature,
}

impl InitFn {
    fn to_call_tokens(&self) -> TokenStream {
        let ident = &self.sig.ident;
        quote! { #ident() }
    }
}

struct HookFn {
    sig: Signature,
    export: Export,
}

impl HookFn {
    fn to_original_fn(&self) -> OriginalFn {
        OriginalFn { hook_fn: self }
    }
}

struct OriginalFn<'a> {
    hook_fn: &'a HookFn,
}

impl ToTokens for OriginalFn<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let HookFn { sig, export } = self.hook_fn;
        let abi = &sig.abi;
        let output = &sig.output;
        let address_ident = export.address().ident(Span::call_site());

        let (pats, tys): (Vec<_>, Vec<_>) = sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                FnArg::Typed(PatType { pat, ty, .. }) => Some((pat, ty)),
                FnArg::Receiver(_) => None,
            })
            .collect();

        tokens.extend(quote! {
            #[allow(non_snake_case)]
            pub #sig {
                unsafe {
                    std::mem::transmute::<_, #abi fn(#(#tys),*) #output>(#address_ident)(#(#pats),*)
                }
            }
        })
    }
}

struct Initializer<'a> {
    load_library_fn: &'a LoadLibraryFn<'a>,
    init_fn: Option<&'a InitFn>,
}

impl ToTokens for Initializer<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let load_library_fn_call = self.load_library_fn.to_call_tokens();
        let init_fn_call = self.init_fn.map(|init_fn| {
            let tokens = init_fn.to_call_tokens();
            quote! { super::#tokens; }
        });

        tokens.extend(quote! {
            #[used]
            #[unsafe(link_section = ".CRT$XCU")]
            static INITIALIZER: extern "C" fn() = {
                extern "C" fn init() {
                    #load_library_fn_call;
                    #init_fn_call;
                }
                init
            };
        });
    }
}

struct OriginalModule<'a> {
    ctx: &'a Context,
}

impl ToTokens for OriginalModule<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let export_addresses = self.ctx.library.exports.iter().map(Export::address);
        let original_fns = self.ctx.hook_fns.iter().map(HookFn::to_original_fn);

        let hook_exports: HashSet<_> = self
            .ctx
            .hook_fns
            .iter()
            .map(|hook_fn| &hook_fn.export)
            .collect();

        let shim_fns = self
            .ctx
            .library
            .exports
            .iter()
            .filter(|export| !hook_exports.contains(export))
            .map(|export| ShimFn { export });

        let load_library_fn = LoadLibraryFn {
            library: &self.ctx.library,
        };

        let initializer = Initializer {
            load_library_fn: &load_library_fn,
            init_fn: self.ctx.init_fn.as_ref(),
        };

        tokens.extend(quote! {
            mod original {
                use super::*;

                #(#export_addresses)*
                #(#original_fns)*
                #(#shim_fns)*
                #load_library_fn
                #initializer
            }
        })
    }
}
