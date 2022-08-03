include!("src/commands/update/parse.rs");

#[cfg(windows)]
fn main() {
    extern crate winapi;
    extern crate winres;

    // add the resource to the executable
    let mut resource = winres::WindowsResource::new();
    resource.set_icon("build/windows/resources/hop.ico");
    resource.set_language(winapi::um::winnt::MAKELANGID(
        winapi::um::winnt::LANG_ENGLISH,
        winapi::um::winnt::SUBLANG_ENGLISH_US,
    ));

    // write the version to the resource
    let (major, minor, patch, release) = version(env!("CARGO_PKG_VERSION")).unwrap();

    resource.set_version_info(
        winres::VersionInfo::PRODUCTVERSION,
        (major as u64) << 48
            | (minor as u64) << 32
            | (patch as u64) << 16
            | (release.unwrap_or(0) as u64 + 1),
    );

    // compile the resource file
    resource.compile().unwrap();

    // fix VCRUNTIME140.dll
    static_vcruntime::metabuild();
}

#[cfg(not(windows))]
fn main() {}
