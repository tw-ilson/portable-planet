#[repr(C, align(4))]
#[derive(Copy, Clone)]
pub struct Vertex {
    pub color: u32,
    pub nx: f32, pub ny: f32, pub nz: f32,
    pub x: f32, pub y: f32, pub z: f32,
}

// On a unit sphere the outward normal equals the position.
#[macro_export]
macro_rules! sv {
    ($x:expr, $y:expr, $z:expr) => {
        Vertex { color: 0xffffffff, nx: $x, ny: $y, nz: $z, x: $x, y: $y, z: $z }
    };
}

impl Vertex {
    pub fn dot_prod(&self, other: &Vertex) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn scalar_prod(&self, m: f32) -> Vertex {
        sv!(self.x * m, self.y * m, self.z *m)
    }

    pub fn vector_add(&self, other: &Vertex) -> Vertex {
        sv!(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}
