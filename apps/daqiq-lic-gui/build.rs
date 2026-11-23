fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "Daqiq License Generator Studio");
        res.set("FileDescription", "Daqiq Commercial License Studio v1.3.0");
        res.set("ProductVersion", "1.3.0");
        res.set("FileVersion", "1.3.0");
        res.set("LegalCopyright", "Copyright (c) 2026 Daqiq");
        let _ = res.compile();
    }
}
