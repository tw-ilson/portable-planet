
use alloc::vec::Vec;
use crate::vertex::Vertex;
use crate::perlin::noise;
use crate::icosphere::icosphere;
use crate::prng::Prng;

const POLE_A: f32 = 0.5257311;  // icosphere pole axis (0, A, B)
const POLE_B: f32 = 0.8506508;
const POLE_CAP: f32 = 0.95;     // dot-product threshold for polar snow

const DEEP_OCEAN:    f32 = 0.96;
const SHALLOW_WATER: f32 = 1.00;
const BEACH:         f32 = 1.005;
const LOWLAND:       f32 = 1.05;
const SNOW_PEAK:     f32 = 1.07;

const HEIGHT_DIV:  f32 = 8.0;   // terrain displacement scale (higher = flatter)
const NUDGE_FREQ:  f32 = 20.0;  // fine-detail surface noise frequency
const NUDGE_AMP:   f32 = 0.008; // fine-detail surface noise amplitude

fn apply_brightness(color: u32, delta: f32) -> u32 {
    let adj = (delta * 3000.0) as i32;
    let r = (((color      ) & 0xff) as i32 + adj).clamp(0, 255) as u32;
    let g = (((color >>  8) & 0xff) as i32 + adj).clamp(0, 255) as u32;
    let b = (((color >> 16) & 0xff) as i32 + adj).clamp(0, 255) as u32;
    0xff000000 | (b << 16) | (g << 8) | r
}

pub struct Terrain {
    pub shape: Vec<Vertex>
}

impl Terrain {
    pub fn new(freq: f32, amp: f32, rng: &mut Prng) -> Self {
        let mut sphere = icosphere(5);
        // All decorrelation offsets drawn from rng — arbitrary values, no magic numbers
        let (w_jx, w_jy, w_jz) = (rng.next_f32_range(0.0, 100.0), rng.next_f32_range(0.0, 100.0), rng.next_f32_range(0.0, 100.0));
        let (n_x,  n_y,  n_z)  = (rng.next_f32_range(0.0, 100.0), rng.next_f32_range(0.0, 100.0), rng.next_f32_range(0.0, 100.0));

        for v in sphere.iter_mut() {
            // Domain warping: 3 decorrelated noise samples offset the main sample coords
            let jx = noise(v.x * freq + w_jx, v.y * freq, v.z * freq) * amp;
            let jy = noise(v.x * freq + w_jy, v.y * freq, v.z * freq) * amp;
            let jz = noise(v.x * freq + w_jz, v.y * freq, v.z * freq) * amp;
            let offset = noise(v.x * freq + jx, v.y * freq + jy, v.z * freq + jz) / HEIGHT_DIV + 1.0;
            let disp = offset.max(SHALLOW_WATER);  // clamp water to baseline sphere
            // High-frequency position-based nudge — deterministic per vertex position,
            // so shared edge copies compute the same value (no gaps)
            let nx = noise(v.x * NUDGE_FREQ + n_x, v.y * NUDGE_FREQ, v.z * NUDGE_FREQ) * NUDGE_AMP;
            let ny = noise(v.x * NUDGE_FREQ + n_y, v.y * NUDGE_FREQ, v.z * NUDGE_FREQ) * NUDGE_AMP;
            let nz = noise(v.x * NUDGE_FREQ + n_z, v.y * NUDGE_FREQ, v.z * NUDGE_FREQ) * NUDGE_AMP;
            v.x = v.x * disp + nx;
            v.y = v.y * disp + ny;
            v.z = v.z * disp + nz;
            // normals stay as the original unit vectors (radial direction unchanged)
            let pole_dot = v.ny * POLE_A + v.nz * POLE_B;
            let near_pole = pole_dot.abs() > POLE_CAP;
            let base = if near_pole || offset >= SNOW_PEAK {
                0xffeeeeee_u32  // snow: polar caps + high peaks
            } else if offset < DEEP_OCEAN {
                0xff9b5a00  // deep ocean
            } else if offset < SHALLOW_WATER {
                0xffcf8500  // shallow water
            } else if offset < BEACH {
                0xff80b2c2  // beach/sand
            } else if offset < LOWLAND {
                0xff2d7a1f  // lowland
            } else {
                0xff1f4f7a  // highland
            };
            v.color = apply_brightness(base, nx + ny + nz);
        }

        Terrain { shape: sphere }
    }
}
