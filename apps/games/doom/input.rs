// ============================================================
// DOOM INPUT — прямой опрос PS/2 клавиатуры (порт 0x60)
// ============================================================
// Работаем в graphics mode, прерывания клавиатуры заблокированы
// (мы вызваны из обработчика прерывания). Опрос порта 0x60 напрямую.
//
// Маппинг скан-кодов (AT set 1):
//   0x01 — ESC
//   0x11 — W    (вперёд)
//   0x1F — S    (назад)
//   0x1E — A    (влево)
//   0x20 — D    (вправо)
//   0x1C — Enter (выстрел)
//   0x39 — Space (использовать)
//   0x2A, 0x36 — Shift (бег)
//   0x02/0x03/0x04/0x05/0x06/0x07 — 1..6 (режимы огня)
//   0x80+ — release (scancode | 0x80)
// ============================================================

// ============================================================
// Состояние клавиш — глобальные флаги
// ============================================================
static mut DOOM_ACTIVE: bool = false;
static mut KEY_ESC: bool = false;
static mut KEY_FORWARD: bool = false;
static mut KEY_BACK: bool = false;
static mut KEY_LEFT: bool = false;
static mut KEY_RIGHT: bool = false;
static mut KEY_FIRE: bool = false;
static mut KEY_USE: bool = false;
static mut KEY_RUN: bool = false;
static mut KEY_MODE_1: bool = false;
static mut KEY_MODE_2: bool = false;
static mut KEY_MODE_3: bool = false;
static mut KEY_MODE_4: bool = false;
static mut KEY_MODE_5: bool = false;
static mut KEY_MODE_6: bool = false;
static mut KEY_FPU_TOGGLE: bool = false; // \ (0x2B) — переключение FPU

/// Включить/выключить режим Doom (глобальный флаг)
pub fn set_active(active: bool) {
    // SAFETY: bare-metal однопоточное ядро; вызывается до/после game loop,
    // не конкурирует с чтением DOOM_ACTIVE из того же потока.
    unsafe {
        DOOM_ACTIVE = active;
    }
}

pub fn is_active() -> bool {
    // SAFETY: аналогично set_active — единственный поток, без гонок.
    unsafe { DOOM_ACTIVE }
}

/// Опрашивает буфер клавиатуры и обновляет состояние клавиш.
/// Возвращает true если нажата ESC (сигнал выхода из Doom).
pub fn poll() -> bool {
    // SAFETY: KEY_* — static mut; вызывается только из главного game loop
    // (единственный поток). ps2::has_scancode/read_scancode — PS/2 порты,
    // допустимы пока PS/2 контроллер инициализирован BIOS/UEFI.
    unsafe {
        while crate::ps2::has_scancode() {
            let sc = crate::ps2::read_scancode();
            let pressed = sc & 0x80 == 0;
            let code = sc & 0x7F;
            match code {
                0x01 => {
                    if pressed {
                        KEY_ESC = true;
                    }
                }
                0x02 => {
                    if pressed {
                        KEY_MODE_1 = true;
                    }
                }
                0x03 => {
                    if pressed {
                        KEY_MODE_2 = true;
                    }
                }
                0x04 => {
                    if pressed {
                        KEY_MODE_3 = true;
                    }
                }
                0x05 => {
                    if pressed {
                        KEY_MODE_4 = true;
                    }
                }
                0x06 => {
                    if pressed {
                        KEY_MODE_5 = true;
                    }
                }
                0x07 => {
                    if pressed {
                        KEY_MODE_6 = true;
                    }
                }
                0x11 => KEY_FORWARD = pressed,
                0x1F => KEY_BACK = pressed,
                0x1E => KEY_LEFT = pressed,
                0x20 => KEY_RIGHT = pressed,
                0x1C => KEY_FIRE = pressed,
                0x39 => KEY_USE = pressed,
                0x2A | 0x36 => KEY_RUN = pressed,
                0x2B => {
                    if pressed {
                        KEY_FPU_TOGGLE = true;
                    }
                } // \
                _ => {}
            }
        }
        KEY_ESC
    }
}

/// Сбрасывает флаг ESC после обработки.
pub fn clear_esc() {
    // SAFETY: однопоточный доступ — вызывается между кадрами game loop.
    unsafe {
        KEY_ESC = false;
    }
}

/// Сбрасывает всё состояние при выходе из Doom.
pub fn reset() {
    // SAFETY: однопоточный доступ — вызывается после выхода из game loop,
    // когда никто другой не читает KEY_*.
    unsafe {
        KEY_ESC = false;
        KEY_FORWARD = false;
        KEY_BACK = false;
        KEY_LEFT = false;
        KEY_RIGHT = false;
        KEY_FIRE = false;
        KEY_USE = false;
        KEY_RUN = false;
        KEY_MODE_1 = false;
        KEY_MODE_2 = false;
        KEY_MODE_3 = false;
        KEY_MODE_4 = false;
        KEY_MODE_5 = false;
        KEY_MODE_6 = false;
        KEY_FPU_TOGGLE = false;
    }
}

// ============================================================
// Снимки состояния (для игровой логики)
// SAFETY (все геттеры ниже): чтение static mut bool — атомарная операция
// на x86_64 (выровненная 1-байтовая загрузка). Единственный поток —
// гонок данных нет. Аналог volatile-читалок в классических ОС-ядрах.
// ============================================================

pub fn is_forward() -> bool {
    unsafe { KEY_FORWARD }
}
pub fn is_back() -> bool {
    unsafe { KEY_BACK }
}
pub fn is_left() -> bool {
    unsafe { KEY_LEFT }
}
pub fn is_right() -> bool {
    unsafe { KEY_RIGHT }
}
pub fn is_fire() -> bool {
    unsafe { KEY_FIRE }
}
pub fn is_use() -> bool {
    unsafe { KEY_USE }
}
pub fn is_run() -> bool {
    unsafe { KEY_RUN }
}

pub fn take_mode_1() -> bool {
    unsafe {
        let pressed = KEY_MODE_1;
        KEY_MODE_1 = false;
        pressed
    }
}

pub fn take_mode_2() -> bool {
    unsafe {
        let pressed = KEY_MODE_2;
        KEY_MODE_2 = false;
        pressed
    }
}

pub fn take_mode_3() -> bool {
    unsafe {
        let pressed = KEY_MODE_3;
        KEY_MODE_3 = false;
        pressed
    }
}

pub fn take_mode_4() -> bool {
    unsafe {
        let pressed = KEY_MODE_4;
        KEY_MODE_4 = false;
        pressed
    }
}

pub fn take_mode_5() -> bool {
    unsafe {
        let pressed = KEY_MODE_5;
        KEY_MODE_5 = false;
        pressed
    }
}

pub fn take_mode_6() -> bool {
    unsafe {
        let pressed = KEY_MODE_6;
        KEY_MODE_6 = false;
        pressed
    }
}

/// \ — переключение FPU-физики (накапливающий флаг, снимается один раз)
pub fn take_fpu_toggle() -> bool {
    unsafe {
        let pressed = KEY_FPU_TOGGLE;
        KEY_FPU_TOGGLE = false;
        pressed
    }
}
