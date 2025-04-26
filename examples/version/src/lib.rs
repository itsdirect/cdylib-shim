use cdylib_shim::shim;

#[shim("version.dll")]
mod version {
    use std::{
        ffi::{c_char, c_int, c_ulong, c_void},
        fs::File,
    };

    #[init]
    fn init() {
        let file = File::create("version.log").unwrap();

        tracing_subscriber::fmt()
            .with_writer(file)
            .with_ansi(false)
            .init();
    }

    #[hook]
    unsafe extern "system" fn GetFileVersionInfoA(
        lptstrFileName: *const c_char,
        dwHandle: c_ulong,
        dwLen: c_ulong,
        lpData: *const c_void,
    ) -> c_int {
        tracing::info!("Hello from GetFileVersionInfoA!");
        unsafe { original::GetFileVersionInfoA(lptstrFileName, dwHandle, dwLen, lpData) }
    }
}
