mod config;

use std::collections::HashSet;

use proc_macro::TokenStream;
use quote::quote;
use shim_core::Library;
use syn::{parse_macro_input, TypeBareFn};

#[proc_macro]
pub fn shim(input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(input as config::Config);
    let library_name = config.library.expect("library not specified");
    let functions = config.functions.expect("functions not specified");
    let library = Library::load_system(&library_name).expect("library not found");
    let all_functions = library.all().expect("failed to get all exports");

    let defined_functions = HashSet::<String>::from_iter(functions.iter().map(|f| f.0.to_string()));

    for function in all_functions.iter() {
        if !defined_functions.contains(function.as_str()) {
            panic!("function {} not defined", function);
        }
    }

    let load = config.load.map(|load| {
        quote! {
            #load();
        }
    });

    let unload = config.unload.map(|load| {
        quote! {
            #load();
        }
    });

    let container_functions = functions.iter().map(|(ident, bare_fn)| {
        quote! {
            #ident: #bare_fn
        }
    });

    let load_functions = functions.iter().map(|(ident, _)| {
        let name = ident.to_string();

        quote! {
            #ident: library.get(#name)?,
        }
    });

    let container = quote! {
        struct Functions {
            #(#container_functions),*
        }

        impl Functions {
            unsafe fn load() -> Option<Self> {
                let library = shim::Library::load_system(#library_name)?;

                let functions = Self {
                    #(#load_functions)*
                };

                Some(functions)
            }
        }

        static FUNCTIONS: Lazy<Functions> = Lazy::new(|| unsafe {
            Functions::load().unwrap()
        });
    };

    let exports = functions.iter().map(|(ident, bare_fn)| {
        let TypeBareFn {
            unsafety,
            abi,
            inputs,
            output,
            ..
        } = bare_fn;

        let args = inputs
            .iter()
            .filter_map(|a| a.name.as_ref())
            .map(|(i, _)| i);

        quote! {
            #[no_mangle]
            #unsafety #abi fn #ident(#inputs) #output {
                (FUNCTIONS.#ident)(#(#args),*)
            }
        }
    });

    let expanded = quote! {
        use shim::entry::*;

        #container
        #(#exports)*

        #[no_mangle]
        extern "system" fn DllMain(_: HINSTANCE, fdwReason: DWORD, _: LPVOID) -> BOOL {
            match fdwReason {
                DLL_PROCESS_ATTACH => {
                    #load
                }
                DLL_PROCESS_DETACH => {
                    #unload
                }
                _ => {}
            }

            TRUE
        }
    };

    expanded.into()
}
