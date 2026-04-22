@echo off
setlocal EnableExtensions EnableDelayedExpansion
chcp 65001 >nul 2>nul

set "SCRIPT_DIR=%~dp0"
for %%I in ("%SCRIPT_DIR%..\..") do set "REPO_ROOT=%%~fI\"
set "INSTALLER_DIR=%SCRIPT_DIR%"
pushd "%REPO_ROOT%" >nul

set "BUILDER=%INSTALLER_DIR%nhs_build.py"
set "SENDER=%REPO_ROOT%tools\send_nhs.py"
set "DEFAULT_TCP=localhost:4321"
set "TRANSPORT=tcp"
set "TCP_TARGET=%DEFAULT_TCP%"
set "SERIAL_PORT="
set "BAUD=115200"
set "BUILD_ARGS="
set "OUTPUT_FILE="
set "INPUT_ARG=%~1"

if "%~1"=="" goto :interactive
if /I "%~1"=="--help" goto :usage
if /I "%~1"=="-h" goto :usage

shift

:parse_args
if "%~1"=="" goto :after_parse

if /I "%~1"=="--tcp" (
    if "%~2"=="" goto :missing_value
    set "TRANSPORT=tcp"
    set "TCP_TARGET=%~2"
    shift
    shift
    goto :parse_args
)

if /I "%~1"=="--port" (
    if "%~2"=="" goto :missing_value
    set "TRANSPORT=serial"
    set "SERIAL_PORT=%~2"
    shift
    shift
    goto :parse_args
)

if /I "%~1"=="--baud" (
    if "%~2"=="" goto :missing_value
    set "BAUD=%~2"
    shift
    shift
    goto :parse_args
)

if /I "%~1"=="-o" (
    if "%~2"=="" goto :missing_value
    set "OUTPUT_FILE=%~f2"
    shift
    shift
    goto :parse_args
)

if /I "%~1"=="--output" (
    if "%~2"=="" goto :missing_value
    set "OUTPUT_FILE=%~f2"
    shift
    shift
    goto :parse_args
)

if /I "%~1"=="--name" (
    if "%~2"=="" goto :missing_value
    set "BUILD_ARGS=!BUILD_ARGS! --name ""%~2"""
    shift
    shift
    goto :parse_args
)

if /I "%~1"=="--author" (
    if "%~2"=="" goto :missing_value
    set "BUILD_ARGS=!BUILD_ARGS! --author ""%~2"""
    shift
    shift
    goto :parse_args
)

if /I "%~1"=="--version" (
    if "%~2"=="" goto :missing_value
    set "BUILD_ARGS=!BUILD_ARGS! --version ""%~2"""
    shift
    shift
    goto :parse_args
)

if /I "%~1"=="--cat" (
    if "%~2"=="" goto :missing_value
    set "BUILD_ARGS=!BUILD_ARGS! --cat ""%~2"""
    shift
    shift
    goto :parse_args
)

if /I "%~1"=="--rodata" (
    if "%~2"=="" goto :missing_value
    set "BUILD_ARGS=!BUILD_ARGS! --rodata ""%~f2"""
    shift
    shift
    goto :parse_args
)

if /I "%~1"=="--data" (
    if "%~2"=="" goto :missing_value
    set "BUILD_ARGS=!BUILD_ARGS! --data ""%~f2"""
    shift
    shift
    goto :parse_args
)

if /I "%~1"=="--bss" (
    if "%~2"=="" goto :missing_value
    set "BUILD_ARGS=!BUILD_ARGS! --bss %~2"
    shift
    shift
    goto :parse_args
)

if /I "%~1"=="--lump" (
    if "%~2"=="" goto :missing_value
    set "BUILD_ARGS=!BUILD_ARGS! --lump ""%~2"""
    shift
    shift
    goto :parse_args
)

if /I "%~1"=="--fpu" (
    set "BUILD_ARGS=!BUILD_ARGS! --fpu"
    shift
    goto :parse_args
)

if /I "%~1"=="--gfx" (
    set "BUILD_ARGS=!BUILD_ARGS! --gfx"
    shift
    goto :parse_args
)

if /I "%~1"=="--speaker" (
    set "BUILD_ARGS=!BUILD_ARGS! --speaker"
    shift
    goto :parse_args
)

if /I "%~1"=="--native" (
    set "BUILD_ARGS=!BUILD_ARGS! --native"
    shift
    goto :parse_args
)

echo [!] Unknown option: %~1
echo.
goto :usage_error

:after_parse
if not exist "%BUILDER%" (
    echo [!] Builder not found: %BUILDER%
    goto :fail
)

if not exist "%SENDER%" (
    echo [!] Sender not found: %SENDER%
    goto :fail
)

call :resolve_python
if errorlevel 1 goto :fail

call :resolve_input "%INPUT_ARG%"
if errorlevel 1 goto :fail

if /I "!INPUT_MODE!"=="source" (
    if not defined OUTPUT_FILE set "OUTPUT_FILE=!INPUT_DIR!!INPUT_STEM!.nhs"

    echo [*] Step 1/2: build .nhs package
    echo     Source: !SOURCE_FILE!
    echo     Output: !OUTPUT_FILE!
    %PYTHON_CMD% "%BUILDER%" "!SOURCE_FILE!" -o "!OUTPUT_FILE!" !BUILD_ARGS!
    if errorlevel 1 (
        echo [!] nhs_build.py failed.
        goto :fail
    )
    set "NHS_FILE=!OUTPUT_FILE!"
) else (
    set "NHS_FILE=!RESOLVED_NHS!"
)

