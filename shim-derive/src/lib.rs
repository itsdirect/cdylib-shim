use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Data, DataStruct, DeriveInput, Ident, Lit, Meta, NestedMeta, Type,
    TypeBareFn,
};

#[proc_macro_derive(Shim, attributes(shim))]
pub fn derive_shim(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let library_name = get_library_name(&input);
    let struct_name = &input.ident;
    let struct_name_uppercase =
        Ident::new(&struct_name.to_string().to_uppercase(), struct_name.span());
    let data = get_struct(&input);

    let get_functions = data.fields.iter().map(|f| {
        let ident = f.ident.as_ref().unwrap();

        quote! {
            #ident: library.get(stringify!(#ident))?
        }
    });

    let exports = data.fields.iter().map(|f| {
        let ident = f.ident.as_ref().unwrap();
        let bare_fn = get_bare_fn(&f.ty);
        let abi = bare_fn.abi.as_ref().unwrap();

        let inputs = &bare_fn.inputs;
        let output = &bare_fn.output;

        let args = bare_fn.inputs.iter().map(|i| {
            let (name, _) = i.name.as_ref().unwrap();
            quote! { #name }
        });

        quote! {
            #[no_mangle]
            #abi fn #ident(#inputs) #output {
                (#struct_name_uppercase.#ident)(#(#args,)*)
            }
        }
    });

    let expanded = quote! {
        impl #struct_name {
            fn load() -> std::option::Option<Self> {
                unsafe {
                    let library = shim::Library::load_system(#library_name)?;
                    Some(Self {
                        #(#get_functions,)*
                    })
                }
            }
        }

        static #struct_name_uppercase: shim::Lazy<#struct_name> = shim::Lazy::new(|| #struct_name::load().unwrap());

        #(#exports)*
    };

    expanded.into()
}

fn get_library_name(input: &DeriveInput) -> String {
    for attr in &input.attrs {
        if let Some(segment) = attr.path.segments.first() {
            if segment.ident == "shim" {
                if let Ok(Meta::List(list)) = attr.parse_meta() {
                    if let Some(NestedMeta::Lit(Lit::Str(lit))) = list.nested.first() {
                        return lit.value();
                    }
                }
            }
        }
    }

    panic!("Library name not specified. Example: #[shim(\"version.dll\")]")
}

fn get_struct(input: &DeriveInput) -> &DataStruct {
    if let Data::Struct(ref data) = input.data {
        return data;
    }

    panic!("Only valid on structs");
}

fn get_bare_fn(ty: &Type) -> &TypeBareFn {
    if let Type::BareFn(bare_fn) = ty {
        return bare_fn;
    }

    panic!("Only valid on functions");
}
