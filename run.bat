@echo off
setlocal EnableExtensions EnableDelayedExpansion
chcp 65001 >nul 2>nul

for %%I in ("%~dp0.") do set "PROJECT_ROOT=%%~fI"
pushd "%PROJECT_ROOT%" >nul

set "KERNEL=target\x86_64-blog_os\debug\blog_os"
set "IMAGE=target\x86_64-blog_os\debug\blog_os-uefi.img"
set "QEMU=C:\Program Files\qemu\qemu-system-x86_64.exe"
set "IMAGE_BUILDER_MANIFEST=tools\image_builder\Cargo.toml"
set "IMAGE_BUILDER_TARGET=tools\image_builder\target"
set "HOST_CARGO_TARGET=x86_64-pc-windows-msvc"
set "OVMF_CODE=tmp\qemu-fw\edk2-x86_64-code.fd"
set "OVMF_VARS_TEMPLATE=tmp\qemu-fw\edk2-x86_64-vars.fd"
set "OVMF_VARS=target\x86_64-blog_os\debug\ovmf-vars.fd"
set "SERIAL_LOG=qemu-serial.log"
set "DEBUGCON_LOG=qemu-debugcon.log"
set "ACTION="
set "VBOX_VM=NeroShizaOS"
set "VBOXMANAGE=C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"
set "VBOX_STORAGECTL=NVMe"
set "VBOX_VDI=target\x86_64-blog_os\debug\blog_os-uefi.vdi"

for %%A in (%*) do (
    if /I "%%~A"=="qemu" set "ACTION=QEMU"
    if /I "%%~A"=="build" set "ACTION=BUILD"
    if /I "%%~A"=="vbox" set "ACTION=VBOX"
    if /I "%%~A"=="1" set "ACTION=QEMU"
    if /I "%%~A"=="2" set "ACTION=BUILD"
    if /I "%%~A"=="3" set "ACTION=VBOX"
    if /I "%%~A"=="--build-only" set "ACTION=BUILD"
)

if not defined ACTION (
    echo.
    echo [1] Sobrat^' i zapustit^' v QEMU ^(UEFI^)
    echo [2] Tol^'ko sobrat^' UEFI obraz
    echo [3] Sobrat^' UEFI, konvertirovat^' v VDI i zapustit^' v VirtualBox
    echo.
    choice /c 123 /n /m "Vybor punkta [1/2/3]: "
    set "_CH=!errorlevel!"
    if "!_CH!"=="3" set "ACTION=VBOX"
    if "!_CH!"=="2" set "ACTION=BUILD"
    if "!_CH!"=="1" set "ACTION=QEMU"
)

echo [*] NeroShizaDev-OS v0.3 - Sborka...
echo [*] Proverka formatirovaniya ^(cargo fmt^)...
cargo fmt --all -- --check
if errorlevel 1 (
    echo [!] Oshibka formatirovaniya. Zapustite: cargo fmt --all
    pause
    popd >nul
    exit /b 1
)
cargo build --bin blog_os
if errorlevel 1 (
    echo [!] Oshibka sborki yadra.
    pause
    popd >nul
    exit /b 1
)

if not exist "%KERNEL%" (
    echo [!] Yadro ne naydeno: %KERNEL%
    pause
    popd >nul
    exit /b 1
)

if not exist "%IMAGE_BUILDER_MANIFEST%" (
    echo [!] Image builder ne naiden: %IMAGE_BUILDER_MANIFEST%
    pause
    popd >nul
    exit /b 1
)

echo [*] Sборка UEFI-obraza...
powershell -NoProfile -ExecutionPolicy Bypass -File "%PROJECT_ROOT%\tools\apply_patches.ps1" -ProjectRoot "%PROJECT_ROOT%"
if errorlevel 1 (
    echo [!] Oshibka primeneniya patchey.
    pause
    popd >nul
    exit /b 1
)
set "CARGO_BUILD_TARGET="
set "CARGO_TARGET_DIR="
set "RUSTFLAGS="
set "CARGO_ENCODED_RUSTFLAGS="
set "RUSTC_WORKSPACE_WRAPPER="
pushd "%SystemDrive%\" >nul
cargo +nightly run --manifest-path "%PROJECT_ROOT%\%IMAGE_BUILDER_MANIFEST%" --target "%HOST_CARGO_TARGET%" --target-dir "%PROJECT_ROOT%\%IMAGE_BUILDER_TARGET%" -- uefi "%PROJECT_ROOT%\%KERNEL%" "%PROJECT_ROOT%\%IMAGE%"
set "IMAGE_BUILD_RC=%ERRORLEVEL%"
popd >nul
if not "%IMAGE_BUILD_RC%"=="0" (
    echo [!] Oshibka sborki UEFI-obraza.
    pause
    popd >nul
    exit /b 1
)

if not exist "%IMAGE%" (
    echo [!] Oshibka sborki! Obraz ne sozdan.
    pause
    popd >nul
    exit /b 1
)

if /I "%ACTION%"=="BUILD" (
    echo [*] Obraz sobran: %IMAGE%
    popd >nul
    exit /b 0
)

if /I "%ACTION%"=="QEMU" goto run_qemu
if /I "%ACTION%"=="VBOX" goto run_vbox

echo [ERR] Neizvestnoe deystvie: %ACTION%
pause
popd >nul
exit /b 1

:run_qemu
if not exist "%QEMU%" (
    echo [!] QEMU ne nayden: %QEMU%
    echo     https://www.qemu.org/download/#windows
    pause
    popd >nul
    exit /b 1
)

