use std::ffi::CString;
use std::mem::transmute_copy;
use std::path::{Path, PathBuf};
use widestring::{WideCString, WideString};
use winapi::shared::minwindef::{HMODULE, MAX_PATH};
use winapi::shared::ntdef::WCHAR;
use winapi::um::libloaderapi::{GetProcAddress, LoadLibraryW};
use winapi::um::sysinfoapi::GetSystemDirectoryW;

pub struct Library {
    handle: HMODULE,
}

impl Library {
    pub unsafe fn load_system<P: AsRef<Path>>(path: P) -> Option<Self> {
        let mut system_directory = [0 as WCHAR; MAX_PATH];
        let result = GetSystemDirectoryW(&mut system_directory as _, system_directory.len() as _);

        if result == 0 {
            return None;
        }

        let system_directory = WideString::from_ptr(&system_directory as _, result as _);
        let path = PathBuf::from(system_directory.to_os_string()).join(path);
        Self::load(path)
    }

    pub unsafe fn load<P: AsRef<Path>>(path: P) -> Option<Self> {
        let wide_path = WideCString::from_os_str_unchecked(path.as_ref());
        let handle = LoadLibraryW(wide_path.as_ptr());

        if handle.is_null() {
            return None;
        }

        Some(Self { handle })
    }

    pub unsafe fn get<T>(&self, name: &str) -> Option<T> {
        let owned_name = CString::new(name).ok()?;
        let address = GetProcAddress(self.handle, owned_name.as_ptr());

        if address.is_null() {
            return None;
        }

        Some(transmute_copy(&address))
    }
}
