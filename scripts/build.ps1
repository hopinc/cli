# powershell -ExecutionPolicy Bypass -File .\scripts\build.ps1

cargo build --release --target x86_64-pc-windows-msvc --package hop-cli

cargo wix -I .\build\windows\main.wxs -v --nocapture --target x86_64-pc-windows-msvc --output target/wix/hop-x86_64-pc-windows-msvc.msi --package hop-cli

target/wix/hop-x86_64-pc-windows-msvc.msi