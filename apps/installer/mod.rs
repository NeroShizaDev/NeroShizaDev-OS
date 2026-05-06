// ============================================================
// NHS — NeroShizaDev Hybrid System
// ============================================================
// Самоустанавливающиеся пакеты для NeroShizaDev-OS.
//
// Аналогия:
//   Windows setup.exe  →  .nhs файл
//   Program Files\     →  AppSlot[0..15] в BSS (64KB каждый)
//   Реестр Windows     →  AppRegistry (static [InstalledApp; 16])
//   Панель управления  →  команда `uninstall` + менеджер в launcher
//   Ярлык в Пуске      →  автоматический пункт в LAUNCH PAD
//   Мастер установки   →  TUI-визард с прогресс-баром
//
// Поток:
//   1. .nhs файл → include_bytes!() или ramdisk
//   2. Shell: `install demo|hello` → installer::install()
//   3. Installer читает header, проверяет CRC32
//   4. TUI-визард: имя, автор, версия, прогресс-бар
//   5. Полный .nhs blob копируется в AppSlot[n] (BSS)
//   6. Запись в AppRegistry
//   7. Shell: `install list` показывает реестр установленных apps
//   8. `uninstall <slot>` → очищает слот + реестр
// ============================================================

pub mod catalog;
pub mod crc32;
pub mod header;
pub mod installer;
pub mod nsfs;
pub mod registry;
pub mod runtime;
pub mod serial_recv;
pub mod slots;
