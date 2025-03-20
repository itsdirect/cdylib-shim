#[shim::shim("version.dll")]
mod version {
    #[load]
    fn load() {
        println!("Loading");
    }

    #[unload]
    fn unload() {
        println!("Unloading");
    }

    #[hook]
    unsafe extern "system" fn GetFileVersionInfoA() {
        println!("GetFileVersionInfoA");
    }
}
