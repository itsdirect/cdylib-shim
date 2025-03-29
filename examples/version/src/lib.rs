#![feature(naked_functions)]

use cdylib_shim::shim;

#[shim("version.dll")]
mod version {
    use std::{
        fs::File,
        io::Write,
        sync::{LazyLock, Mutex},
    };

    use winapi::shared::{
        minwindef::{BOOL, DWORD, LPVOID},
        ntdef::LPCSTR,
    };

    struct Logger(Mutex<File>);

    impl Logger {
        fn new(name: &str) -> Self {
            Self(Mutex::new(File::create(name).unwrap()))
        }

        fn log(&self, message: &str) {
            let mut file = self.0.lock().unwrap();
            writeln!(file, "{}", message).unwrap();
        }
    }

    static LOGGER: LazyLock<Logger> = LazyLock::new(|| Logger::new("version.txt"));

    #[init]
    fn init() {
        LOGGER.log("Hello from init!");
    }

    #[hook]
    unsafe extern "system" fn GetFileVersionInfoA(
        lptstrFileName: LPCSTR,
        dwHandle: DWORD,
        dwLen: DWORD,
        lpData: LPVOID,
    ) -> BOOL {
        LOGGER.log("Hello from GetFileVersionInfoA!");
        unsafe { original::GetFileVersionInfoA(lptstrFileName, dwHandle, dwLen, lpData) }
    }
}
