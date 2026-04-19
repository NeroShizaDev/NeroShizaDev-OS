@echo off
setlocal

set "SCRIPT_DIR=%~dp0"
set "ROOT=%SCRIPT_DIR%.."
set "REPORT=%SCRIPT_DIR%reports\custom_inspector_report.txt"

powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT_DIR%custom_inspector.ps1" -RootPath "%ROOT%" -ReportPath "%REPORT%"
set "CODE=%ERRORLEVEL%"

echo.
echo Report: %REPORT%
exit /b %CODE%

