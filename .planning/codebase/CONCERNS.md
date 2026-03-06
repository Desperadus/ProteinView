# Codebase Concerns

**Analysis Date:** 2026-03-06

## Tech Debt

**Duplicated `atoms_bonded_3d` function:**
- Issue: The function `atoms_bonded_3d` is copy-pasted identically in two files with the same signature, body, and constant (1.9 A cutoff).
- Files: `src/render/braille.rs` (line 46), `src/render/hd.rs` (line 130)
- Impact: Bug fixes or cutoff changes must be applied in both places. Easy to diverge silently.
- Fix approach: Move to a shared utility module (e.g., `src/render/geometry.rs` or `src/model/protein.rs`) and import from both renderers.

**Duplicated `color_to_rgb` function:**
- Issue: The helper `color_to_rgb` that converts `ratatui::style::Color` to `[u8; 3]` is duplicated identically in two rendering modules.
- Files: `src/render/ribbon.rs` (line 105), `src/render/hd.rs` (line 74)
- Impact: Inconsistency risk if one copy is updated but not the other.
- Fix approach: Move to `src/render/color.rs` as a public helper and import.

**Hand-rolled vector math instead of a math crate:**
- Issue: `src/render/ribbon.rs` defines its own `V3` type alias and helper functions (`v3_add`, `v3_sub`, `v3_scale`, `v3_dot`, `v3_cross`, `v3_len`, `v3_normalize`). `src/render/framebuffer.rs` has its own separate `normalize` function. The camera module in `src/render/camera.rs` inlines rotation math directly.
- Files: `src/render/ribbon.rs` (lines 56-99), `src/render/framebuffer.rs` (lines 646-653), `src/render/camera.rs` (lines 65-87), `src/render/hd.rs` (lines 82-96)
- Impact: Code duplication across the rendering pipeline. Rotation logic is repeated between `camera.rs::project()` and `hd.rs::rotate_normal()`. No shared matrix type makes adding perspective projection or new transformations harder.
- Fix approach: Either introduce a lightweight math crate (e.g., `glam`) or consolidate all vector/matrix operations into a single `src/render/math.rs` module.

**`framebuffer_to_widget` is `#[cfg(test)]` only but exists in production code:**
- Issue: The half-block rendering function `framebuffer_to_widget` and several `Framebuffer` methods (`clear`, `rasterize_triangle`, `draw_circle`) are gated behind `#[cfg(test)]` and never used in production paths. The production rendering path uses `framebuffer_to_braille_widget` and `to_rgb_image` instead.
- Files: `src/render/framebuffer.rs` (lines 45-46, 74-77, 296-299, 406-499)
- Impact: ~100 lines of dead production code. The half-block widget converter is a complete alternative rendering path that is only exercised in one trivial test.
- Fix approach: Either promote these to production use (e.g., as a third fallback rendering mode) or move them to a `#[cfg(test)]` helper module.

**Braille canvas mode (non-HD) does not support Cartoon visualization:**
- Issue: In the non-HD braille rendering path (`src/render/braille.rs`), `VizMode::Cartoon` falls through to the same backbone rendering as `VizMode::Backbone`. The `match` arm groups them: `VizMode::Backbone | VizMode::Cartoon => { render_backbone(...) }`.
- Files: `src/render/braille.rs` (line 75)
- Impact: Users who toggle to non-HD mode while in Cartoon mode silently get backbone rendering instead of cartoon ribbons. No visual or textual indication that cartoon is unavailable in this mode.
- Fix approach: Either implement cartoon rendering for the braille Canvas path (lower priority) or display a mode indicator/warning when Cartoon is selected in non-HD mode.

**CIF file is read from disk twice during loading:**
- Issue: When loading a CIF file, `pdbtbx::open()` reads and parses the file for atomic data, and then `parse_cif_ss_records()` re-opens and reads the same file to extract secondary structure records from raw text. The PDB path has the same pattern via `parse_ss_records()`.
- Files: `src/parser/pdb.rs` (lines 9-16, 53-65), `src/model/secondary.rs` (lines 22-102, 182-299)
- Impact: Doubles I/O for structure loading. For typical PDB/CIF files (< 1 MB) this is negligible, but for large complexes or network-mounted filesystems this adds latency.
- Fix approach: Parse SS records from the already-loaded pdbtbx PDB object when possible (pdbtbx may expose HELIX/SHEET data), or cache the file content in memory for the second pass.

