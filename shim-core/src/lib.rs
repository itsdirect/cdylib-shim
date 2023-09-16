use object::Object;
use std::ffi::CString;
use std::mem::transmute_copy;
use std::path::{Path, PathBuf};
use widestring::{WideCString, WideString};
use winapi::shared::minwindef::{HMODULE, MAX_PATH};
use winapi::shared::ntdef::WCHAR;
use winapi::um::libloaderapi::{GetProcAddress, LoadLibraryW};
use winapi::um::processthreadsapi::GetCurrentProcess;
use winapi::um::psapi::GetModuleInformation;
use winapi::um::sysinfoapi::GetSystemDirectoryW;

pub struct Library(HMODULE);

impl Library {
    pub fn load_system<P: AsRef<Path>>(path: P) -> Option<Self> {
        unsafe {
            let mut system_directory = [0 as WCHAR; MAX_PATH];
            let result =
                GetSystemDirectoryW(&mut system_directory as _, system_directory.len() as _);

            if result == 0 {
                return None;
            }

            let system_directory = WideString::from_ptr(&system_directory as _, result as _);
            let path = PathBuf::from(system_directory.to_os_string()).join(path);
            Self::load(path)
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Option<Self> {
        unsafe {
            let wide_path = WideCString::from_os_str_unchecked(path.as_ref());
            let module = LoadLibraryW(wide_path.as_ptr());

            if module.is_null() {
                return None;
            }

            Some(Self(module))
        }
    }

    pub fn all(&self) -> Option<Vec<String>> {
        unsafe {
            let process = GetCurrentProcess();
            let mut module_info = std::mem::zeroed();

            let result = GetModuleInformation(
                process,
                self.0,
                &mut module_info,
                std::mem::size_of_val(&module_info) as _,
            );

            if result == 0 {
                return None;
            }

            let module = std::slice::from_raw_parts(
                module_info.lpBaseOfDll as *mut u8,
                module_info.SizeOfImage as _,
            );

            let Ok(file) = object::File::parse(module) else {
                return None;
            };

            let Ok(exports) = file.exports() else {
                return None;
            };

            let exports = exports
                .iter()
                .filter_map(|x| std::str::from_utf8(x.name()).ok())
                .map(ToOwned::to_owned)
                .collect();

            Some(exports)
        }
    }

    /// # Safety
    pub unsafe fn get<T>(&self, name: &str) -> Option<T> {
        let owned_name = CString::new(name).ok()?;
        let address = GetProcAddress(self.0, owned_name.as_ptr());

        if address.is_null() {
            return None;
        }

        Some(transmute_copy(&address))
    }
}
