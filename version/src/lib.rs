#![allow(non_snake_case)]

use shim::Shim;
use winapi::ctypes::c_void;
use winapi::shared::minwindef::{BOOL, DWORD, LPCVOID, LPDWORD, LPVOID, PUINT};
use winapi::shared::ntdef::{LPCSTR, LPCWSTR, LPSTR, LPWSTR};

#[derive(Shim)]
#[shim("version.dll")]
struct Functions {
    GetFileVersionInfoSizeA:
        extern "system" fn(lptstrFilename: LPCSTR, lpdwHandle: *mut DWORD) -> DWORD,
    GetFileVersionInfoSizeW:
        extern "system" fn(lptstrFilename: LPCWSTR, lpdwHandle: *mut DWORD) -> DWORD,
    GetFileVersionInfoSizeExA:
        extern "system" fn(dwFlags: DWORD, lpwstrFilename: LPCSTR, lpdwHandle: LPDWORD) -> DWORD,
    GetFileVersionInfoSizeExW:
        extern "system" fn(dwFlags: DWORD, lpwstrFilename: LPCWSTR, lpdwHandle: LPDWORD) -> DWORD,
    GetFileVersionInfoA: extern "system" fn(
        lptstrFilename: LPCSTR,
        dwHandle: DWORD,
        dwLen: DWORD,
        lpData: *mut c_void,
    ) -> BOOL,
    GetFileVersionInfoW: extern "system" fn(
        lptstrFilename: LPCWSTR,
        dwHandle: DWORD,
        dwLen: DWORD,
        lpData: *mut c_void,
    ) -> BOOL,
    GetFileVersionInfoExA: extern "system" fn(
        dwFlags: DWORD,
        lpwstrFilename: LPCSTR,
        dwHandle: DWORD,
        dwLen: DWORD,
        lpData: LPVOID,
    ) -> BOOL,
    GetFileVersionInfoExW: extern "system" fn(
        dwFlags: DWORD,
        lpwstrFilename: LPCWSTR,
        dwHandle: DWORD,
        dwLen: DWORD,
        lpData: LPVOID,
    ) -> BOOL,
    VerQueryValueA: extern "system" fn(
        pBlock: LPCVOID,
        lpSubBlock: LPCSTR,
        lplpBuffer: &mut LPVOID,
        puLen: PUINT,
    ) -> BOOL,
    VerQueryValueW: extern "system" fn(
        pBlock: LPCVOID,
        lpSubBlock: LPCWSTR,
        lplpBuffer: &mut LPVOID,
        puLen: PUINT,
    ) -> BOOL,
    VerLanguageNameA: extern "system" fn(wLang: DWORD, szLang: LPSTR, cchLang: DWORD) -> DWORD,
    VerLanguageNameW: extern "system" fn(wLang: DWORD, szLang: LPWSTR, cchLang: DWORD) -> DWORD,
}