**`Protein::bounding_radius` only considers C-alpha atoms:**
- Issue: The bounding radius calculation filters to `is_ca` atoms only. In wireframe mode, all atoms are rendered, so side-chain atoms beyond the C-alpha bounding sphere may be clipped or appear at edges.
- Files: `src/model/protein.rs` (lines 78-85)
- Impact: Minor visual clipping in wireframe mode for proteins with extended side chains.
- Fix approach: Compute bounding radius over all atoms, or add a separate `bounding_radius_all_atoms()` method and use it when wireframe mode is active.

## Known Bugs

**Rainbow color scheme uses `seq_num` divided by `total_residues`:**
- Symptoms: Rainbow coloring can produce incorrect hue mapping when residue sequence numbers are non-contiguous (common in PDB files with insertion codes or missing residues) or when `seq_num` exceeds `total_residues`.
- Files: `src/render/color.rs` (lines 171-177)
- Trigger: Load any PDB with non-sequential residue numbering (e.g., numbering gaps from disordered regions).
- Workaround: Use a different color scheme.

**Interface `color_to_rgb` checks against `chain.id.clone()` on every residue call:**
- Symptoms: The `interface_color` method clones `chain.id` to check set membership on every single residue color lookup, creating a `String` allocation per residue per frame.
- Files: `src/render/color.rs` (line 110)
- Trigger: Interface mode with any protein; allocates on every render call.
- Workaround: None needed for correctness, but it is a per-frame allocation hotspot.

## Security Considerations

**Fetch feature downloads to predictable temp path without validation:**
- Risk: The `fetch_pdb` function writes to `$TMPDIR/{PDB_ID}.cif` without sanitizing the PDB ID input. A crafted PDB ID like `../../etc/foo` could theoretically write outside the temp directory (path traversal), though `std::env::temp_dir().join()` provides some protection.
- Files: `src/parser/fetch.rs` (lines 7-14)
- Current mitigation: `temp_dir().join()` normalizes paths on most platforms. The PDB ID is uppercased and used in a format string.
- Recommendations: Validate that PDB ID matches `^[A-Za-z0-9]{4}$` before constructing the URL and file path. Consider using `tempfile` crate for safe temp file creation.

**No TLS certificate validation control:**
- Risk: The `reqwest::blocking::get` call uses default TLS settings. In environments with custom CA bundles or corporate proxies, this may fail silently or connect to unexpected endpoints.
- Files: `src/parser/fetch.rs` (line 8)
- Current mitigation: Default `reqwest` TLS behavior is generally secure.
- Recommendations: Low priority. Consider allowing a `--no-verify-ssl` flag for corporate environments if users report issues.

**PDB parser silently ignores errors:**
- Risk: The `parse_ss_records` function silently returns an empty vec on file-open failure (line 26). The pdbtbx fallback path in `load_structure` silently degrades to loose parsing. Users get no warning that secondary structure data may be missing.
- Files: `src/model/secondary.rs` (lines 23-26), `src/parser/pdb.rs` (lines 9-16)
- Current mitigation: The structure still loads; only SS annotation is missing.
- Recommendations: Log a warning when SS parsing fails or returns zero ranges for a file that should contain them.

## Performance Bottlenecks

**O(n*m) brute-force interface analysis:**
- Problem: `analyze_interface` compares every residue in chain i against every residue in chain j, and for each pair compares all heavy atoms pairwise. For a protein with N residues and A atoms per residue across C chains, this is O(C^2 * N^2 * A^2).
- Files: `src/model/interface.rs` (lines 49-101)
- Cause: No spatial indexing. All-pairs comparison.
- Improvement path: Use a spatial index (k-d tree or grid) to prune distant residue pairs. The `pdbtbx` crate already has `rstar` feature enabled (see `Cargo.toml`) but it is not used for interface analysis. For the typical antibody-antigen complex (~500 residues) this is fast enough, but for large assemblies (>5000 residues) it will become a bottleneck.

