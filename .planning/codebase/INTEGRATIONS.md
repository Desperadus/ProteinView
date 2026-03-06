# External Integrations

**Analysis Date:** 2026-03-06

## APIs & External Services

**RCSB Protein Data Bank (optional, behind `fetch` feature):**
- Purpose: Download protein structure files by PDB ID
- SDK/Client: `reqwest` 0.12.28 (blocking HTTP client)
- Auth: None required (public API)
- Endpoint: `https://files.rcsb.org/download/{PDB_ID}.cif`
- Implementation: `src/parser/fetch.rs`
- Behavior: Downloads mmCIF file to system temp directory, returns path. Feature-gated via `#[cfg(feature = "fetch")]`.
- Error handling: HTTP status check, returns `anyhow::Result` with descriptive error on failure.

```rust
// src/parser/fetch.rs - the entire integration
let url = format!("https://files.rcsb.org/download/{}.cif", pdb_id.to_uppercase());
let response = reqwest::blocking::get(&url)?;
let tmp_path = std::env::temp_dir().join(format!("{}.cif", pdb_id.to_uppercase()));
std::fs::write(&tmp_path, response.bytes()?)?;
```

**Terminal Graphics Protocols:**
- Purpose: Render pixel-perfect images directly in the terminal
- SDK/Client: `ratatui-image` 9.0.0
- Protocols supported: Sixel, Kitty graphics protocol, iTerm2 inline images
- Detection: Auto-detected at startup via `Picker::from_query_stdio()` in `src/main.rs`
- Fallback: Half-block characters (always available), then colored braille characters
- Implementation: `src/ui/viewport.rs` (rendering), `src/main.rs` (protocol detection), `src/app.rs` (picker storage)

## Data Storage

**Databases:**
- None. This is a standalone TUI application with no database.

**File Storage:**
- Local filesystem only
- Reads PDB (`.pdb`) and mmCIF (`.cif`, `.mmcif`) files from paths provided as CLI arguments
- When using `--fetch`, writes downloaded files to system temp directory (`std::env::temp_dir()`)
- No persistent state, no config files, no cache directories

**Caching:**
- In-memory only:
  - Ribbon mesh cache in `App.mesh_cache` (`src/app.rs`) - regenerated when color scheme changes
  - Interface analysis computed once at startup in `App.interface_analysis` (`src/app.rs`)
- No disk-based caching

## Authentication & Identity

**Auth Provider:**
- Not applicable. No user accounts, no authentication, no authorization.

## Monitoring & Observability

**Error Tracking:**
- None. Errors are reported via `anyhow::Result` and printed to stderr.
- Custom panic hook in `src/main.rs` restores terminal state before printing panic info.

**Logs:**
- Optional debug logging to file via `--log <path>` CLI flag
- Implementation: Custom `log!` macro in `src/main.rs` that writes to an `Option<std::fs::File>`
- Logs terminal size, picker protocol/font, app state, key events, frame rendering info
- No structured logging framework. No log levels. File-or-nothing.

```rust
// src/main.rs - logging macro
macro_rules! log {
    ($file:expr, $($arg:tt)*) => {
        if let Some(f) = $file.as_mut() {
            use std::io::Write;
            let _ = writeln!(f, $($arg)*);
            let _ = f.flush();
        }
    };
}
```

## CI/CD & Deployment

**Hosting:**
- Published to crates.io as `proteinview`
- Source on GitHub: `https://github.com/tristanfarmer/proteinview`

**CI Pipeline:**
- No CI configuration detected (no `.github/workflows/`, no `.gitlab-ci.yml`, no `Makefile`, no `Dockerfile`)

**Distribution:**
```bash
cargo install --path .                    # Local install
cargo install --path . --features fetch   # With fetch support
cargo build --release                     # Build release binary
```

## Environment Configuration

**Required env vars:**
- None. The application takes all configuration via CLI arguments.

**Secrets location:**
- No secrets. The only external service (RCSB PDB) is a public API requiring no authentication.

**CLI arguments (the only configuration surface):**
- `file` - Path to PDB or mmCIF file (positional, optional if `--fetch` used)
- `--hd` / `--pixel` - Enable HD pixel rendering
- `--color <scheme>` - Color scheme: structure, chain, element, bfactor, rainbow (default: structure)
- `--mode <mode>` - Visualization mode: backbone, wireframe (default: backbone, but app defaults to Cartoon)
- `--fetch <PDB_ID>` - Fetch structure from RCSB PDB (requires `fetch` feature)
- `--log <path>` - Write debug log to file

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None

## File Format Integrations

**PDB Format (`*.pdb`):**
- Parser: `pdbtbx` crate with fallback to loose strictness for non-standard files (e.g., AlphaFold3 output)
- Secondary structure: Parsed from HELIX/SHEET records via custom parser in `src/model/secondary.rs`
- Implementation: `src/parser/pdb.rs`

**mmCIF Format (`*.cif`, `*.mmcif`):**
- Parser: Same `pdbtbx` crate
- Secondary structure: Parsed from `_struct_conf` and `_struct_sheet_range` CIF categories via custom state-machine parser in `src/model/secondary.rs`
- Uses `auth_asym_id` / `auth_seq_id` fields to match pdbtbx's chain/residue identifiers

**Terminal Protocol Integration:**
- Sixel: Bitmap image encoding for terminals supporting DEC Sixel graphics
- Kitty: Kitty terminal's native graphics protocol (base64-encoded PNG)
- iTerm2: Inline image protocol
- Halfblocks: Unicode half-block characters (fallback, always available)
- Detection and encoding handled entirely by `ratatui-image` crate

---

*Integration audit: 2026-03-06*