if not exist "%OVMF_CODE%" (
    echo [!] OVMF CODE ne nayden: %OVMF_CODE%
    pause
    popd >nul
    exit /b 1
)

if not exist "%OVMF_VARS_TEMPLATE%" (
    echo [!] OVMF VARS template ne nayden: %OVMF_VARS_TEMPLATE%
    pause
    popd >nul
    exit /b 1
)

rem Сбрасываем UEFI NVRAM на каждом запуске QEMU:
rem иначе OVMF может запомнить вход в UEFI Shell и больше не грузить BOOTX64.EFI.
copy /y "%OVMF_VARS_TEMPLATE%" "%OVMF_VARS%" >nul

echo [*] Zapusk v QEMU ^(UEFI, framebuffer^)...
echo     Komandy: gubka, zvuk, vremya, pomosh
echo     Esc=sbros, CapsLock=enter, ScrollLock=RUS/ENG
echo     Serial log: %SERIAL_LOG%
echo     Debugcon log: %DEBUGCON_LOG%
echo.

if exist "%SERIAL_LOG%" del /f /q "%SERIAL_LOG%" >nul 2>nul
if exist "%DEBUGCON_LOG%" del /f /q "%DEBUGCON_LOG%" >nul 2>nul

"%QEMU%" -cpu max -machine q35,pcspk-audiodev=snd0 -drive if=pflash,format=raw,readonly=on,file="%OVMF_CODE%" -drive if=pflash,format=raw,file="%OVMF_VARS%" -drive format=raw,file="%IMAGE%" -boot order=c -no-reboot -no-shutdown -audiodev sdl,id=snd0 -serial "file:%PROJECT_ROOT%\%SERIAL_LOG%" -debugcon "file:%PROJECT_ROOT%\%DEBUGCON_LOG%" -monitor none

popd >nul
exit /b 0

:run_vbox
if not exist "%VBOXMANAGE%" (
    echo [!] VBoxManage ne nayden: %VBOXMANAGE%
    pause
    popd >nul
    exit /b 1
)

call :get_vm_state
if /I not "%VM_STATE%"=="poweroff" (
    echo [*] VirtualBox sync: power off VM "%VBOX_VM%" ^(state=%VM_STATE%^)...
    "%VBOXMANAGE%" controlvm "%VBOX_VM%" poweroff >nul 2>nul
    call :wait_for_vm_state poweroff 15
    if errorlevel 1 (
        echo [!] VM "%VBOX_VM%" ne pereshla v poweroff. Tekushee sostoyanie: %VM_STATE%
        pause
        popd >nul
        exit /b 1
    )
)

"%VBOXMANAGE%" modifyvm "%VBOX_VM%" --firmware efi >nul 2>nul

rem --- Detach old VDI and purge from VBox media registry ---
"%VBOXMANAGE%" storageattach "%VBOX_VM%" --storagectl "%VBOX_STORAGECTL%" --port 0 --device 0 --type hdd --medium none >nul 2>nul
"%VBOXMANAGE%" closemedium disk "%PROJECT_ROOT%\%VBOX_VDI%" >nul 2>nul
if exist "%VBOX_VDI%" del /f /q "%VBOX_VDI%" >nul 2>nul

echo [*] VirtualBox sync: UEFI IMG -^> VDI...
"%VBOXMANAGE%" convertfromraw --format VDI "%PROJECT_ROOT%\%IMAGE%" "%PROJECT_ROOT%\%VBOX_VDI%"
if errorlevel 1 (
    echo [!] Ne udalos skonvertirovat UEFI IMG v VDI.
    pause
    popd >nul
    exit /b 1
)

echo [*] VirtualBox sync: attach VDI to VM "%VBOX_VM%"...
"%VBOXMANAGE%" storageattach "%VBOX_VM%" --storagectl "%VBOX_STORAGECTL%" --port 0 --device 0 --type hdd --medium "%PROJECT_ROOT%\%VBOX_VDI%"
if errorlevel 1 (
    echo [!] Ne udalos podklyuchit VDI k VM "%VBOX_VM%".
    pause
    popd >nul
    exit /b 1
)

echo [*] VirtualBox run: start VM "%VBOX_VM%"...
"%VBOXMANAGE%" startvm "%VBOX_VM%"
if errorlevel 1 (
    echo [!] Ne udalos zapustit VM "%VBOX_VM%".
    pause
    popd >nul
    exit /b 1
)

popd >nul
exit /b 0

:get_vm_state
set "VM_STATE="
set "VM_STATE_FILE=%TEMP%\neroshiza-vmstate-%RANDOM%-%RANDOM%.tmp"
"%VBOXMANAGE%" showvminfo "%VBOX_VM%" --machinereadable > "%VM_STATE_FILE%" 2>nul
for /f "tokens=2 delims==" %%S in ('findstr /b "VMState=" "%VM_STATE_FILE%"') do set "VM_STATE=%%~S"
if exist "%VM_STATE_FILE%" del /f /q "%VM_STATE_FILE%" >nul 2>nul
exit /b 0

:wait_for_vm_state
set "WAIT_TARGET=%~1"
set "WAIT_RETRIES=%~2"
:wait_for_vm_state_loop
call :get_vm_state
if /I "%VM_STATE%"=="%WAIT_TARGET%" exit /b 0
if "%WAIT_RETRIES%"=="0" exit /b 1
set /a WAIT_RETRIES-=1
timeout /t 1 /nobreak >nul
goto wait_for_vm_state_loop
