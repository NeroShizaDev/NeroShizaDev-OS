@echo off
chcp 65001 >nul 2>nul

set "IMAGE=target\x86_64-blog_os\debug\bootimage-blog_os.bin"

echo [*] Сборка blog_os...
cargo bootimage
if not exist "%IMAGE%" (
    echo [!] Ошибка сборки! Образ не создан.
    pause
    exit /b 1
)

where qemu-system-x86_64 >nul 2>nul
if %errorlevel% neq 0 (
    if exist "C:\Program Files\qemu\qemu-system-x86_64.exe" (
        set "QEMU=C:\Program Files\qemu\qemu-system-x86_64.exe"
    ) else (
        echo [!] QEMU не найден!
        echo     Скачай: https://www.qemu.org/download/#windows
        echo     Установи в C:\Program Files\qemu и добавь в PATH
        pause
        exit /b 1
    )
) else (
    set "QEMU=qemu-system-x86_64"
)

echo [*] Запуск в QEMU...
"%QEMU%" -cpu max -drive format=raw,file=%IMAGE% -no-reboot -no-shutdown
pause
