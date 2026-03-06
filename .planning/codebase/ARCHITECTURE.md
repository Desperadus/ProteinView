# Architecture

**Analysis Date:** 2026-03-06

## Pattern Overview

**Overall:** Single-binary TUI application following a layered Model-View-Renderer architecture with a synchronous main loop and dedicated input thread.

**Key Characteristics:**
- **Layered separation:** Data model (`model/`), file parsing (`parser/`), 3D rendering pipeline (`render/`), and terminal UI (`ui/`) are distinct modules with one-way dependency flow.
- **Immediate-mode rendering:** Every frame re-renders the 3D scene from scratch; the only cache is the ribbon triangle mesh (`App.mesh_cache`).
- **Dual rendering paths:** Braille-character rendering (default) and HD pixel rendering (via software rasterizer + optional Sixel/Kitty graphics protocol).
- **No async runtime:** Everything is synchronous except the input thread, which uses `std::sync::mpsc` channels.

## Layers

**Model Layer:**
- Purpose: Domain types representing protein structures and computed analysis.
- Location: `src/model/`
- Contains: `Protein`, `Chain`, `Residue`, `Atom`, `SecondaryStructure` data types; `InterfaceAnalysis` computation; secondary structure assignment logic.
- Depends on: Nothing (pure data + algorithms).
- Used by: `parser/`, `render/`, `app.rs`, `ui/`.

**Parser Layer:**
- Purpose: Load protein structure files (PDB, mmCIF) into the domain model.
- Location: `src/parser/`
- Contains: `pdb.rs` (structure loading via `pdbtbx` crate), `fetch.rs` (optional RCSB download).
- Depends on: `model/` (produces `Protein`), `pdbtbx` crate, optionally `reqwest`.
- Used by: `main.rs` (called once at startup).

**Render Layer:**
- Purpose: 3D projection, software rasterization, and scene-to-widget conversion.
- Location: `src/render/`
- Contains: Camera/projection (`camera.rs`), color schemes (`color.rs`), braille rendering (`braille.rs`), HD framebuffer rasterizer (`hd.rs`, `framebuffer.rs`), ribbon mesh generation (`ribbon.rs`).
- Depends on: `model/`, `app::VizMode`, `ratatui` (for widget types and `Color`).
- Used by: `ui/viewport.rs` (called every frame).

**UI Layer:**
- Purpose: Layout and render terminal widgets using ratatui.
- Location: `src/ui/`
- Contains: `viewport.rs` (3D viewport widget), `header.rs`, `statusbar.rs`, `helpbar.rs`, `help_overlay.rs`, `interface_panel.rs`.
- Depends on: `app::App` (reads state), `render/` (for viewport rendering).
- Used by: `main.rs` (called in `terminal.draw()` closure).

**Application State:**
- Purpose: Central state container holding protein data, camera, color scheme, viz mode, and UI toggles.
- Location: `src/app.rs`
- Contains: `App` struct, `VizMode` enum, state mutation methods (`cycle_color`, `toggle_interface`, `ribbon_mesh`, etc.).
- Depends on: `model/`, `render/camera`, `render/color`, `render/ribbon`.
- Used by: `main.rs` (owns the instance), `ui/` (reads via `&App` references).

**Event Handling:**
- Purpose: Decouple terminal input from rendering to keep quit responsive during slow HD frames.
- Location: `src/event.rs`
- Contains: `spawn_input_thread()` function returning `(mpsc::Receiver<KeyEvent>, Arc<AtomicBool>)`.
- Depends on: `crossterm::event`.
- Used by: `main.rs`.

## Data Flow

**Startup (Load & Initialize):**

