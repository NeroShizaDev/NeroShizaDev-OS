// ============================================================
// ГУБКА МЕНГЕРА v3 — с нормалями и освещением
// ============================================================
// Трёхмерность за счёт: нормалей поверхности + направленного света.
// Normal = градиент DE методом конечных разностей.
// Brightness = dot(normal, light_dir) → палитра символов.
// ============================================================

use core::arch::asm;

const W: usize = 80;
const H: usize = 25;
const VGA: usize = 0xB8000;
const MARCH_STEPS: u32 = 48;

// 10 символов от пустого к плотному — чистый градиент
static SHADE: &[u8] = b" .,:;+*%#@";

// Fixed-point: *1024
const FP: i64 = 1024;

fn iabs(x: i64) -> i64 { if x < 0 { -x } else { x } }
fn imax(a: i64, b: i64) -> i64 { if a > b { a } else { b } }
fn imin(a: i64, b: i64) -> i64 { if a < b { a } else { b } }

// ============================================================
// Menger Sponge Distance Estimator (Inigo Quilez)
// ============================================================

fn de(x: i64, y: i64, z: i64) -> i64 {
    // Box: max(|x|-1, |y|-1, |z|-1)
    let mut d = imax(imax(iabs(x) - FP, iabs(y) - FP), iabs(z) - FP);

    let mut s: i64 = FP;
    for _ in 0..3 {
        // a = mod(pos * s, 2.0) - 1.0   (в FP: mod(pos*s/FP, 2*FP) - FP)
        let ps = s;
        let ax = ((x * ps / FP) % (2 * FP) + 3 * FP) % (2 * FP) - FP;
        let ay = ((y * ps / FP) % (2 * FP) + 3 * FP) % (2 * FP) - FP;
        let az = ((z * ps / FP) % (2 * FP) + 3 * FP) % (2 * FP) - FP;

        s *= 3;

        let rx = iabs(FP - 3 * iabs(ax));
        let ry = iabs(FP - 3 * iabs(ay));
        let rz = iabs(FP - 3 * iabs(az));

        let c = (imin(imin(imax(rx, ry), imax(ry, rz)), imax(rz, rx)) - FP) * FP / s;
        d = imax(d, c);
    }
    d
}

// ============================================================
// Нормаль поверхности — градиент DE (конечные разности)
// Возвращает (nx, ny, nz) * FP (не нормализованные, но достаточно)
// ============================================================

const EPS: i64 = 4;

fn normal(x: i64, y: i64, z: i64) -> (i64, i64, i64) {
    let nx = de(x + EPS, y, z) - de(x - EPS, y, z);
    let ny = de(x, y + EPS, z) - de(x, y - EPS, z);
    let nz = de(x, y, z + EPS) - de(x, y, z - EPS);
    (nx, ny, nz)
}

// ============================================================
// isqrt — целочисленный квадратный корень (Ньютон)
// ============================================================

fn isqrt(val: i64) -> i64 {
    if val <= 0 { return 0; }
    let mut x = val;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + val / x) / 2;
    }
    x
}

// ============================================================
// FPU sin/cos
// ============================================================

fn fpu_sin(a: i64) -> i64 {
    let r: i64;
    // SAFETY: x87 FPU FSIN — доступен в long mode (CPUID).
    // push/pop через rsp явный: fild читает из [rsp], fistp пишет в [rsp].
    // x87-стек: push push fdivp (2→1) fsin (1→1) push fmulp (2→1) fistp (1→0) — баланс 0.
    // add rsp,16/8 вручную корректируют RSP после наших push.
    unsafe {
        asm!(
            "push {a}", "fild qword ptr [rsp]",
            "push {k}", "fild qword ptr [rsp]", "add rsp, 16",
            "fdivp", "fsin",
            "push {k}", "fild qword ptr [rsp]", "add rsp, 8",
            "fmulp", "sub rsp, 8", "fistp qword ptr [rsp]", "pop {r}",
            a = in(reg) a, k = in(reg) FP, r = out(reg) r,
        );
    }
    r
}

fn fpu_cos(a: i64) -> i64 {
    let r: i64;
    // SAFETY: аналогично fpu_sin — x87 FCOS; баланс стека идентичен.
    unsafe {
        asm!(
            "push {a}", "fild qword ptr [rsp]",
            "push {k}", "fild qword ptr [rsp]", "add rsp, 16",
            "fdivp", "fcos",
            "push {k}", "fild qword ptr [rsp]", "add rsp, 8",
            "fmulp", "sub rsp, 8", "fistp qword ptr [rsp]", "pop {r}",
            a = in(reg) a, k = in(reg) FP, r = out(reg) r,
        );
    }
    r
}

// ============================================================
// РЕНДЕР КАДРА
// ============================================================

