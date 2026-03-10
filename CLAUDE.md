# PSP Homebrew Notes

## Toolchain

- Build tool: `cargo psp` (from `cargo-psp v0.2.8`)
- Target: `mipsel-sony-psp` (set in `.cargo/config.toml`)
- Requires nightly + `rust-src` component (see `rust-toolchain.toml`)
- Output: `target/mipsel-sony-psp/debug/EBOOT.PBP`
- Upload: `just upload <psp_mount>` — guards against missing `PSP/GAME` dir

## Project Structure

- `src/main.rs` must be `#![no_std] #![no_main]`
- Entry point declared with `psp::module!("name", major, minor)`
- Actual entry point is `fn psp_main()`
- All `sys::sce*` calls are `unsafe`; declare graphics functions as `unsafe fn` and wrap callsites

## Rust 2024 Edition Gotchas

- `unsafe fn` bodies still warn on unsafe ops (`unsafe_op_in_unsafe_fn`) — calls inside need their own `unsafe {}` or suppress the lint
- Mutable statics require `&raw mut` instead of `&mut` (`static_mut_refs` lint is deny-by-default)

## Graphics (GE / sceGu)

- The PSP Graphics Engine (GE) is a separate hardware processor; it executes **display lists** asynchronously
- Display list: flat `[u32]` buffer of 32-bit commands (`[8-bit opcode | 24-bit operand]`)
- Must be 16-byte aligned — use `Align16<[u32; 0x40000]>` (1 MiB, standard frame budget)
- `sceGuStart(ctx, list_ptr)` — begins recording into the list
- `sceGuFinish()` + `sceGuSync(Finish, Wait)` — commits and blocks until GE is done
- Frame loop: `sceGuSwapBuffers()` after sync, `sceDisplayWaitVblankStart()` for vsync

### VRAM Layout

```
fbp0 — draw buffer  (Psm8888, BUF_WIDTH × SCREEN_HEIGHT)
fbp1 — disp buffer  (Psm8888, BUF_WIDTH × SCREEN_HEIGHT)
zbp  — depth buffer (Psm4444, BUF_WIDTH × SCREEN_HEIGHT)
```
Allocated via `psp::vram_alloc::get_vram_allocator()`.

### Vertex Format

Vertex layout must match the `VertexType` flags passed to `sceGumDrawArray`:
- Field order is fixed by hardware: `[color] [normal] [position]`
- `COLOR_8888`     → `color: u32` (ABGR)
- `NORMAL_32BITF`  → `nx: f32, ny: f32, nz: f32`
- `VERTEX_32BITF`  → `x: f32, y: f32, z: f32`
- `TRANSFORM_3D`   → positions go through GUM matrix stack
- `LightComponent` flags are bitflags — combine with `|` (e.g. `DIFFUSE | SPECULAR`)

For per-vertex color with lighting, add `COLOR_8888` to `VertexType` and omit `sceGuMaterial`
for the diffuse component (vertex color drives it). Keep `sceGuMaterial(SPECULAR, ...)` for
a separate white specular highlight.

### Shading

- `ShadingModel::Smooth` — Gouraud: interpolates color/lighting across triangle (blurry biomes)
- `ShadingModel::Flat` — uses last vertex of each triangle; hard polygon edges, good for low-poly

### Color Format

PSP GE colors are **ABGR** (`0xAABBGGRR`):
- Solid red:   `0xff0000ff`
- Solid green: `0xff00ff00`
- Solid blue:  `0xffff0000`

## Module Structure

| File | Purpose |
|------|---------|
| `main.rs` | GU init, render loop, matrix stack, lighting setup |
| `vertex.rs` | `Vertex` struct + `sv!` macro (unit sphere vertex) |
| `icosphere.rs` | Geodesic sphere generation via slerp subdivision |
| `utils.rs` | `lerp`, `slerp` |
| `perlin.rs` | Improved Perlin noise (Ken Perlin reference impl) |
| `terrain.rs` | Procedural planet: domain warp, displacement, biome coloring |
| `prng.rs` | MT19937 wrapper seeded from `sceKernelLibcTime` |

