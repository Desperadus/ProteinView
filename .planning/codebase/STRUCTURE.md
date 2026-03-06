# Codebase Structure

**Analysis Date:** 2026-03-06

## Directory Layout

```
proteinview/
├── src/                    # All Rust source code
│   ├── main.rs             # Binary entry point, CLI, event loop
│   ├── app.rs              # Application state container
│   ├── event.rs            # Input thread spawning
│   ├── model/              # Domain data types and analysis
│   │   ├── mod.rs          # Module re-exports
│   │   ├── protein.rs      # Protein/Chain/Residue/Atom structs
│   │   ├── secondary.rs    # Secondary structure parsing (PDB + CIF)
│   │   └── interface.rs    # Inter-chain contact analysis
│   ├── parser/             # File format loading
│   │   ├── mod.rs          # Module re-exports
│   │   ├── pdb.rs          # PDB/mmCIF loading via pdbtbx
│   │   └── fetch.rs        # RCSB PDB download (optional feature)
│   ├── render/             # 3D rendering pipeline
│   │   ├── mod.rs          # Module re-exports
│   │   ├── camera.rs       # 3D camera, projection, rotation
│   │   ├── color.rs        # Color schemes (6 modes)
│   │   ├── braille.rs      # Braille-character rendering (Canvas)
│   │   ├── hd.rs           # HD framebuffer rendering orchestrator
│   │   ├── framebuffer.rs  # Software rasterizer (z-buffer, triangles, lines)
│   │   └── ribbon.rs       # Cartoon ribbon mesh generation
│   └── ui/                 # Terminal UI widgets
│       ├── mod.rs          # Module re-exports
│       ├── viewport.rs     # Main 3D viewport (delegates to render/)
│       ├── header.rs       # Top title bar
│       ├── statusbar.rs    # Mode/chain/color info bar
│       ├── helpbar.rs      # Bottom keybinding hints
│       ├── help_overlay.rs # Centered help popup
│       └── interface_panel.rs  # Interface analysis sidebar
├── examples/               # Sample structure files for testing
│   ├── 1UBQ.pdb           # Ubiquitin (single chain, small)
│   ├── 4HHB.pdb           # Hemoglobin (4 chains)
│   ├── 1ZVH.cif           # Antibody-antigen complex (mmCIF)
│   └── AF3_TNFa.pdb       # AlphaFold3 output (non-standard metadata)
├── assets/                 # Screenshots, demo GIF, ASCII header
│   ├── demo.gif
│   ├── demo.mov
│   ├── header.txt
│   └── *.png              # Various screenshot examples
├── docs/                   # Design documents
│   └── plans/
│       └── 2026-03-04-proteinview-design.md
├── Cargo.toml              # Package manifest
├── Cargo.lock              # Dependency lockfile
├── README.md               # Project documentation
├── LICENSE                 # MIT license
└── .gitignore              # Git ignore rules
```

## Directory Purposes

**`src/`:**
- Purpose: All Rust source code for the binary.
- Contains: 4 submodules (`model/`, `parser/`, `render/`, `ui/`) plus 3 root files (`main.rs`, `app.rs`, `event.rs`).
- Key files: `src/main.rs` (entry point), `src/app.rs` (central state).

**`src/model/`:**
- Purpose: Pure domain types and computational algorithms. No I/O, no rendering, no TUI dependencies.
- Contains: Data structures (`Protein`, `Chain`, `Residue`, `Atom`), secondary structure parsing from raw file records, inter-chain interface analysis.
- Key files: `src/model/protein.rs` (core data types), `src/model/interface.rs` (contact computation with tests).

**`src/parser/`:**
- Purpose: Load external file formats into domain model.
- Contains: PDB/mmCIF loading, optional network fetching.
- Key files: `src/parser/pdb.rs` (main loader function `load_structure()`).

**`src/render/`:**
- Purpose: 3D rendering pipeline from domain model to displayable widgets.
- Contains: Camera/projection math, color mapping, two rendering backends (braille canvas, HD framebuffer), ribbon mesh geometry generation.
- Key files: `src/render/framebuffer.rs` (804 lines, largest file -- software rasterizer), `src/render/ribbon.rs` (578 lines, Catmull-Rom spline + mesh generation).

**`src/ui/`:**
- Purpose: Terminal widget rendering using ratatui. Each file is one widget.
- Contains: Viewport (delegates to render/), header, statusbar, helpbar, help overlay, interface panel.
- Key files: `src/ui/viewport.rs` (chooses braille vs HD rendering path).

**`examples/`:**
- Purpose: Sample protein structure files for development and testing.
- Contains: 3 PDB files, 1 mmCIF file. Used by integration tests in `src/model/secondary.rs`.
- Generated: No (committed sample data).

**`assets/`:**
- Purpose: README images and demo media.
- Contains: PNG screenshots, GIF demo, MOV video, ASCII header art.
- Generated: Manually captured.

**`docs/`:**
- Purpose: Design and planning documents.
- Contains: Initial design research document.

## Key File Locations

**Entry Points:**
- `src/main.rs`: Binary entry point. CLI parsing (`Cli` struct lines 34-59), terminal setup, main event loop, frame rendering.

**Configuration:**
- `Cargo.toml`: Package metadata, dependencies, features (`fetch`), release profile.
- No runtime config files. All configuration via CLI arguments.

