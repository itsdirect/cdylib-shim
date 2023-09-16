#[allow(non_snake_case)]
mod exports {
    fn load() {
        std::fs::write("test.log", "Hello world!").unwrap();
    }

    shim::shim! {
        library: "version.dll",
        load,
        functions: [
            GetFileVersionInfoA: unsafe extern "C" fn(
                lptstrFilename: *const u8,
                dwHandle: u32,
                dwLen: u32,
                lpData: *mut u8,
            ) -> u32,

            GetFileVersionInfoByHandle: unsafe extern "C" fn(
                hFile: *mut u8,
                dwHandle: u32,
                dwLen: u32,
                lpData: *mut u8,
            ) -> u32,

            GetFileVersionInfoExA: unsafe extern "C" fn(
                dwFlags: u32,
                lpwstrFilename: *const u8,
                dwHandle: u32,
                dwLen: u32,
                lpData: *mut u8,
            ) -> u32,

            GetFileVersionInfoExW: unsafe extern "C" fn(
                dwFlags: u32,
                lpwstrFilename: *const u16,
                dwHandle: u32,
                dwLen: u32,
                lpData: *mut u8,
            ) -> u32,

            GetFileVersionInfoSizeA: unsafe extern "C" fn(
                lptstrFilename: *const u8,
                lpdwHandle: *mut u32,
            ) -> u32,

            GetFileVersionInfoSizeExA: unsafe extern "C" fn(
                dwFlags: u32,
                lpwstrFilename: *const u8,
                lpdwHandle: *mut u32,
            ) -> u32,

            GetFileVersionInfoSizeExW: unsafe extern "C" fn(
                dwFlags: u32,
                lpwstrFilename: *const u16,
                lpdwHandle: *mut u32,
            ) -> u32,

            GetFileVersionInfoSizeW: unsafe extern "C" fn(
                lptstrFilename: *const u16,
                lpdwHandle: *mut u32,
            ) -> u32,

            GetFileVersionInfoW: unsafe extern "C" fn(
                lptstrFilename: *const u16,
                dwHandle: u32,
                dwLen: u32,
                lpData: *mut u8,
            ) -> u32,

            VerFindFileA: unsafe extern "C" fn(
                uFlags: u32,
                szFileName: *const u8,
                szWinDir: *const u8,
                szAppDir: *const u8,
                szCurDir: *const u8,
                puCurDirLen: *mut u32,
                szDestDir: *mut u8,
                puDestDirLen: *mut u32,
            ) -> u32,

            VerFindFileW: unsafe extern "C" fn(
                uFlags: u32,
                szFileName: *const u16,
                szWinDir: *const u16,
                szAppDir: *const u16,
                szCurDir: *const u16,
                puCurDirLen: *mut u32,
                szDestDir: *mut u16,
                puDestDirLen: *mut u32,
            ) -> u32,

            VerInstallFileA: unsafe extern "C" fn(
                uFlags: u32,
                szSrcFileName: *const u8,
                szDestFileName: *const u8,
                szSrcDir: *const u8,
                szDestDir: *const u8,
                szCurDir: *const u8,
                szTmpFile: *mut u8,
                puTmpFileLen: *mut u32,
            ) -> u32,

            VerInstallFileW: unsafe extern "C" fn(
                uFlags: u32,
                szSrcFileName: *const u16,
                szDestFileName: *const u16,
                szSrcDir: *const u16,
                szDestDir: *const u16,
                szCurDir: *const u16,
                szTmpFile: *mut u16,
                puTmpFileLen: *mut u32,
            ) -> u32,

            VerQueryValueA: unsafe extern "C" fn(
                pBlock: *const u8,
                lpSubBlock: *const u8,
                lplpBuffer: *mut *mut u8,
                puLen: *mut u32,
            ) -> u32,

            VerQueryValueW: unsafe extern "C" fn(
                pBlock: *const u8,
                lpSubBlock: *const u16,
                lplpBuffer: *mut *mut u8,
                puLen: *mut u32,
            ) -> u32,
        ]
    }
}
