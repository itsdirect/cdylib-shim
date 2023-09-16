mod library;

use crate::library::Library;
use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Ident, LitStr};

#[proc_macro]
pub fn shim(input: TokenStream) -> TokenStream {
    let library_name = parse_macro_input!(input as LitStr).value();
    let library = Library::load_system(library_name.as_str()).expect("library not found");
    let functions = library.all();

    let statics = functions.iter().map(|f| {
        let static_name = Ident::new(&f.to_case(Case::ScreamingSnake), library_name.span());

        quote! {
            static mut #static_name: usize = 0;
        }
    });

    let exports = functions.iter().map(|f| {
        let name = Ident::new(f.as_str(), library_name.span());
        let static_name = Ident::new(&f.to_case(Case::ScreamingSnake), library_name.span());

        quote! {
            #[no_mangle]
            #[naked]
            unsafe extern "C" fn #name() {
                std::arch::asm!("jmp [rip + {}]", sym #static_name, options(noreturn))
            }
        }
    });

    let load_statics = functions.iter().map(|f| {
        let name = f.as_str();
        let static_name = Ident::new(&f.to_case(Case::ScreamingSnake), library_name.span());

        quote! {
            #static_name = library.get(#name)?;
        }
    });

    let expanded = quote! {
        mod exports {
            #(#statics)*
            #(#exports)*

            pub fn load() -> Option<()> {
                unsafe {
                    let library = shim::Library::load_system(#library_name)?;
                    #(#load_statics)*
                    Some(())
                }
            }
        }

        mod entry {
            use shim::entry::*;

            #[no_mangle]
            extern "system" fn DllMain(_: HINSTANCE, fdwReason: DWORD, _: LPVOID) -> BOOL {
                match fdwReason {
                    DLL_PROCESS_ATTACH => super::exports::load().unwrap(),
                    _ => {}
                }

                TRUE
            }
        }
    };

    expanded.into()
}
