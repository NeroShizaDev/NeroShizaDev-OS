// ============================================================
// DOOM STUBS — Глобальный bump-аллокатор + C FFI заглушки
// ============================================================
// 4 MB статической кучи в BSS (нулевые байты — не раздувают бинарник).
// Bump-аллокатор: O(1) alloc, dealloc — no-op.
// Куча сбрасывается через reset_heap() при выходе из Doom.
// ============================================================

use core::alloc::{GlobalAlloc, Layout};
use spin::Mutex;

const HEAP_SIZE: usize = 4 * 1024 * 1024; // 4 МБ в BSS

// SAFETY: нулевая инициализация гарантирована компоновщиком (.bss)
static mut HEAP_STORAGE: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

// ============================================================
// Bump-аллокатор
// ============================================================

struct BumpAllocator {
    ptr: usize,
    end: usize,
}

impl BumpAllocator {
    const fn new() -> Self {
        Self { ptr: 0, end: 0 }
    }

    fn init(&mut self, start: *mut u8, size: usize) {
        self.ptr = start as usize;
        self.end  = self.ptr + size;
    }

    fn reset(&mut self) {
        // Возвращаем указатель на начало — память «освобождена»
        let start = self.end - HEAP_SIZE;
        self.ptr = start;
    }

    /// Выровненное выделение: O(1).
    fn alloc_aligned(&mut self, layout: Layout) -> *mut u8 {
        if self.end == 0 {
            // Куча ещё не инициализирована
            return core::ptr::null_mut();
        }
        let align = layout.align();
        let ptr   = (self.ptr + align - 1) & !(align - 1);
        let end   = ptr.saturating_add(layout.size());
        if end > self.end {
            return core::ptr::null_mut(); // OOM
        }
        self.ptr = end;
        ptr as *mut u8
    }
}

// ============================================================
// Потокобезопасная обёртка (spin::Mutex)
// ============================================================

struct LockedHeap(Mutex<BumpAllocator>);

// SAFETY: BumpAllocator сериализован через spin::Mutex
unsafe impl Sync for LockedHeap {}
unsafe impl Send for LockedHeap {}

unsafe impl GlobalAlloc for LockedHeap {
    /// # Safety
    /// layout.size() > 0 и layout.align() — степень двойки (инварианты Rust allocator API).
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0.lock().alloc_aligned(layout)
    }

    /// # Safety
    /// ptr ранее возвращён alloc(layout). Bump-аллокатор игнорирует dealloc.
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // no-op: память освобождается только через reset_heap()
    }
}

// Единственный глобальный аллокатор для кейты
#[global_allocator]
static HEAP: LockedHeap = LockedHeap(Mutex::new(BumpAllocator::new()));

// ============================================================
// Публичный API
// ============================================================

/// Инициализирует кучу. Вызывать один раз перед doom::run().
pub fn init_heap() {
    // SAFETY: addr_of_mut! даёт сырой указатель без создания &mut на static.
    //         HEAP_STORAGE живёт до конца программы. Доступ сериализован Mutex.
    let start = core::ptr::addr_of_mut!(HEAP_STORAGE) as *mut u8;
    HEAP.0.lock().init(start, HEAP_SIZE);
}

/// Сбрасывает bump-указатель. Освобождает все аллокации Doom разом.
pub fn reset_heap() {
    HEAP.0.lock().reset();
}

// ============================================================
// C FFI — malloc / free / calloc
// Нужны если doom-rs вызывает libc напрямую (через extern "C")
// ============================================================

#[unsafe(no_mangle)]
pub extern "C" fn malloc(size: usize) -> *mut u8 {
    // SAFETY: Layout гарантированно валиден (size >= 1, align = 8)
    unsafe {
        HEAP.alloc(Layout::from_size_align_unchecked(size.max(1), 8))
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn free(_ptr: *mut u8) {
    // Bump-аллокатор: dealloc — no-op
}

#[unsafe(no_mangle)]
pub extern "C" fn calloc(count: usize, size: usize) -> *mut u8 {
    let total = count.saturating_mul(size);
    let ptr   = malloc(total);
    if !ptr.is_null() {
        // SAFETY: ptr указывает на total байт выделенной памяти
        unsafe { core::ptr::write_bytes(ptr, 0, total); }
    }
    ptr
}
