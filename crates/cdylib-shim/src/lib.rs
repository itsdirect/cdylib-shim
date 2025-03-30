/// A macro for creating dynamic library shims that can intercept and modify function calls.
///
/// The `shim` macro allows you to create a library that acts as a drop-in replacement for an existing library,
/// while providing the ability to hook and modify the behavior of exported functions.
///
/// # Usage
///
/// The macro takes a library name as an argument and is applied to a module. Within this module, you can define:
///
/// - An initialization function marked with `#[init]` that runs when the library is loaded
/// - Hook functions marked with `#[hook]` that intercept calls to specific exported functions
///
/// The original library's functions are available through the automatically generated `original` module.
///
/// # Example
///
/// This example creates a shim for `version.dll` that logs whenever `GetFileVersionInfoA` is called:
///
/// ```rust
/// #![feature(naked_functions)]
///
/// use cdylib_shim::shim;
///
/// #[shim("version.dll")]
/// mod version {
///     use std::fs::File;
///
///     use winapi::shared::{
///         minwindef::{BOOL, DWORD, LPVOID},
///         ntdef::LPCSTR,
///     };
///
///     #[init]
///     fn init() {
///         let file = File::create("version.log").unwrap();
///
///         tracing_subscriber::fmt()
///             .with_writer(file)
///             .with_ansi(false)
///             .init();
///     }
///
///     #[hook]
///     unsafe extern "system" fn GetFileVersionInfoA(
///         lptstrFileName: LPCSTR,
///         dwHandle: DWORD,
///         dwLen: DWORD,
///         lpData: LPVOID,
///     ) -> BOOL {
///         tracing::info!("Hello from GetFileVersionInfoA!");
///         unsafe { original::GetFileVersionInfoA(lptstrFileName, dwHandle, dwLen, lpData) }
///     }
/// }
/// ```
pub use cdylib_shim_macros::shim;

#[doc(hidden)]
#[path = "private.rs"]
pub mod __private;
