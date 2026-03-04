//
// Perlin Noise
//
// adapted from https://cs.nyu.edu/~perlin/noise/

use crate::utils::lerp;
use psp::math::floorf;

const PERMUTATION: [i32; 256] = [ 151,160,137,91,90,15,
   131,13,201,95,96,53,194,233,7,225,140,36,103,30,69,142,8,99,37,240,21,10,23,
   190, 6,148,247,120,234,75,0,26,197,62,94,252,219,203,117,35,11,32,57,177,33,
   88,237,149,56,87,174,20,125,136,171,168, 68,175,74,165,71,134,139,48,27,166,
   77,146,158,231,83,111,229,122,60,211,133,230,220,105,92,41,55,46,245,40,244,
   102,143,54, 65,25,63,161, 1,216,80,73,209,76,132,187,208, 89,18,169,200,196,
   135,130,116,188,159,86,164,100,109,198,173,186, 3,64,52,217,226,250,124,123,
   5,202,38,147,118,126,255,82,85,212,207,206,59,227,47,16,58,17,182,189,28,42,
   223,183,170,213,119,248,152, 2,44,154,163, 70,221,153,101,155,167, 43,172,9,
   129,22,39,253, 19,98,108,110,79,113,224,232,178,185, 112,104,218,246,97,228,
   251,34,242,193,238,210,144,12,191,179,162,241, 81,51,145,235,249,14,239,107,
   49,192,214, 31,181,199,106,157,184, 84,204,176,115,121,50,45,127, 4,150,254,
   138,236,205,93,222,114,67,29,24,72,243,141,128,195,78,66,215,61,156,180
];

static P: [i32; 512] = {
    let mut arr = [0; 512];
    let mut i = 0;
    while i < 256 {
        arr[i] = PERMUTATION[i];
        arr[256 + i] = PERMUTATION[i];
        i += 1;
    }
    arr
};

pub fn noise(x: f32, y: f32, z: f32) -> f32 {
    // find the unit cube that contains point
    let (xi, yi, zi) = (
        floorf(x) as i32 & 255,
        floorf(y) as i32 & 255,
        floorf(z) as i32 & 255,
    );
    // find the relative xyz of point in cube
    let x = x - floorf(x);
    let y = y - floorf(y);
    let z = z - floorf(z);
    // compute fade curves for each
    let (u, v, w) = (fade(x), fade(y), fade(z));
    // hash the coords of the 8 cube corners
    let a  = (P[xi as usize]     + yi) as usize;
    let b  = (P[(xi+1) as usize] + yi) as usize;
    let aa = (P[a]               + zi) as usize;
    let ab = (P[a+1]             + zi) as usize;
    let ba = (P[b]               + zi) as usize;
    let bb = (P[b+1]             + zi) as usize;

    // Add blended results from 8 corners of the cube
    lerp(w, lerp(v, lerp(u, grad(P[aa  ], x,    y,    z   ),
                            grad(P[ba  ], x-1., y,    z   )),
                    lerp(u, grad(P[ab  ], x,    y-1., z   ),
                            grad(P[bb  ], x-1., y-1., z   ))),
            lerp(v, lerp(u, grad(P[aa+1], x,    y,    z-1.),
                            grad(P[ba+1], x-1., y,    z-1.)),
                    lerp(u, grad(P[ab+1], x,    y-1., z-1.),
                            grad(P[bb+1], x-1., y-1., z-1.))))
}

#[inline]
fn fade(t: f32) -> f32 { t * t * t * (t * (t * 6. - 15.) + 10.) }

#[inline]
fn grad(hash: i32, x: f32, y: f32, z: f32) -> f32 {
    // convert lo 4 bits of hash code into 12 gradient
    let h = hash & 15;
    let u = if h<8 { x } else { y };
    let v = if h<4 { y } else if h==12||h==14 { x } else { z };
    (if (h&1) == 0 { u } else { -u }) + (if (h&2) == 0 { v } else { -v })
}
