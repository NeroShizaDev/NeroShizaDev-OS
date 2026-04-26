// ============================================================
// DOOM STUBS — Глобальный bump-аллокатор + C FFI заглушки
// ============================================================
// 4 MB статической кучи в BSS (нулевые байты — не раздувают бинарник).
// Bump-аллокатор: O(1) alloc, dealloc — no-op.
// Куча сбрасывается через reset_heap() при выходе из Doom.
// ============================================================

use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;

const HEAP_SIZE: usize = 4 * 1024 * 1024; // 4 МБ в BSS

// SAFETY: нулевая инициализация гарантирована компоновщиком (.bss)
static mut HEAP_STORAGE: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
static HEAP_USED: AtomicUsize = AtomicUsize::new(0);
static HEAP_PEAK: AtomicUsize = AtomicUsize::new(0);
static HEAP_OOM: AtomicUsize = AtomicUsize::new(0);
static HEAP_LAST_OOM_REQ: AtomicUsize = AtomicUsize::new(0);

// ============================================================
// Bump-аллокатор
// ============================================================

struct BumpAllocator {
    start: usize,
    ptr: usize,
    end: usize,
    ready: bool,
}

impl BumpAllocator {
    const fn new() -> Self {
        Self {
            start: 0,
            ptr: 0,
            end: 0,
            ready: false,
        }
    }

    // Идемпотентна: повторный вызов — no-op, уже живые аллокации не трогает.
    fn init(&mut self, start: *mut u8, size: usize) {
        if self.ready {
            return;
        }
        self.start = start as usize;
        self.ptr = self.start;
        self.end = self.start + size;
        self.ready = true;
    }

    fn reset(&mut self) {
        // Сбрасывает bump-указатель — все аллокации «освобождены» разом.
        // Безопасно вызывать только если ready == true.
        if !self.ready {
            return;
        }
        self.ptr = self.start;
    }

    /// Выровненное выделение: O(1).
    fn alloc_aligned(&mut self, layout: Layout) -> *mut u8 {
        if !self.ready {
            return core::ptr::null_mut();
        }
        let align = layout.align();
        let ptr = (self.ptr + align - 1) & !(align - 1);
        let end = ptr.saturating_add(layout.size());
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
        let (ptr, used) = {
            let mut heap = self.0.lock();
            let ptr = heap.alloc_aligned(layout);
            let used = if ptr.is_null() {
                HEAP_USED.load(Ordering::Acquire)
            } else {
                heap.ptr.saturating_sub(heap.start)
            };
            (ptr, used)
        };
        if ptr.is_null() {
            HEAP_OOM.fetch_add(1, Ordering::AcqRel);
            HEAP_LAST_OOM_REQ.store(layout.size(), Ordering::Release);
            crate::serial_println!(
                "[MEM][ALLOC][OOM] size={} align={} used={} total={}",
                layout.size(),
                layout.align(),
                HEAP_USED.load(Ordering::Acquire),
                HEAP_SIZE
            );
            return ptr;
        }

        HEAP_USED.store(used, Ordering::Release);
        let mut peak = HEAP_PEAK.load(Ordering::Acquire);
        while used > peak {
            match HEAP_PEAK.compare_exchange_weak(peak, used, Ordering::AcqRel, Ordering::Acquire) {
                Ok(_) => break,
                Err(actual) => peak = actual,
            }
        }
        crate::serial_println!(
            "[MEM][ALLOC] ptr={:#x} size={} align={} used={} peak={}",
            ptr as usize,
            layout.size(),
            layout.align(),
            used,
            HEAP_PEAK.load(Ordering::Acquire)
        );
        ptr
    }

    /// # Safety
    /// ptr ранее возвращён alloc(layout). Bump-аллокатор игнорирует dealloc.
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        crate::serial_println!(
            "[MEM][FREE-NOOP] ptr={:#x} size={} align={}",
            _ptr as usize,
            _layout.size(),
            _layout.align()
        );
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
    HEAP_USED.store(0, Ordering::Release);
    HEAP_PEAK.store(0, Ordering::Release);
    crate::serial_println!(
        "[MEM][HEAP] init start={:#x} size={}",
        start as usize,
        HEAP_SIZE
    );
}

/// Сбрасывает bump-указатель. Освобождает все аллокации Doom разом.
pub fn reset_heap() {
    let used = HEAP_USED.load(Ordering::Acquire);
    HEAP.0.lock().reset();
    HEAP_USED.store(0, Ordering::Release);
    crate::serial_println!(
        "[MEM][HEAP] reset released={} peak={}",
        used,
        HEAP_PEAK.load(Ordering::Acquire)
    );
}

/// Метрики кучи — возвращают 0 пока Doom не инициализирован.
pub fn heap_used_bytes() -> usize {
    HEAP_USED.load(Ordering::Acquire)
}
pub fn heap_total_bytes() -> usize {
    HEAP_SIZE
}
pub fn heap_peak_used_bytes() -> usize {
    HEAP_PEAK.load(Ordering::Acquire)
}
pub fn heap_oom_count() -> usize {
    HEAP_OOM.load(Ordering::Acquire)
}
pub fn heap_last_oom_request() -> usize {
    HEAP_LAST_OOM_REQ.load(Ordering::Acquire)
}

// ============================================================
// C FFI — malloc / free / calloc
// Нужны если doom-rs вызывает libc напрямую (через extern "C")
// ============================================================

#[unsafe(no_mangle)]
pub extern "C" fn malloc(size: usize) -> *mut u8 {
    // SAFETY: Layout гарантированно валиден (size >= 1, align = 8)
    unsafe { HEAP.alloc(Layout::from_size_align_unchecked(size.max(1), 8)) }
}

#[unsafe(no_mangle)]
pub extern "C" fn free(_ptr: *mut u8) {
    // Bump-аллокатор: dealloc — no-op
}

#[unsafe(no_mangle)]
pub extern "C" fn calloc(count: usize, size: usize) -> *mut u8 {
    let total = count.saturating_mul(size);
    let ptr = malloc(total);
    if !ptr.is_null() {
        // SAFETY: ptr указывает на total байт выделенной памяти
        unsafe {
            core::ptr::write_bytes(ptr, 0, total);
        }
    }
    ptr
}