**Full framebuffer allocation every frame:**
- Problem: `render_hd_framebuffer` allocates a new `Framebuffer` with two `Vec` allocations (`color` and `depth`) on every single frame. At HD resolution with Sixel/Kitty (e.g., 1920x1080 pixels), this is ~24 MB of allocations per frame at 30 FPS.
- Files: `src/render/hd.rs` (line 28), `src/render/framebuffer.rs` (lines 34-42)
- Cause: No framebuffer reuse between frames.
- Improvement path: Keep a persistent `Framebuffer` in `App` and add a `clear()` method (already exists but is `#[cfg(test)]` only). Reuse the allocation across frames, only reallocating when the terminal is resized.

**Cartoon ribbon mesh is fully re-projected every frame:**
- Problem: While the ribbon mesh triangles are cached in `App::mesh_cache`, every frame iterates through all triangles to project all three vertices through the camera. For a large protein this can be thousands of triangles with 3 projections each.
- Files: `src/render/hd.rs` (lines 34-62)
- Cause: No frustum culling or level-of-detail. Back-face culling is not performed (two-sided lighting is used instead).
- Improvement path: Add back-face culling to skip ~50% of triangles. Implement bounding-box pre-check per chain to skip off-screen chains entirely.

**Software triangle rasterizer is single-threaded:**
- Problem: The entire rasterization pipeline (triangle projection, rasterization, z-buffering) runs on the main thread. Large proteins in cartoon mode with many triangles cause frame drops.
- Files: `src/render/framebuffer.rs` (lines 87-186), `src/render/hd.rs` (lines 34-62)
- Cause: No parallelism in the rendering pipeline.
- Improvement path: Use `rayon` for parallel triangle rasterization across scanlines or triangle batches. The framebuffer z-buffer would need atomic operations or tile-based partitioning.

**Wireframe mode is O(n^2) per residue for bond detection:**
- Problem: Bond detection in wireframe mode checks all atom pairs within each residue using a nested loop: `for i in 0..projected.len() { for j in (i+1)..projected.len() { ... } }`. For residues with many atoms (e.g., large ligands or modified residues), this is O(A^2).
- Files: `src/render/hd.rs` (lines 178-186), `src/render/braille.rs` (lines 136-145)
- Cause: No precomputed bond list; distance-based heuristic at render time.
- Improvement path: Precompute bonds during parsing (using standard residue templates or the 1.9 A distance cutoff) and store them in the `Residue` struct. This moves the O(A^2) work to load time instead of every frame.

## Fragile Areas

**Secondary structure column parsing (PDB format):**
- Files: `src/model/secondary.rs` (lines 22-102)
- Why fragile: Hard-coded byte offsets (`line.as_bytes()[19]`, `&line[21..25]`, etc.) for PDB HELIX/SHEET records. Any deviation from standard PDB column formatting (common in software-generated PDB files, AlphaFold output, etc.) silently produces wrong results or skips records.
- Safe modification: Always test with the example PDB files in `examples/`. Add new example files that exercise edge cases. The CIF parser (lines 182-299) is more robust due to column-name-based parsing.
- Test coverage: No unit tests for `parse_ss_records` (PDB format). CIF parsing has good test coverage.

**Zoom calculation depends on multiple rendering mode branches:**
- Files: `src/app.rs` (lines 56-81, 183-203)
- Why fragile: The auto-zoom formula has separate branches for: (1) HD mode with true graphics protocol, (2) HD mode with braille fallback, (3) non-HD mode. The pixel dimension calculations must exactly match the corresponding logic in `src/ui/viewport.rs` (lines 18-19, 45-55). If any branch diverges, the protein will be incorrectly sized.
- Safe modification: Extract the pixel-dimension calculation into a shared function called by both `App::new`/`App::recalculate_zoom` and `render_viewport`/`render_hd_viewport`.
- Test coverage: No tests for zoom calculation.

**Main loop mixes input handling, rendering, and frame timing:**
- Files: `src/main.rs` (lines 131-255)
- Why fragile: The main loop is a single 125-line function handling input dispatch, frame skipping logic, terminal drawing, and timing. Adding new keybindings or rendering modes requires modifying this monolithic loop.
- Safe modification: Extract input handling into an `App::handle_key` method. Extract the frame-skip logic into a helper.
- Test coverage: None. The main loop is untestable in its current form.

## Scaling Limits

