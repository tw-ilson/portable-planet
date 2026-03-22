# portable-planet

Procedural planet generator for PSP homebrew

## Dependencies

- Rust nightly with `rust-src` component
- `cargo-psp` (v0.2.8+)
- PSPLink v3 installed on PSP at `PSP/GAME/PSPLINK/` (see https://pspdev.github.io/debugging.html)
- `usbhostfs_pc` and `pspsh` in `~/pspdev/bin/`

## Setup

1. Install PSPLink to PSP: `just install-psplink /path/to/psp`
2. Boot PSPLink on PSP
3. Connect PSP via USB

## Commands

```bash
just psplink-start   # start USB host filesystem daemon (needed for `just run`)
just run             # build and load .prx over USB
just psplink-stop    # stop daemon
just upload <path>   # copy EBOOT.PBP to PSP filesystem
```

## Development

`just run` resets PSPLink, builds with `cargo psp`, and loads the `.prx` over USB for fast iteration.
