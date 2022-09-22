include!("src/commands/update/parse.rs");

#[cfg(all(windows, not(debug_assertions)))]
fn main() {
    use winapi::um::winnt::{LANG_ENGLISH, MAKELANGID, SUBLANG_ENGLISH_US};
    use winres::VersionInfo::PRODUCTVERSION;
    use winres::WindowsResource;

    // add the resource to the executable
    let mut resource = WindowsResource::new();
    resource.set_icon("build/windows/resources/hop.ico");
    resource.set_language(MAKELANGID(LANG_ENGLISH, SUBLANG_ENGLISH_US));

    // write the version to the resource
    let (major, minor, patch, release) = version(env!("CARGO_PKG_VERSION")).unwrap();

    resource.set_version_info(
        PRODUCTVERSION,
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

// no need to add for non windows or debug builds
#[cfg(any(not(windows), debug_assertions))]
fn main() {}
