use std::{env, path::PathBuf};

fn main() {
    let mut args = env::args_os().skip(1);
    let mode = args.next().expect("missing boot mode");
    let kernel_path = PathBuf::from(args.next().expect("missing kernel path"));
    let image_path = PathBuf::from(args.next().expect("missing image path"));

    if args.next().is_some() {
        panic!("unexpected extra arguments");
    }

    // Запрашиваем минимальное разрешение 1920×1080.
    // Если UEFI GOP не поддерживает — загрузчик выберет наибольшее доступное.
    let mut boot_config = bootloader::BootConfig::default();
    boot_config.frame_buffer.minimum_framebuffer_width = Some(1920);
    boot_config.frame_buffer.minimum_framebuffer_height = Some(1080);
    // Убираем логгинг загрузчика на экран — ядро само управляет framebuffer.
    boot_config.frame_buffer_logging = false;

    match mode.to_string_lossy().to_ascii_lowercase().as_str() {
        "bios" => {
            let bios = bootloader::BiosBoot::new(&kernel_path);
            bios.create_disk_image(&image_path)
                .expect("failed to create BIOS disk image");
            println!("created BIOS image: {}", image_path.display());
        }
        "uefi" => {
            let mut uefi = bootloader::UefiBoot::new(&kernel_path);
            uefi.set_boot_config(&boot_config);
            uefi.create_disk_image(&image_path)
                .expect("failed to create UEFI disk image");
            println!("created UEFI image: {}", image_path.display());
        }
        other => panic!("unsupported boot mode: {other}"),
    }
}