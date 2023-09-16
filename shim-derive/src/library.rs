use std::ffi::CStr;
use std::path::{Path, PathBuf};
use widestring::{WideCString, WideString};
use winapi::shared::minwindef::{HMODULE, MAX_PATH};
use winapi::shared::ntdef::WCHAR;
use winapi::um::libloaderapi::LoadLibraryW;
use winapi::um::sysinfoapi::GetSystemDirectoryW;
use winapi::um::winnt::{
    IMAGE_DIRECTORY_ENTRY_EXPORT, IMAGE_DOS_SIGNATURE, IMAGE_NT_SIGNATURE, PIMAGE_DOS_HEADER,
    PIMAGE_EXPORT_DIRECTORY, PIMAGE_NT_HEADERS,
};

pub struct Library {
    handle: HMODULE,
}

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
            let handle = LoadLibraryW(wide_path.as_ptr());

            if handle.is_null() {
                return None;
            }

            Some(Self { handle })
        }
    }

    pub fn all(&self) -> Vec<String> {
        unsafe {
            let base = self.handle as usize;
            let dos_header = &*(base as PIMAGE_DOS_HEADER);
            assert!(dos_header.e_magic == IMAGE_DOS_SIGNATURE);
            let nt_headers = &*((base + dos_header.e_lfanew as usize) as PIMAGE_NT_HEADERS);
            assert!(nt_headers.Signature == IMAGE_NT_SIGNATURE);
            assert!(nt_headers.OptionalHeader.NumberOfRvaAndSizes > 0);
            let export_directory = &*((base
                + nt_headers.OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_EXPORT as usize]
                    .VirtualAddress as usize)
                as PIMAGE_EXPORT_DIRECTORY);
            assert!(export_directory.AddressOfNames != 0);
            let names = (base + export_directory.AddressOfNames as usize) as *const u32;
            let mut result = Vec::new();

            for i in 0..export_directory.NumberOfNames {
                let offset = names.offset(i as isize).read();
                let name = (base + offset as usize) as *const i8;
                result.push(CStr::from_ptr(name).to_str().unwrap().to_owned());
            }

            result
        }
    }
}