1. `main.rs` parses CLI args via `clap` (`Cli` struct).
2. If `--fetch` is provided, `parser::fetch::fetch_pdb()` downloads from RCSB to a temp file.
3. `parser::pdb::load_structure()` opens the file with `pdbtbx`, converts to `Protein`, then assigns secondary structure from PDB HELIX/SHEET records (`model::secondary::assign_from_pdb_file`), falling back to CIF parsing (`assign_from_cif_file`) for mmCIF files.
4. `App::new()` centers the protein, computes `bounding_radius()`, calculates auto-zoom based on terminal size and graphics protocol, pre-computes `InterfaceAnalysis`, and generates initial `ribbon_mesh`.

**Main Loop (30 FPS):**

1. Drain all queued `KeyEvent`s from the input channel (`input_rx.try_recv()`).
2. Dispatch key codes to `App` state mutations (rotate, zoom, pan, cycle color/mode, toggle interface/help).
3. If `mesh_dirty`, regenerate ribbon mesh via `app.ribbon_mesh()`.
4. Adaptive frame skipping: if previous draw exceeded 2x tick rate, skip 1-3 frames.
5. `terminal.draw()` closure executes the UI layer:
   - If interface panel active: split layout horizontally (sidebar | main).
   - Main area split vertically: header (1 row) | viewport (flex) | statusbar (2 rows) | helpbar (1 row).
   - `ui::viewport::render_viewport()` chooses between braille or HD path.
6. `app.tick()` advances auto-rotation if enabled.
7. Sleep for 33ms (tick rate).

**Braille Rendering Path (default mode):**

1. `render::braille::render_protein()` creates a ratatui `Canvas` widget with `Marker::Braille`.
2. For Backbone/Cartoon modes: iterates C-alpha atoms, projects via `Camera::project()`, draws thick lines between consecutive atoms in the same chain.
3. For Wireframe mode: iterates all atoms, checks bond distances (< 1.9A), draws thin lines.
4. The `Canvas` widget is rendered directly by ratatui.

**HD Rendering Path (`--hd` or `m` toggle):**

1. `render::hd::render_hd_framebuffer()` creates a `Framebuffer` at pixel resolution.
2. For Cartoon mode: projects pre-computed `RibbonTriangle` mesh vertices through camera, rasterizes triangles with Lambert shading + depth fog via `Framebuffer::rasterize_triangle_depth()`.
3. For Backbone: draws thick 3D lines between C-alpha atoms with circle endpoints.
4. For Wireframe: draws all bonds as thick 3D lines with atom circles.
5. If terminal supports Sixel/Kitty/iTerm2: converts `Framebuffer` to `image::RgbImage` via `to_rgb_image()`, renders through `ratatui-image`.
6. Fallback: converts `Framebuffer` to braille characters via `framebuffer_to_braille_widget()` (colored braille with 2x4 dot-per-cell resolution).

**State Management:**
- All mutable state lives in `App` struct, owned by `main()`.
- Camera state (`Camera`) updates via direct method calls from key dispatch.
- Ribbon mesh uses a dirty flag (`mesh_dirty`) to avoid regeneration unless color scheme changes.
- Interface analysis is computed once at startup (immutable after init).

## Key Abstractions

**Protein / Chain / Residue / Atom:**
- Purpose: Hierarchical domain model matching PDB file structure.
- Examples: `src/model/protein.rs`
- Pattern: Nested `Vec` ownership (Protein owns Chains, Chains own Residues, Residues own Atoms). No Rc/Arc; cloning is cheap since data is loaded once.

**Camera:**
- Purpose: 3D-to-2D orthographic projection with rotation (Euler angles), zoom, and pan.
- Examples: `src/render/camera.rs`
- Pattern: `Camera::project(x, y, z) -> Projected` applies X/Y/Z rotation matrices then scales and translates. Returns `Projected { x, y, z }` where z is depth for z-buffering.

**Framebuffer:**
- Purpose: Software pixel buffer with z-buffer for HD rendering.
- Examples: `src/render/framebuffer.rs`
- Pattern: Row-major `Vec<[u8; 3]>` color array + `Vec<f64>` depth array. Provides `rasterize_triangle_depth()`, `draw_line_3d()`, `draw_thick_line_3d()`, `draw_circle_z()`. Output converted to ratatui widgets via `framebuffer_to_braille_widget()` or to `RgbImage` via `to_rgb_image()`.

