#[cfg(target_arch = "x86_64")]
#[no_mangle]
#[percpu_macros::def_percpu]
static SELF_PTR: usize = 0;

pub fn init_percpu_reg(cpu_id: usize) {
    let tp = percpu_area_base(cpu_id);
    unsafe { write_percpu_reg(tp) }
}

pub fn init() -> usize {
    return 1;
}

pub fn percpu_area_base(_cpu_id: usize) -> usize {
    unsafe extern "C"{
        fn __start_percpu();
    }
    __start_percpu as usize
}

/// Writes the architecture-specific per-CPU data register.
///
/// This register is used to hold the per-CPU data base on each CPU.
///
/// # Safety
///
/// This function is unsafe because it writes the low-level register directly.
pub unsafe fn write_percpu_reg(tp: usize) {
    unsafe {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "x86_64")] {
                if cfg!(target_os = "linux") {
                    const ARCH_SET_GS: u32 = 0x1001;
                    const SYS_ARCH_PRCTL: u32 = 158;
                    core::arch::asm!(
                        "syscall",
                        in("eax") SYS_ARCH_PRCTL,
                        in("edi") ARCH_SET_GS,
                        in("rsi") tp,
                    );
                } else if cfg!(target_os = "none") {
                    x86::msr::wrmsr(x86::msr::IA32_GS_BASE, tp as u64);
                } else {
                    unimplemented!()
                }
                SELF_PTR.write_current_raw(tp);
            } else if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
                core::arch::asm!("mv gp, {}", in(reg) tp)
            } else if #[cfg(all(target_arch = "aarch64", not(feature = "arm-el2")))] {
                core::arch::asm!("msr TPIDR_EL1, {}", in(reg) tp)
            } else if #[cfg(all(target_arch = "aarch64", feature = "arm-el2"))] {
                core::arch::asm!("msr TPIDR_EL2, {}", in(reg) tp)
            } else if #[cfg(target_arch = "loongarch64")] {
                core::arch::asm!("move $r21, {}", in(reg) tp)
            }
        }
    }
}

