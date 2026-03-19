use psp::Align16;
use crate::perlin::noise;
use crate::prng::Prng;

pub const TEX_W: usize = 512;
pub const TEX_H: usize = 256;

const LOW_SCALE:  f32 = 5.0;
const HIGH_SCALE: f32 = 350.0;

const STAR_THRESHOLD: f32 = 0.4;

pub static mut PIXELS: Align16<[u32; TEX_W * TEX_H]> = Align16([0; TEX_W * TEX_H]);

pub unsafe fn generate(rng: &mut Prng) {
    let ox1 = rng.next_f32_range(0.0, 100.0);
    let oy1 = rng.next_f32_range(0.0, 100.0);
    let ox2 = rng.next_f32_range(0.0, 100.0);
    let oy2 = rng.next_f32_range(0.0, 100.0);

    for row in 0..TEX_H {
        for col in 0..TEX_W {
            let u = col as f32 / TEX_W as f32;
            let v = row as f32 / TEX_H as f32;

            // Low-frequency field: broad density regions
            let low  = noise(u * LOW_SCALE  + ox1, v * LOW_SCALE  + oy1, 0.0);
            let density = (low + 1.0) * 0.5;

            // High-frequency field: isolated ~1px dots
            let high = noise(u * HIGH_SCALE + ox2, v * HIGH_SCALE + oy2, 50.0);
            let detail = (high + 1.0) * 0.5;

            let combined = density * detail;
            let pixel = if combined > STAR_THRESHOLD {
                let t = (combined - STAR_THRESHOLD) / (1.0 - STAR_THRESHOLD);
                let b = (t * 255.0) as u32;
                0xff000000 | (b << 16) | (b << 8) | b
            } else {
                0xff000000
            };

            // Write in swizzled layout: GE fetches 16-byte × 8-row blocks;
            // storing them contiguously (4 u32 wide × 8 rows = 32 u32 per block)
            // eliminates cache thrashing on RAM textures.
            const ROW_BLOCKS: usize = TEX_W / 4;
            let block_idx = (col / 4) + ROW_BLOCKS * (row / 8);
            let swizzled_idx = block_idx * 32 + (row % 8) * 4 + (col % 4);
            (&raw mut PIXELS).cast::<u32>().add(swizzled_idx).write(pixel);
        }
    }
}
