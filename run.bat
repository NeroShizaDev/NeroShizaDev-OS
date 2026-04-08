@echo off
chcp 65001 >nul 2>nul

set "IMAGE=target\x86_64-blog_os\debug\bootimage-blog_os.bin"
set "QEMU=C:\Program Files\qemu\qemu-system-x86_64.exe"

echo [*] NeroShizaDev OS v0.2 - Sborka...
cargo bootimage
if not exist "%IMAGE%" (
    echo [!] Oshibka sborki! Obraz ne sozdan.
    pause
    exit /b 1
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

"%QEMU%" -cpu max -drive format=raw,file=%IMAGE% -no-reboot -no-shutdown -audiodev sdl,id=snd0 -machine pcspk-audiodev=snd0

pause