**Large proteins (>10,000 residues):**
- Current capacity: Works well for typical proteins up to ~2000 residues. Example files range from 76 to 574 residues.
- Limit: Cartoon mode generates ~14 spline subdivisions per residue pair, each producing multiple triangles. A 10,000-residue protein would produce ~280,000+ triangles, causing multi-second frame times with the software rasterizer.
- Scaling path: Add level-of-detail (reduce spline subdivisions for distant chains), implement frustum culling, or switch to GPU rendering (wgpu) for very large structures.

**Multi-model PDB files (NMR ensembles):**
- Current capacity: Only the first model is loaded (pdbtbx default behavior).
- Limit: Cannot view alternate conformations or NMR ensemble members.
- Scaling path: Add `--model N` CLI flag and model cycling keybinding.

## Dependencies at Risk

**`ratatui-image` version 9.0.0:**
- Risk: Major version (9.x) of a fast-moving crate. The `Picker::from_query_stdio()` API, `ProtocolType` enum variants, and `Resize` enum may change in future releases.
- Impact: Build breakage on dependency update. The graphics protocol detection is tightly coupled to this crate's API.
- Migration plan: Pin to exact version in `Cargo.toml` (currently using `"9.0.0"`). Monitor changelog before upgrading.

**Rust edition 2024 requirement:**
- Risk: `edition = "2024"` in `Cargo.toml` requires Rust 1.85+, which is very recent. Many users and CI systems may not have this version yet.
- Impact: Users on stable Rust < 1.85 cannot compile the project.
- Migration plan: Consider whether edition 2024 features are actually needed. If not, downgrading to edition 2021 (Rust 1.56+) would significantly widen compatibility.

## Missing Critical Features

**No terminal resize handling:**
- Problem: Terminal dimensions are captured once at startup (`crossterm::terminal::size()`). If the user resizes their terminal window, the viewport dimensions, zoom factor, and framebuffer size remain stale.
- Blocks: Usability in tiling window managers, terminal multiplexers (tmux/screen), and any scenario where the terminal is resized during use.

**No error reporting to the user:**
- Problem: Errors during parsing (missing SS records, malformed PDB lines) are silently swallowed. The user sees a structure rendered without secondary structure coloring and has no indication why.
- Blocks: Debugging loading issues. Users may think the tool is broken rather than understanding their input file has issues.

**No mouse support:**
- Problem: Rotation, zoom, and pan require keyboard shortcuts. No mouse drag for rotation or scroll for zoom.
- Blocks: Intuitive interaction for users accustomed to molecular viewers (PyMOL, ChimeraX) where mouse interaction is primary.

## Test Coverage Gaps

**No tests for rendering pipeline:**
- What's not tested: `src/render/hd.rs` (206 lines), `src/render/braille.rs` (165 lines), `src/render/camera.rs` (88 lines), `src/render/ribbon.rs` (578 lines -- only tested indirectly through the CIF integration test)
- Files: All files in `src/render/` except `framebuffer.rs`
- Risk: Rendering regressions (incorrect projection, wrong colors, clipping errors) go undetected.
- Priority: Medium. Camera projection and ribbon generation are the most important to test.

**No tests for the App state machine:**
- What's not tested: `src/app.rs` (204 lines) -- mode cycling, chain navigation, interface toggling, zoom recalculation
- Files: `src/app.rs`
- Risk: State transitions (e.g., toggling interface mode while on last chain) could panic or produce incorrect state.
- Priority: Medium. These are pure state transitions that are easy to unit test.

**No tests for PDB format secondary structure parsing:**
- What's not tested: `parse_ss_records()` function that parses HELIX/SHEET records from PDB files using hard-coded column offsets.
- Files: `src/model/secondary.rs` (lines 22-102)
- Risk: The most fragile parser in the codebase has zero test coverage. PDB column-offset parsing is known to be error-prone.
- Priority: High. Add tests using the `1UBQ.pdb` and `4HHB.pdb` example files.

**No tests for UI components:**
- What's not tested: All files in `src/ui/` (307 lines total across 6 files)
- Files: `src/ui/viewport.rs`, `src/ui/interface_panel.rs`, `src/ui/help_overlay.rs`, `src/ui/statusbar.rs`, `src/ui/helpbar.rs`, `src/ui/header.rs`
- Risk: Low. These are thin rendering wrappers around ratatui widgets with no complex logic.
- Priority: Low.

---

*Concerns audit: 2026-03-06*
