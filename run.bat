@echo off
chcp 65001 >nul 2>nul

set "KERNEL=target\x86_64-blog_os\debug\blog_os"
set "IMAGE=target\x86_64-blog_os\debug\blog_os-bios.img"
set "QEMU=C:\Program Files\qemu\qemu-system-x86_64.exe"
set "TMP_BUILDER=%TEMP%\neroshiza_image_builder"
set "TMP_TARGET=%TMP_BUILDER%\target"
set "PROJECT_ROOT=%CD%"
set "BUILD_ONLY=0"
set "DEBUG_GDB=0"
set "SERIAL_BRIDGE=0"
set "SERIAL_PORT=4321"

if /I "%~1"=="--build-only" set "BUILD_ONLY=1"
if /I "%~1"=="--serial-bridge" set "SERIAL_BRIDGE=1"
if /I "%~2"=="--serial-bridge" set "SERIAL_BRIDGE=1"
if /I "%~1"=="--debug" set "DEBUG_GDB=1"
if /I "%~2"=="--debug" set "DEBUG_GDB=1"
if /I "%~1"=="--serial-log" set "SERIAL_BRIDGE=0"
if /I "%~2"=="--serial-log" set "SERIAL_BRIDGE=0"

echo [*] NeroShizaDev-OS v0.3 - Sborka...
cargo build --bin blog_os
if errorlevel 1 (
    echo [!] Oshibka sborki yadra.
    pause
    exit /b 1
)

if not exist "%KERNEL%" (
    echo [!] Yadro ne naydeno: %KERNEL%
    pause
    exit /b 1
)

if exist "%TMP_BUILDER%" rmdir /s /q "%TMP_BUILDER%"
mkdir "%TMP_BUILDER%\src" >nul 2>nul

powershell -NoProfile -Command "$cargoToml = @('[package]','name = ''neroshiza_image_builder''','version = ''0.1.0''','edition = ''2021''','','[dependencies]','bootloader = { version = ''0.11.15'', default-features = false, features = [''bios''] }'); Set-Content -LiteralPath '%TMP_BUILDER%\Cargo.toml' -Value $cargoToml; $mainRs = @('use std::{env, path::PathBuf};','','fn main() {','    let mut args = env::args_os().skip(1);','    let kernel_path = PathBuf::from(args.next().expect(\"missing kernel path\"));','    let image_path = PathBuf::from(args.next().expect(\"missing image path\"));','','    if args.next().is_some() {','        panic!(\"unexpected extra arguments\");','    }','','    let bios = bootloader::BiosBoot::new(&kernel_path);','    bios.create_disk_image(&image_path)','        .unwrap_or_else(|err| panic!(\"failed to create BIOS disk image: {err}\"));','','    println!(\"created BIOS image: {}\", image_path.display());','}'); Set-Content -LiteralPath '%TMP_BUILDER%\src\main.rs' -Value $mainRs"
if errorlevel 1 (
    echo [!] Ne udalos podgotovit temporary image builder.
    pause
    exit /b 1
)

echo [*] Sборка BIOS-obraza...
pushd "%TMP_BUILDER%"
set "CARGO_BUILD_TARGET="
set "CARGO_TARGET_DIR="
set "RUSTFLAGS="
set "CARGO_ENCODED_RUSTFLAGS="
set "RUSTC_WORKSPACE_WRAPPER="
cargo +nightly run --target-dir "%TMP_TARGET%" -- "%PROJECT_ROOT%\%KERNEL%" "%PROJECT_ROOT%\%IMAGE%"
set "BUILD_RC=%ERRORLEVEL%"
popd
if errorlevel 1 (
    echo [!] Oshibka sborki BIOS-obraza.
    pause
    exit /b 1
)
if not "%BUILD_RC%"=="0" (
    echo [!] Oshibka sborki BIOS-obraza.
    pause
    exit /b 1
)
if not exist "%IMAGE%" (
    echo [!] Oshibka sborki! Obraz ne sozdan.
    pause
    exit /b 1
)

if "%BUILD_ONLY%"=="1" (
    echo [*] Obraz sobran: %IMAGE%
    exit /b 0
)

if not exist "%QEMU%" (
    echo [!] QEMU ne najden: %QEMU%
    echo     https://www.qemu.org/download/#windows
    pause
    exit /b 1
)

echo [*] Zapusk v QEMU (PC Speaker ON)...
echo     Komandy: gubka, zvuk, vremya, pomosh
echo     Esc=sbros, CapsLock=enter, ScrollLock=RUS/ENG
echo.

set "QEMU_EXTRA="
if "%DEBUG_GDB%"=="1" (
    set "QEMU_EXTRA=-s -S"
    echo [*] GDB DEBUG MODE: QEMU zhdyot podklyucheniya na localhost:1234
    echo     gdb target\x86_64-blog_os\debug\blog_os
    echo.
)

set "SERIAL_LOG_PATH=%PROJECT_ROOT%\serial.log"
set "QEMU_SERIAL=-serial file:%SERIAL_LOG_PATH%"
if "%SERIAL_BRIDGE%"=="1" (
    set "QEMU_SERIAL=-serial tcp:127.0.0.1:%SERIAL_PORT%,server,nowait"
    echo [*] SERIAL OTA MODE: COM1 bridge na localhost:%SERIAL_PORT%
    echo     Shell: install serial
    echo     Host:  python tools/send_nhs.py apps/installer/demo.nhs --tcp localhost:%SERIAL_PORT%
    echo.
)
if "%SERIAL_BRIDGE%"=="0" (
    > "%SERIAL_LOG_PATH%" type nul
    echo [*] SERIAL LOG MODE: %SERIAL_LOG_PATH%
)

"%QEMU%" -cpu max -drive format=raw,file=%IMAGE% -no-reboot -no-shutdown -audiodev sdl,id=snd0 -machine pcspk-audiodev=snd0 %QEMU_SERIAL% %QEMU_EXTRA%

pause
