# Technology Stack

**Analysis Date:** 2026-03-06

## Languages

**Primary:**
- Rust (Edition 2024) - Entire codebase. All source files are `.rs` under `src/`.

**Secondary:**
- None. No scripting languages, no build scripts, no web frontend.

## Runtime

**Environment:**
- Native binary compiled to platform target (Linux, macOS, Windows)
- MSRV (minimum supported Rust version): 1.85 (set in `Cargo.toml` via `rust-version = "1.85"`)
- Current toolchain on this machine: rustc 1.93.1

**Package Manager:**
- Cargo 1.93.1
- Lockfile: `Cargo.lock` present and committed (83KB, pinning exact versions)

## Frameworks

**Core:**
- ratatui 0.29.0 - Terminal UI framework. Provides layout, widgets, styling, and the rendering loop. Used in all `src/ui/*.rs` files and `src/main.rs`.
- crossterm 0.28.1 - Terminal backend for ratatui. Handles raw mode, alternate screen, keyboard input, and terminal size queries.

**Testing:**
- Built-in `#[cfg(test)]` modules with `#[test]` functions. No external test framework.
- Tests run via `cargo test`.

**Build/Dev:**
- Cargo (standard Rust build system)
- Release profile with LTO, stripping, and single codegen unit (`Cargo.toml` `[profile.release]`)

## Key Dependencies

**Critical:**
- `pdbtbx` 0.12.0 (with `rstar` feature) - Parses PDB and mmCIF structural biology file formats. Core to loading protein data. Used in `src/parser/pdb.rs`.
- `ratatui` 0.29.0 - TUI framework. The entire UI layer depends on it.
- `crossterm` 0.28.1 - Terminal I/O backend.
- `ratatui-image` 9.0.0 (with `crossterm` feature) - Graphics protocol support (Sixel, Kitty, iTerm2). Enables HD pixel rendering in `src/ui/viewport.rs`.
- `image` 0.25.9 - Image buffer manipulation. Used to convert the software framebuffer to `RgbImage` for graphics protocol output (`src/render/framebuffer.rs`).

**Infrastructure:**
- `clap` 4.5.60 (with `derive` feature) - CLI argument parsing via derive macros. Used in `src/main.rs` for the `Cli` struct.
- `anyhow` 1.0.102 - Error handling with context. Used throughout for `Result<T>` returns.

**Optional (behind `fetch` feature flag):**
- `reqwest` 0.12.28 (with `blocking` feature) - HTTP client for fetching PDB files from RCSB. Used in `src/parser/fetch.rs`.
- `tokio` 1.50.0 (with `rt`, `macros` features) - Async runtime required by reqwest. Only pulled in when `fetch` feature is enabled.

## Feature Flags

Defined in `Cargo.toml`:
```toml
[features]
default = []
fetch = ["dep:reqwest", "dep:tokio"]
```

- `default` - No optional features. Builds a lean binary with local-file-only support.
- `fetch` - Enables `--fetch <PDB_ID>` to download structures from RCSB PDB via HTTP. Adds reqwest and tokio as dependencies.

Build with features:
```bash
cargo build --release                # default (no fetch)
cargo build --release --features fetch  # with RCSB fetch
```

## Configuration

**Environment:**
- No environment variables required for basic operation.
- No `.env` file. No secrets. No API keys for default build.
- The `fetch` feature contacts `https://files.rcsb.org/download/` (public, no auth required).

**Build:**
- `Cargo.toml` - Package metadata, dependencies, features, release profile
- `Cargo.lock` - Exact dependency versions
- No `build.rs` build script
- No `.cargo/config.toml`
- No `rust-toolchain.toml` (relies on MSRV field in Cargo.toml)

**Release Profile:**
```toml
[profile.release]
lto = true        # Link-time optimization for smaller binary
strip = true      # Strip debug symbols
codegen-units = 1 # Single codegen unit for better optimization
```

## Platform Requirements

**Development:**
- Rust 1.85+ (Edition 2024)
- Any terminal emulator (braille rendering works everywhere with Unicode support)
- For HD mode development: terminal with Sixel or Kitty graphics protocol (WezTerm, Kitty, foot, iTerm2)

**Production:**
- Single static binary, zero runtime dependencies
- Linux, macOS, or Windows
- Terminal with Unicode support (minimum)
- Terminal with Sixel/Kitty/iTerm2 graphics protocol (for HD mode, optional)

## Software Rendering

The application implements its own 3D software rasterizer entirely in Rust with no GPU dependencies:
- `src/render/framebuffer.rs` - RGB framebuffer with z-buffer, triangle rasterization, Bresenham line drawing
- `src/render/ribbon.rs` - Catmull-Rom spline interpolation, cross-section extrusion, triangle mesh generation
- `src/render/camera.rs` - 3D rotation matrices, orthographic projection
- `src/render/hd.rs` - High-level rasterization coordinator
- `src/render/braille.rs` - Braille character canvas rendering via ratatui Canvas widget
- `src/render/color.rs` - Color scheme management (structure, chain, element, B-factor, rainbow, interface)

No external math or linear algebra crates are used. All vector math is hand-rolled with inline `V3` type aliases and helper functions.

---

*Stack analysis: 2026-03-06*