**Core Logic:**
- `src/app.rs`: Central `App` state struct. Owns `Protein`, `Camera`, `ColorScheme`, `VizMode`, `InterfaceAnalysis`, ribbon mesh cache.
- `src/model/protein.rs`: `Protein`, `Chain`, `Residue`, `Atom`, `SecondaryStructure` types.
- `src/model/interface.rs`: `analyze_interface()` function, `InterfaceAnalysis` struct, `Contact` struct.
- `src/model/secondary.rs`: `assign_from_pdb_file()`, `assign_from_cif_file()` functions, CIF state-machine parser.
- `src/render/camera.rs`: `Camera` struct with `project()` method (3D rotation + orthographic projection).
- `src/render/color.rs`: `ColorScheme` struct with `residue_color()` and `atom_color()` methods.
- `src/render/ribbon.rs`: `generate_ribbon_mesh()` function, Catmull-Rom spline, cross-section extrusion.
- `src/render/framebuffer.rs`: `Framebuffer` struct with `rasterize_triangle_depth()`, `draw_line_3d()`, `draw_thick_line_3d()`, `draw_circle_z()`, `framebuffer_to_braille_widget()`, `to_rgb_image()`.

**Testing:**
- `src/model/secondary.rs`: 4 unit/integration tests for CIF parsing and SS assignment.
- `src/model/interface.rs`: 8 unit tests for contact detection and analysis.
- `src/render/framebuffer.rs`: 10 unit tests for framebuffer, z-buffer, rasterization, quantization.
- Test data: `examples/1ZVH.cif` used by secondary structure tests via `concat!(env!("CARGO_MANIFEST_DIR"), "/examples/1ZVH.cif")`.

## Naming Conventions

**Files:**
- `snake_case.rs`: All Rust source files use lowercase snake_case (e.g., `interface_panel.rs`, `help_overlay.rs`).
- Module files: `mod.rs` in each directory for re-exports.

**Directories:**
- `snake_case/`: All lowercase (e.g., `model/`, `parser/`, `render/`, `ui/`).

**Types:**
- `PascalCase` for structs and enums: `Protein`, `Camera`, `Framebuffer`, `VizMode`, `ColorSchemeType`.
- `PascalCase` for enum variants: `SecondaryStructure::Helix`, `VizMode::Cartoon`.

**Functions:**
- `snake_case` for all functions: `load_structure()`, `render_viewport()`, `analyze_interface()`, `generate_ribbon_mesh()`.
- Prefixes: `render_*` for UI widget functions, `draw_*` for framebuffer primitives, `v3_*` for vector math helpers.

**Constants:**
- `SCREAMING_SNAKE_CASE`: `SPLINE_SUBDIVISIONS`, `COIL_SEGMENTS`, `HELIX_HALF_WIDTH`, `SIDEBAR_WIDTH`.

## Where to Add New Code

**New Visualization Mode:**
- Add variant to `VizMode` enum in `src/app.rs`.
- Update `VizMode::next()` and `VizMode::name()` in `src/app.rs`.
- Add rendering logic in `src/render/braille.rs` (match arm in `render_protein()`).
- Add HD rendering logic in `src/render/hd.rs` (match arm in `render_hd_framebuffer()`).
- Status bar auto-updates from `app.viz_mode.name()`.

**New Color Scheme:**
- Add variant to `ColorSchemeType` enum in `src/render/color.rs`.
- Update `ColorSchemeType::next()` and `name()`.
- Add color mapping method to `ColorScheme` impl.
- Wire into `residue_color()` and/or `atom_color()` match arms.

**New UI Widget:**
- Create `src/ui/new_widget.rs`.
- Add `pub mod new_widget;` to `src/ui/mod.rs`.
- Add layout constraint and render call in `terminal.draw()` closure in `src/main.rs` (lines 194-237).

**New Keybinding:**
- Add match arm in the key dispatch block in `src/main.rs` (lines 137-166).
- If it toggles state, add field to `App` struct in `src/app.rs`.
- Update help overlay text in `src/ui/help_overlay.rs`.
- Update helpbar hints in `src/ui/helpbar.rs`.

**New Parser/File Format:**
- Add module in `src/parser/` (e.g., `src/parser/mol2.rs`).
- Add `pub mod mol2;` to `src/parser/mod.rs`.
- Wire into file loading logic in `src/main.rs` (file extension dispatch).

**New Model Analysis:**
- Add module in `src/model/` (e.g., `src/model/surface_area.rs`).
- Add `pub mod surface_area;` to `src/model/mod.rs`.
- Compute in `App::new()` or lazily on demand.

**Tests:**
- Place `#[cfg(test)] mod tests` at the bottom of the relevant source file. Tests are co-located, not in a separate directory.
- Use `examples/` directory files for integration test data. Reference via `concat!(env!("CARGO_MANIFEST_DIR"), "/examples/...")`.

## Special Directories

**`target/`:**
- Purpose: Cargo build artifacts.
- Generated: Yes (by `cargo build`).
- Committed: No (in `.gitignore`).

**`examples/`:**
- Purpose: Sample PDB/CIF files for development and integration tests.
- Generated: No (manually downloaded from RCSB PDB and AlphaFold).
- Committed: Partially (excluded from crate via `Cargo.toml` `exclude` field, but tracked in git).

**`.planning/`:**
- Purpose: GSD planning and analysis documents.
- Generated: Yes (by analysis tooling).
- Committed: Not specified.

---

*Structure analysis: 2026-03-06*
