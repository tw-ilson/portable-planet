use core::ptr;
use psp::sys::{
    SceKernelUtilsMt19937Context,
    sceKernelLibcTime,
    sceKernelUtilsMt19937Init,
    sceKernelUtilsMt19937UInt,
};

pub struct Prng(SceKernelUtilsMt19937Context);

impl Prng {
    pub unsafe fn new() -> Self {
        let mut ctx = SceKernelUtilsMt19937Context { count: 0, state: [0; 624] };
        let seed = sceKernelLibcTime(ptr::null_mut()) as u32;
        sceKernelUtilsMt19937Init(&mut ctx, seed);
        Prng(ctx)
    }

    pub fn next_u32(&mut self) -> u32 {
        unsafe { sceKernelUtilsMt19937UInt(&mut self.0) }
    }

    /// Returns a value in [0, 1)
    pub fn next_f32(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }

    pub fn next_f32_range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + self.next_f32() * (hi - lo)
    }
}