fn render(angle: i64) {
    let vga = VGA as *mut u8;

    let sa = fpu_sin(angle);
    let ca = fpu_cos(angle);
    // Лёгкий тилт: ~15°
    let sb = fpu_sin(260);
    let cb = fpu_cos(260);

    // Направление света: (577, 577, -577) ≈ normalize(1,1,-1) * FP
    // |v| = sqrt(3) ≈ 1.73, 1024/1.73 ≈ 591
    let lx: i64 = 591;
    let ly: i64 = 591;
    let lz: i64 = -591;

    for row in 0..H {
        for col in 0..W {
            // UV: aspect ratio 2:1 для текста (символ ~вдвое выше ширины)
            let u = (col as i64 * 2 - W as i64) * FP / W as i64;
            let v = (H as i64 - row as i64 * 2) * FP / H as i64;

            // Луч: направление
            let mut dx = u;
            let mut dy = v;
            let dz: i64 = FP * 3 / 2; // фокус 1.5

            // Вращение Y
            let dx2 = (dx * ca - dz * sa) / FP;
            let dz2 = (dx * sa + dz * ca) / FP;
            dx = dx2;
            // Вращение X (тилт)
            let dy2 = (dy * cb - dz2 * sb) / FP;
            let dz3 = (dy * sb + dz2 * cb) / FP;
            dy = dy2;

            // Нормализация направления
            let len2 = dx * dx + dy * dy + dz3 * dz3;
            let len = isqrt(len2);
            if len == 0 {
                let off = (row * W + col) * 2;
                unsafe { *vga.add(off) = b' '; *vga.add(off + 1) = 0; }
                continue;
            }
            let ndx = dx * FP / len;
            let ndy = dy * FP / len;
            let ndz = dz3 * FP / len;

            // Камера
            let mut px: i64 = 0;
            let mut py: i64 = 0;
            let mut pz: i64 = -FP * 5 / 2; // z = -2.5

            let mut t: i64 = 0;
            let mut hit = false;

            for _ in 0..MARCH_STEPS {
                let d = de(px, py, pz);
                if d < 2 {
                    hit = true;
                    break;
                }
                if t > FP * 10 { break; }
                let step = if d < 5 { 5 } else { d };
                px += ndx * step / FP;
                py += ndy * step / FP;
                pz += ndz * step / FP;
                t += step;
            }

            let (ch, attr) = if hit {
                // Нормаль поверхности
                let (nx, ny, nz) = normal(px, py, pz);
                let nlen2 = nx * nx + ny * ny + nz * nz;
                let nlen = isqrt(nlen2);

                let bright = if nlen > 0 {
                    // dot(normal, light)
                    let dot = (nx * lx + ny * ly + nz * lz) / nlen;
                    // clamp к [0, FP]
                    let clamped = if dot < 0 { 0 } else if dot > FP { FP } else { dot };
                    // Ambient 20% + diffuse 80%
                    (FP / 5) + clamped * 4 / 5
                } else {
                    FP / 3
                };

                // Индекс символа: bright ∈ [0, FP] → [0, 9]
                let idx = (bright * 9 / FP) as usize;
                let idx = if idx > 9 { 9 } else { idx };

                // Цвет по яркости: тёмный=жёлтый, средний=белый, яркий=белый+bold
                let color = if idx < 3 {
                    0x06 // тёмно-коричневый (тень)
                } else if idx < 6 {
                    0x0E // жёлтый
                } else {
                    0x0F // яркий белый (свет)
                };

                (SHADE[idx], color)
            } else {
                (b' ', 0x00)
            };

            let off = (row * W + col) * 2;
            // SAFETY: off = (row*W + col)*2, row<H=25, col<W=80 → off < 25*80*2 = 4000.
            // vga = 0xB8000 identity-mapped; прямая запись без volatile допустима
            // здесь т.к. WRITER.lock() не используется (menger-loop без блокировок).
            unsafe {
                *vga.add(off) = ch;
                *vga.add(off + 1) = attr;
            }
        }
    }
}

// ============================================================
// API
// ============================================================

pub fn run_demo() {
    crate::vga_buffer::clear_screen();

    // SAFETY: ps2::read_scancode — порт 0x60, сдвигает FIFO контроллера.
    // Bare-metal ядро: единственный потребитель, нет конкурентных читателей.
    let mut last_sc: u8 = unsafe { crate::ps2::read_scancode() };
    for _ in 0..50000u32 {
        let sc: u8 = unsafe { crate::ps2::read_scancode() };
        if sc & 0x80 != 0 { last_sc = sc; break; }
        last_sc = sc;
    }

    let mut angle: i64 = 0;

    loop {
        render(angle);

        angle += 30; // ~1.7° за кадр
        if angle > 6434 { angle -= 6434; }

        // SAFETY: ps2::read_scancode — порт 0x60, проверяем скан-код для выхода.
        let sc: u8 = unsafe { crate::ps2::read_scancode() };
        if sc != last_sc && sc & 0x80 == 0 { break; }
        last_sc = sc;

        // SAFETY: ps2::wait_vblank — порт 0x3DA, ожидаем vblank без разрывов.
        // Чтение сбрасывает AC flip-flop как побочный эффект (ожидаемо).
        unsafe { crate::ps2::wait_vblank(); }
    }

    crate::vga_buffer::clear_screen();
    crate::locale::print_localized_line(
        crate::user_messages::current(crate::user_messages::UiText::MengerDone),
        0x0B,
    );
    crate::print!("> ");
}