if not exist "!NHS_FILE!" (
    echo [!] NHS package not found: !NHS_FILE!
    goto :fail
)

echo [*] Step 2/2: send package to running OS
echo     Package: !NHS_FILE!
if /I "%TRANSPORT%"=="serial" (
    if not defined SERIAL_PORT (
        echo [!] Serial mode selected but COM port was not provided.
        goto :fail
    )
    echo     Transport: COM !SERIAL_PORT! @ %BAUD%
    echo     Guest shell must already be waiting in: install serial
    %PYTHON_CMD% "%SENDER%" "!NHS_FILE!" --port "%SERIAL_PORT%" --baud %BAUD%
) else (
    echo     Transport: TCP %TCP_TARGET%
    echo     Guest shell must already be waiting in: install serial
    %PYTHON_CMD% "%SENDER%" "!NHS_FILE!" --tcp %TCP_TARGET%
)

if errorlevel 1 (
    echo [!] send_nhs.py failed.
    goto :fail
)

echo.
echo [OK] Install pipeline finished.
echo     Builder: nhs_build.py
echo     Sender:  send_nhs.py
echo     Payload: !NHS_FILE!
popd >nul
exit /b 0

:resolve_python
where py >nul 2>nul
if not errorlevel 1 (
    set "PYTHON_CMD=py -3"
    goto :eof
)

where python >nul 2>nul
if not errorlevel 1 (
    set "PYTHON_CMD=python"
    goto :eof
)

echo [!] Python not found. Install Python or use py launcher.
exit /b 1

:resolve_input
set "INPUT_RAW=%~1"
set "INPUT_MODE="
set "RESOLVED_NHS="
set "SOURCE_FILE="
set "INPUT_DIR="
set "INPUT_STEM="

if exist "%~f1" (
    if /I "%~x1"==".nhs" (
        set "INPUT_MODE=package"
        set "RESOLVED_NHS=%~f1"
        goto :eof
    )

    set "INPUT_MODE=source"
    set "SOURCE_FILE=%~f1"
    set "INPUT_DIR=%~dp1"
    set "INPUT_STEM=%~n1"
    goto :eof
)

if exist "%INSTALLER_DIR%nhsapps\%~1.nhs" (
    set "INPUT_MODE=package"
    set "RESOLVED_NHS=%INSTALLER_DIR%nhsapps\%~1.nhs"
    goto :eof
)

if exist "%INSTALLER_DIR%%~1.nhs" (
    set "INPUT_MODE=package"
    set "RESOLVED_NHS=%INSTALLER_DIR%%~1.nhs"
    goto :eof
)

echo [!] Input not found: %~1
echo     Expected .nss/.nhs file, or built-in package name like demo/hello.
exit /b 1

:missing_value
echo [!] Missing value for option %~1
echo.
goto :usage_error

:interactive
cls
echo ============================================================
echo   NHS INSTALLER  --  NeroShizaDev-OS
echo ============================================================
echo.
echo   Built-in packages:  demo   hello
echo   Your script:        path\to\script.nss
echo.
echo   IMPORTANT: before sending:
echo     1. Run QEMU via run.bat
echo     2. In OS shell type:  install serial
echo.
echo ============================================================
echo.
set /p "INPUT_ARG=Enter package name (demo / hello / path\to\script.nss): "
if "!INPUT_ARG!"=="" (
    echo Cancelled.
    pause
    popd >nul
    exit /b 0
)
echo.
goto :after_parse

:usage
echo INSTALL.bat - build and push NHS packages from host to a running NeroShizaDev-OS guest
echo.
echo Usage:
echo   INSTALL.bat ^<input.nss^|input.nhs^|demo^|hello^> [options]
echo.
echo Examples:
echo   INSTALL.bat demo
echo   INSTALL.bat hello --tcp localhost:4321
echo   INSTALL.bat apps\my_app.nss --name "My App" --author "Kotlew"
echo   INSTALL.bat apps\my_app.nss --port COM3
echo   INSTALL.bat apps\my_app.nhs --tcp localhost:4321
echo.
echo Options:
echo   --tcp HOST:PORT   Send via QEMU serial bridge. Default: %DEFAULT_TCP%
echo   --port COMX       Send via real serial port instead of TCP.
echo   --baud N          Serial baud rate. Default: 115200
echo   -o, --output FILE Output .nhs path when input is source.
echo   --name TEXT       Override manifest app name.
echo   --author TEXT     Override manifest author.
echo   --version X.Y.Z   Override manifest version.
echo   --cat NAME        Package category for nhs_build.py
echo   --rodata FILE     Add RODATA section from file.
echo   --data FILE       Add DATA section from file.
echo   --bss N           Set BSS size.
echo   --lump FILE:TAG   Add lump to package.
echo   --fpu             Set FLAG_NEEDS_FPU.
echo   --gfx             Set FLAG_NEEDS_GFX.
echo   --speaker         Set FLAG_NEEDS_SPEAKER.
echo   --native          Build as native NHS instead of script.
echo.
echo Guest side:
echo   1. Start QEMU with serial TCP bridge on localhost:4321
echo   2. In the OS shell run: install serial
echo   3. Run this INSTALL.bat on the host
echo.
pause
popd >nul
exit /b 0

:usage_error
echo.
pause
popd >nul
exit /b 1

:fail
echo.
echo [!] FAIL. Проверь:
echo     1. QEMU запущен через run.bat
echo     2. В шелле QEMU набери: install serial
echo     3. Затем запусти INSTALL.bat снова
echo.
pause
popd >nul
exit /b 1