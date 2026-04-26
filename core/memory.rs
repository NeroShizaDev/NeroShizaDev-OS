use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
    },
};

/// Initialize a new OffsetPageTable.
///
/// SAFETY: физическая память полностью отображена в виртуальную начиная
/// с `physical_memory_offset`. Функция должна вызываться ровно один раз.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    unsafe {
        let level_4_table = active_level_4_table(physical_memory_offset);
        OffsetPageTable::new(level_4_table, physical_memory_offset)
    }
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;
    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    unsafe { &mut *page_table_ptr }
}

/// Создаёт тестовое отображение страницы на фрейм 0xb8000.
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;
    let map_to_result = unsafe { mapper.map_to(page, frame, flags, frame_allocator) };
    map_to_result.expect("map_to failed").flush();
}

/// FrameAllocator, который всегда возвращает None (заглушка).
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}

/// FrameAllocator из карты памяти загрузчика (bootloader_api 0.11).
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryRegions,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// SAFETY: карта памяти должна быть валидна; все Usable-регионы
    /// должны быть действительно свободны.
    pub unsafe fn init(memory_map: &'static MemoryRegions) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> + '_ {
        // bootloader_api 0.11: MemoryRegion.kind + start/end (байты)
        self.memory_map
            .iter()
            .filter(|r| r.kind == MemoryRegionKind::Usable)
            .flat_map(|r| (r.start..r.end).step_by(4096))
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        match frame {
            Some(frame) => crate::serial_println!(
                "[MEM][FRAME] idx={} phys={:#x}",
                self.next - 1,
                frame.start_address().as_u64()
            ),
            None => crate::serial_println!(
                "[MEM][FRAME][NONE] idx={} usable_exhausted=1",
                self.next - 1
            ),
        }
        frame
    }
}

// ============================================================
// VGA MEMORY MAPPING
// ============================================================
// Identity-маппинг VGA-региона 0xA0000-0xBFFFF (128 КБ = 32 страницы).
// Включает:
//   0xA0000-0xAFFFF — Mode 13h graphics (для Doom)
//   0xB8000-0xB8FFF — VGA text buffer (основной вывод ядра)
// ============================================================

/// Identity-маппит VGA видеопамять (0xA0000..0xC0000) в page tables.
/// Bootloader 0.11 с Dynamic-маппингом физической памяти НЕ делает этого
/// автоматически для низких адресов, поэтому мы добавляем вручную.
pub fn map_vga_memory(
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let flags = Flags::PRESENT | Flags::WRITABLE;
    // 0xA0000..0xC0000 = 32 страницы по 4 КБ
    for page_addr in (0xA0000u64..0xC0000).step_by(4096) {
        let page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(page_addr));
        let frame = PhysFrame::containing_address(PhysAddr::new(page_addr));
        // Если страница уже замаплена (bootloader мог замапить) — пропускаем.
        if mapper.translate_page(page).is_ok() {
            continue;
        }
        let result = unsafe { mapper.map_to(page, frame, flags, frame_allocator) };
        result.expect("map_to VGA failed").flush();
    }
}
