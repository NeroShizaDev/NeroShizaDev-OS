# NeroShizaDev-OS — GDB init script
# Используется при: gdb -x .gdbinit
# или автоматически если add-auto-load-safe-path настроен

set architecture i386:x86-64
set disassembly-flavor intel

# Подключение к QEMU GDB-серверу
target remote localhost:1234

# Загрузить отладочные символы ядра
symbol-file target/x86_64-neroshiza_dev_os/debug/neroshiza_dev_os

# Удобные breakpoints для старта
# break kernel_main
# break _start

# Показать регистры при каждой остановке
define hook-stop
  info registers rip rsp rbp rflags
end

echo \n[GDB] NeroShizaOS подключён! Используй 'c' для старта, 'b kernel_main' для breakpoint.\n\n

