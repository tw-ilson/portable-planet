use alloc::vec::Vec;
use psp::Align16;
use crate::vertex::Vertex;
use crate::utils::slerp;
use crate::sv;


pub fn icosphere(frag_steps: usize) -> Vec<Vertex> {
    let v_cap = 60 * 4_usize.pow(frag_steps as u32);

    // Icosahedron vertex coordinates on a unit sphere.
    // φ = (1+√5)/2; vertices are (0, ±a, ±b), (±a, ±b, 0), (±b, 0, ±a)
    // where a = 1/√(1+φ²), b = φ·a.
    const A: f32 = 0.5257311;
    const B: f32 = 0.8506508;
    static ICO: Align16<[Vertex; 60]> = Align16([
        // Top cap (v0 = (0,A,B))
        sv!( 0.0,  A,  B), sv!( 0.0, -A,  B), sv!(-B,  0.0,  A), // v0,v2,v10
        sv!( 0.0,  A,  B), sv!(-B,  0.0,  A), sv!(-A,  B,  0.0), // v0,v10,v5
        sv!( 0.0,  A,  B), sv!(-A,  B,  0.0), sv!( A,  B,  0.0), // v0,v5,v4
        sv!( 0.0,  A,  B), sv!( A,  B,  0.0), sv!( B,  0.0,  A), // v0,v4,v8
        sv!( 0.0,  A,  B), sv!( B,  0.0,  A), sv!( 0.0, -A,  B), // v0,v8,v2
        // Middle band — upper triangles (2 upper-ring + 1 lower-ring vertex)
        sv!( 0.0, -A,  B), sv!( B,  0.0,  A), sv!( A, -B,  0.0), // v2,v8,v6
        sv!( B,  0.0,  A), sv!( A,  B,  0.0), sv!( B,  0.0, -A), // v8,v4,v9
        sv!( A,  B,  0.0), sv!(-A,  B,  0.0), sv!( 0.0,  A, -B), // v4,v5,v1
        sv!(-A,  B,  0.0), sv!(-B,  0.0,  A), sv!(-B,  0.0, -A), // v5,v10,v11
        sv!(-B,  0.0,  A), sv!( 0.0, -A,  B), sv!(-A, -B,  0.0), // v10,v2,v7
        // Middle band — lower triangles (1 upper-ring + 2 lower-ring vertices)
        sv!( B,  0.0,  A), sv!( B,  0.0, -A), sv!( A, -B,  0.0), // v8,v9,v6
        sv!( 0.0, -A,  B), sv!( A, -B,  0.0), sv!(-A, -B,  0.0), // v2,v6,v7
        sv!(-B,  0.0,  A), sv!(-A, -B,  0.0), sv!(-B,  0.0, -A), // v10,v7,v11
        sv!(-A,  B,  0.0), sv!(-B,  0.0, -A), sv!( 0.0,  A, -B), // v5,v11,v1
        sv!( A,  B,  0.0), sv!( 0.0,  A, -B), sv!( B,  0.0, -A), // v4,v1,v9
        // Bottom cap (v3 = (0,-A,-B))
        sv!( 0.0, -A, -B), sv!( 0.0,  A, -B), sv!(-B,  0.0, -A), // v3,v1,v11
        sv!( 0.0, -A, -B), sv!(-B,  0.0, -A), sv!(-A, -B,  0.0), // v3,v11,v7
        sv!( 0.0, -A, -B), sv!(-A, -B,  0.0), sv!( A, -B,  0.0), // v3,v7,v6
        sv!( 0.0, -A, -B), sv!( A, -B,  0.0), sv!( B,  0.0, -A), // v3,v6,v9
        sv!( 0.0, -A, -B), sv!( B,  0.0, -A), sv!( 0.0,  A, -B), // v3,v9,v1
    ]);
    let mut verts: Vec<Vertex> = Vec::with_capacity(v_cap);
    verts.extend_from_slice(&ICO.0);

    // MESH FRAGMENTATION

    for _ in 0..frag_steps {
        let mut v_temp = Vec::with_capacity(verts.len() * 4);
        for poly in verts.as_slice().chunks_exact(3) {
            let (v1, v2, v3) = (poly[0], poly[1], poly[2]);
            let v4 = slerp(v1, v2, 0.5f32);
            let v5 = slerp(v2, v3, 0.5f32);
            let v6 = slerp(v3, v1, 0.5f32);

            v_temp.extend_from_slice(&[v1, v4, v6]);
            v_temp.extend_from_slice(&[v4, v5, v6]);
            v_temp.extend_from_slice(&[v5, v3, v6]);
            v_temp.extend_from_slice(&[v4, v2, v5]);
        }
        core::mem::swap(&mut verts, &mut v_temp);
        v_temp.clear();
    }

    verts
}