**RibbonTriangle:**
- Purpose: Pre-computed triangle mesh for cartoon ribbon visualization.
- Examples: `src/render/ribbon.rs`
- Pattern: `generate_ribbon_mesh()` takes `Protein` + `ColorScheme`, produces `Vec<RibbonTriangle>`. Uses Catmull-Rom spline interpolation through C-alpha backbone, Frenet-Serret frame propagation, and secondary-structure-dependent cross-sections (flat ribbon for helices/sheets, circular tube for coils).

**ColorScheme:**
- Purpose: Map residues/atoms to RGB colors based on selected scheme.
- Examples: `src/render/color.rs`
- Pattern: `ColorScheme::residue_color(&Residue, &Chain) -> Color` and `atom_color(&Atom, &Residue, &Chain) -> Color`. Six schemes: Structure (by SS type), Chain, Element (CPK), BFactor (blue-red gradient), Rainbow (by sequence position), Interface (green/orange by contact analysis).

**InterfaceAnalysis:**
- Purpose: Pre-computed inter-chain contact map for protein interface visualization.
- Examples: `src/model/interface.rs`
- Pattern: `analyze_interface(&Protein, cutoff) -> InterfaceAnalysis`. Compares all heavy-atom pairs between chain pairs; stores contacts sorted by distance, interface residue set, and per-chain counts. Used by interface color scheme and sidebar panel.

## Entry Points

**Binary Entry:**
- Location: `src/main.rs`
- Triggers: `cargo run -- <file.pdb>` or `proteinview <file.pdb>`
- Responsibilities: CLI parsing, file loading, terminal setup/teardown, main event loop, frame rendering orchestration.

**CLI Arguments:**
- Location: `src/main.rs` (`Cli` struct, lines 34-59)
- Key args: `file` (positional), `--hd` (pixel rendering), `--color` (scheme), `--mode` (viz mode), `--fetch` (RCSB download), `--log` (debug log file).

**Structure Loading:**
- Location: `src/parser/pdb.rs` (`load_structure()`)
- Triggers: Called once at startup from `main()`.
- Responsibilities: Opens PDB/mmCIF via pdbtbx (with loose fallback for AlphaFold3 files), converts to domain model, assigns secondary structure.

## Error Handling

**Strategy:** `anyhow::Result` for fallible operations; graceful fallbacks for non-critical failures.

**Patterns:**
- Parser uses `anyhow::Result` with `.or_else()` fallback from strict to loose parsing mode (`src/parser/pdb.rs`, lines 9-16).
- Terminal graphics protocol detection falls back to halfblocks on error (`src/main.rs`, line 111).
- HD viewport falls back from Sixel/Kitty to colored braille on protocol error (`src/ui/viewport.rs`, lines 73-83).
- Secondary structure parsing silently returns empty on file errors (`src/model/secondary.rs`, lines 23-26).
- Panic hook restores terminal state before aborting (`src/main.rs`, lines 93-98).

## Cross-Cutting Concerns

**Logging:** Optional file-based debug logging via `--log` flag. Uses a custom `log!` macro in `src/main.rs` (lines 23-31) that writes to an `Option<File>`. Logs terminal size, picker protocol, frame numbers, key events, and render timing.

**Validation:** Minimal explicit validation. Parser relies on `pdbtbx` for file format validation. Bond distance check uses a hardcoded 1.9A threshold. Interface analysis uses a 4.5A cutoff. No user input validation beyond clap's argument parsing.

**Authentication:** Not applicable (local-only CLI tool). The optional `--fetch` feature makes unauthenticated HTTPS requests to RCSB PDB.

---

*Architecture analysis: 2026-03-06*
