@echo off
chcp 65001 >nul 2>nul

set "ROOT=%~dp0"
pushd "%ROOT%" >nul

echo [*] NeroShizaDev-OS: Rust syntax audit (compiler errors only)
echo [*] 1/2 cargo check -- синтаксис и типы (только ошибки)
cargo check --message-format=short 2>&1 | findstr /I "error"
set ERR=%ERRORLEVEL%
if %ERR% NEQ 0 (
    echo.
    echo [!] Найдены ошибки компилятора!
    echo [*] Полный вывод:
    cargo check --message-format=human 2>&1
    popd >nul
    exit /b 1
)

echo [OK] Шаг 1: синтаксис чистый.
echo [*] 2/2 cargo check --tests -- тесты компилируются?
cargo check --tests --message-format=short 2>&1 | findstr /I "error"
set ERR2=%ERRORLEVEL%
if %ERR2% NEQ 0 (
    echo.
    echo [!] Ошибки в тестах!
    cargo check --tests --message-format=human 2>&1
    popd >nul
    exit /b 1
)

echo [OK] Шаг 2: тесты компилируются.
echo.
echo ====================================
echo  AUDIT PASSED — только синтаксис!
echo  Предупреждения стиля подавлены.
echo ====================================
popd >nul
exit /b 0