## Procedural Planet Techniques

### Icosphere
- 12 vertices / 20 faces icosahedron, subdivided via `slerp` for geodesic accuracy
- Pole axis is `(0, A, B)` where `A=0.5257311`, `B=0.8506508` — **not** a cardinal axis
- Rotation about pole: change-of-basis `Rx(-α) · Rz(val) · Rx(α)`, `α = POLE_ALIGN = 0.5535744`
- Stored as flat non-indexed triangle list — shared edge vertices are **duplicated**

### Terrain Generation
- Radial displacement via Perlin noise: `offset = noise(...) / HEIGHT_DIV + 1.0`
- Domain warping: 3 decorrelated noise samples offset the main sample coordinates
- Decorrelation offsets must be arbitrary (not meaningful) — draw from PRNG, not hardcoded
- Water areas clamped to baseline (`offset.max(SHALLOW_WATER)`) for flat ocean surface
- Normals stay as pre-displacement unit vectors (radial direction unchanged after displacement)

### Non-indexed mesh pitfall
- Any per-vertex random displacement creates gaps at shared edges (duplicate vertices get different values)
- Fix: use **position-based noise** for per-vertex effects — same position → same value regardless of duplication
- True random per-vertex effects require an indexed mesh

### Biome coloring (`terrain.rs`)
Thresholds on displacement `offset`:
```
< DEEP_OCEAN    — deep ocean
< SHALLOW_WATER — shallow water
< BEACH         — beach/sand (thin strip just above waterline)
< LOWLAND       — lowland/grass
< SNOW_PEAK     — highland
≥ SNOW_PEAK     — snow (also: near_pole check via dot product with pole axis)
```
Polar caps: `dot(normal, pole_axis).abs() > POLE_CAP`

### PRNG
- `sceKernelUtilsMt19937Context` is 2500 bytes — fine on PSP stack
- Seed with `sceKernelLibcTime(null_mut())` for per-boot variation
- **Never** call `Prng::new()` inside a vertex loop — it's a kernel syscall
- Draw all per-planet random values once before the loop; use position-based noise inside

## PSPLink / Debugging

### Setup (already done)

- PSP-side: PSPLink v3.2.1 installed at `PSP/GAME/PSPLINK/`; reinstall with `just install-psplink /media/thomas/disk`
- PC-side tools in `~/pspdev/bin/`: `usbhostfs_pc` (pre-built) and `pspsh` (rebuilt from source — pre-built requires GLIBC_2.38, incompatible with Ubuntu 22.04)
- udev rule installed at `/etc/udev/rules.d/50-psplink.rules` (Sony VID `054c`, PID `01c9`)
- `psplink.ini`: `pluser=1` (required for user-mode homebrew), `resetonexit=1`

### Iteration Workflow

```bash
just psplink-start   # start usbhostfs_pc in background (once per session)
just run             # reset PSPLink → cargo psp → load PSP-Test.prx
just psplink-stop    # kill usbhostfs_pc
```

- `usbhostfs_pc` serves `target/mipsel-sony-psp/debug/` as `host0:` on the PSP
- PSPLink loads the `.prx`, not the `.PBP` — `EBOOT.PBP` is XMB-only
- `tail -f /tmp/usbhostfs_pc.log` to inspect host filesystem daemon output

### GDB

- GDB server started on PSP with: `debug <program.elf>` from `pspsh`
- PC side: `psp-gdb <elf>` → `target remote :10001`
- Set breakpoint at `main`, not `_start` (stepping through `_start` hangs PSPLink)
- `calc $epc-$mod` — relative offset for `psp-addr2line` after a crash
- `disasm $epc` — disassemble at exception program counter
