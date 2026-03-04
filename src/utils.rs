use psp::math::{acosf, sinf};
use crate::vertex::Vertex;

pub fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
}

pub fn slerp(v1: Vertex, v2: Vertex, t: f32) -> Vertex {
    if t < 0. || t > 1. {
        return v1;
    }

    let omega = acosf(v1.dot_prod(&v2));
    if omega < 1e-4 {
        return v1
    }

    unsafe {
        v1.scalar_prod(sinf((1.0-t)*omega) / sinf(omega))
            .vector_add(&v2.scalar_prod(sinf(t*omega) / sinf(omega)))
    }
}
