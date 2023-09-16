mod config;

use std::collections::HashSet;

use convert_case::{Case, Casing};
use proc_macro::{Span, TokenStream};
use quote::quote;
use shim_core::Library;
use syn::{parse_macro_input, Ident};

#[proc_macro]
pub fn shim(input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(input as config::Config);
    let library_name = config.library.expect("library not specified");

    let all_functions = unsafe {
        let library = Library::load_system(&library_name).expect("library not found");
        library.all()
    };

    let include = config.include.map(HashSet::<String>::from_iter);
    let exclude = config.exclude.map(HashSet::<String>::from_iter);

    let exported_functions: Vec<_> = all_functions
        .iter()
        .filter(|f| include.as_ref().map(|i| i.contains(*f)).unwrap_or(true))
        .filter(|f| exclude.as_ref().map(|e| !e.contains(*f)).unwrap_or(true))
        .collect();

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

    let statics = all_functions.iter().map(|f| {
        let static_name = Ident::new(&f.to_case(Case::ScreamingSnake), Span::call_site().into());

        quote! {
            pub static mut #static_name: usize = 0;
        }
    });

    let exports = exported_functions.iter().map(|f| {
        let name = Ident::new(f.as_str(), Span::call_site().into());
        let static_name = Ident::new(&f.to_case(Case::ScreamingSnake), Span::call_site().into());

        quote! {
            #[no_mangle]
            #[naked]
            unsafe extern "C" fn #name() {
                std::arch::asm!("jmp [rip + {}]", sym #static_name, options(noreturn))
            }
        }
    });

    let load_statics = all_functions.iter().map(|f| {
        let name = f.as_str();
        let static_name = Ident::new(&f.to_case(Case::ScreamingSnake), Span::call_site().into());

        quote! {
            #static_name = library.get(#name)?;
        }
    });

    let expanded = quote! {
        mod exports {
            #(#statics)*
            #(#exports)*

            fn load() -> Option<()> {
                unsafe {
                    let library = shim::Library::load_system(#library_name)?;
                    #(#load_statics)*
                    Some(())
                }
            }

            use shim::entry::*;

            #[no_mangle]
            extern "system" fn DllMain(_: HINSTANCE, fdwReason: DWORD, _: LPVOID) -> BOOL {
                match fdwReason {
                    DLL_PROCESS_ATTACH => {
                        super::exports::load().unwrap();
                        #load
                    }
                    DLL_PROCESS_DETACH => {
                        #unload
                    }
                    _ => {}
                }

                TRUE
            }
        }
    };

    expanded.into()
}
