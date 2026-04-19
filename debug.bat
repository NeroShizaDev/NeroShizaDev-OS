@echo off
chcp 65001 >nul 2>nul

echo [*] NeroShizaDev-OS — Режим отладки GDB
echo ==========================================
echo  Шаг 1: QEMU запускается с -s -S (ждёт GDB на порту 1234)
echo  Шаг 2: Открой RustRover -^> Run -^> "NeroShizaOS QEMU Debug" -^> Debug (жук)
echo  ИЛИ подключись вручную: gdb -x .gdbinit
echo ==========================================
echo.

call run.bat --debug

