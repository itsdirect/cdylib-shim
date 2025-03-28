use std::{
    ffi::{CStr, CString},
    path::Path,
    ptr::NonNull,
};

use winapi::{
    shared::minwindef::{HMODULE, MAX_PATH},
    um::{
        libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryA},
        sysinfoapi::GetSystemDirectoryA,
    },
};

pub struct Library(HMODULE);

impl Library {
    pub fn load_system(path: impl AsRef<Path>) -> Option<Self> {
        unsafe {
            let mut system_directory = [0; MAX_PATH];
            let result = GetSystemDirectoryA(
                system_directory.as_mut_ptr(),
                size_of_val(&system_directory) as _,
            );

            if result == 0 {
                return None;
            }

            let system_directory = CStr::from_ptr(system_directory.as_ptr()).to_str().ok()?;
            let path = Path::new(system_directory).join(path);
            Self::load(path)
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Option<Self> {
        unsafe {
            let path = CString::new(path.as_ref().as_os_str().as_encoded_bytes()).ok()?;
            let module = LoadLibraryA(path.as_ptr());

            if module.is_null() {
                return None;
            }

            Some(Self(module))
        }
    }

    pub fn get(&self, name: &str) -> Option<NonNull<()>> {
        unsafe {
            let name = CString::new(name).ok()?;
            let address = GetProcAddress(self.0, name.as_ptr());
            NonNull::new(address.cast())
        }
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        unsafe { FreeLibrary(self.0) };
    }
}
