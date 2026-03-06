# Testing Patterns

**Analysis Date:** 2026-03-06

## Test Framework

**Runner:**
- Built-in Rust test framework (`#[test]`, `cargo test`)
- No external test runner or harness
- Config: standard Cargo defaults (no custom test configuration in `Cargo.toml`)

**Assertion Library:**
- Standard library `assert!`, `assert_eq!`, `assert_ne!` macros
- Custom assertion messages used for diagnostic clarity: `assert!(helix_count > 0, "Expected helix residues, got 0")`

**Run Commands:**
```bash
cargo test                    # Run all tests
cargo test -- --nocapture     # Run with stdout visible
cargo test <test_name>        # Run specific test
cargo test -- --test-threads=1  # Single-threaded (if needed)
```

No watch mode, coverage tool, or CI pipeline configured.

## Test File Organization

**Location:**
- Tests are co-located with source code using `#[cfg(test)] mod tests` blocks at the bottom of each file
- No separate `tests/` directory for integration tests
- No `benches/` directory for benchmarks

**Naming:**
- Test functions prefixed with `test_`: `test_contact_detected`, `test_zbuffer`, `test_normalize`
- Descriptive names that indicate what is being verified: `test_quantize_color_near_black_stays_visible`

**Files with Tests:**
- `src/model/interface.rs` -- 8 tests (interface analysis logic)
- `src/model/secondary.rs` -- 5 tests (CIF/PDB secondary structure parsing)
- `src/render/framebuffer.rs` -- 13 tests (framebuffer, z-buffer, rasterization, quantization)

**Files without Tests:**
- `src/main.rs` -- TUI event loop (not unit-testable without mocking terminal)
- `src/app.rs` -- App state management
- `src/event.rs` -- Input thread spawning
- `src/parser/pdb.rs` -- Structure loading (tested indirectly via `secondary.rs`)
- `src/parser/fetch.rs` -- Network fetch (feature-gated)
- `src/render/camera.rs` -- 3D projection math
- `src/render/color.rs` -- Color scheme logic
- `src/render/braille.rs` -- Braille canvas rendering
- `src/render/hd.rs` -- HD rendering pipeline
- `src/render/ribbon.rs` -- Ribbon mesh generation
- `src/ui/*.rs` -- All UI rendering modules

## Test Structure

**Suite Organization:**
```rust
// At the bottom of the source file:
#[cfg(test)]
mod tests {
    use super::*;
    // Additional imports for test-only types
    use crate::model::protein::{Atom, Chain, Protein, Residue, SecondaryStructure};

    // Optional: test helper functions (not annotated with #[test])
    fn make_residue(name: &str, seq_num: i32, x: f64, y: f64, z: f64) -> Residue { ... }
    fn two_chain_protein() -> Protein { ... }

    #[test]
    fn test_specific_behavior() {
        // Arrange
        let protein = two_chain_protein();

        // Act
        let analysis = analyze_interface(&protein, 4.5);

        // Assert
        assert_eq!(analysis.contacts.len(), 1);
        assert_eq!(analysis.total_interface_residues, 2);
    }
}
```

**Patterns:**
- **Arrange-Act-Assert** structure (implicit, not labeled)
- No `setup` / `teardown` functions; each test constructs its own state
- Test helper functions are private, non-`#[test]` functions inside the `mod tests` block
- No `#[should_panic]` tests
- No `#[ignore]` tests
- No async tests

## Test Data & Fixtures

**Inline Fixtures:**
- Model structs are constructed inline in each test:
```rust
// From src/model/interface.rs
fn make_residue(name: &str, seq_num: i32, x: f64, y: f64, z: f64) -> Residue {
    Residue {
        name: name.to_string(),
        seq_num,
        atoms: vec![Atom {
            name: "CA".to_string(),
            element: "C".to_string(),
            x, y, z,
            b_factor: 0.0,
            is_ca: true,
        }],
        secondary_structure: SecondaryStructure::Coil,
    }
}

fn two_chain_protein() -> Protein {
    Protein {
        name: "test".to_string(),
        chains: vec![
            Chain {
                id: "A".to_string(),
                residues: vec![
                    make_residue("ALA", 1, 0.0, 0.0, 0.0),
                    make_residue("GLY", 2, 10.0, 0.0, 0.0),
                ],
            },
            Chain {
                id: "B".to_string(),
                residues: vec![
                    make_residue("ASP", 1, 3.0, 0.0, 0.0),
                    make_residue("LEU", 2, 20.0, 0.0, 0.0),
                ],
            },
        ],
    }
}
```

**File-Based Fixtures:**
- Real PDB/CIF files in `examples/` directory used as integration test fixtures:
  - `examples/1ZVH.cif` -- used by secondary structure parsing tests
  - `examples/1UBQ.pdb`, `examples/4HHB.pdb`, `examples/AF3_TNFa.pdb` -- available but not currently used in tests
- File paths resolved at compile time with `concat!(env!("CARGO_MANIFEST_DIR"), "/examples/1ZVH.cif")`

**Location:**
- No dedicated `fixtures/` or `testdata/` directory
- Example files in `examples/` serve double duty as both user examples and test fixtures

## Mocking

**Framework:** None

**Patterns:**
- No mocking framework used
- Tests operate on real implementations with constructed data
- Functions that would be difficult to test (terminal I/O, network, rendering to screen) are simply not tested
- Some methods are gated to test-only visibility with `#[cfg(test)]`:
  ```rust
  // src/render/framebuffer.rs
  #[cfg(test)]
  pub fn clear(&mut self) { ... }

  #[cfg(test)]
  pub fn rasterize_triangle(&mut self, tri: &Triangle, light_dir: [f64; 3]) { ... }

  #[cfg(test)]
  pub fn draw_circle(&mut self, cx: f64, cy: f64, radius: f64, color: [u8; 3]) { ... }

  #[cfg(test)]
  pub fn framebuffer_to_widget(fb: &Framebuffer) -> Paragraph<'static> { ... }
  ```
  These exist to expose simplified interfaces for testing without polluting the production API.

**What to Mock (if adding tests):**
- Terminal I/O via `crossterm` -- wrap behind a trait for testability
- File system access in parser modules -- accept `Read` trait instead of file paths
- Network requests in `parser/fetch.rs` -- already feature-gated, could add mock HTTP client

**What NOT to Mock:**
- Domain logic (model structs, color schemes, camera math) -- test directly
- Framebuffer operations -- test the actual rasterizer output

## Coverage

**Requirements:** None enforced

**Current State:**
- 26 tests total, all passing
- Tests concentrated in 3 files:
  - `src/model/interface.rs`: 8 tests covering contact detection, filtering, edge cases, summary output
  - `src/model/secondary.rs`: 5 tests covering CIF tokenization, SS record parsing, full load integration
  - `src/render/framebuffer.rs`: 13 tests covering framebuffer init, z-buffer, rasterization, line drawing, circle drawing, quantization, widget conversion
- No tests for: camera projection, color schemes, ribbon mesh generation, UI rendering, app state transitions, braille rendering, HD rendering pipeline

**View Coverage:**
```bash
# Requires cargo-tarpaulin or cargo-llvm-cov
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## Test Types

**Unit Tests:**
- Pure logic functions tested with constructed inputs and verified outputs
- Examples: z-buffer priority, triangle rasterization pixel coverage, vector normalization, CIF tokenization
- Pattern: call function, assert output properties

**Integration Tests (within unit test modules):**
- `test_full_cif_load_has_non_coil_residues` in `src/model/secondary.rs:493-523` -- loads a real CIF file through the full parser pipeline and verifies secondary structure assignment end-to-end
- `test_parse_cif_ss_records_from_example_file` in `src/model/secondary.rs:419-443` -- parses a real CIF file and verifies exact helix/sheet counts

**E2E Tests:**
- Not present. The application is a TUI; end-to-end testing would require terminal emulation.

## Common Patterns

**Numerical Precision Testing:**
```rust
// From src/render/framebuffer.rs
#[test]
fn test_normalize() {
    let v = normalize([3.0, 0.0, 4.0]);
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    assert!((len - 1.0).abs() < 1e-9);
    assert!((v[0] - 0.6).abs() < 1e-9);
}
```
Use epsilon comparison (`< 1e-9`) for floating-point assertions.

**Pixel/Buffer State Testing:**
```rust
// From src/render/framebuffer.rs
#[test]
fn test_zbuffer() {
    let mut fb = Framebuffer::new(4, 4);
    fb.set_pixel(1, 1, 5.0, [255, 0, 0]);
    assert_eq!(fb.color[1 * 4 + 1], [255, 0, 0]);

    // Closer fragment wins
    fb.set_pixel(1, 1, 3.0, [0, 255, 0]);
    assert_eq!(fb.color[1 * 4 + 1], [0, 255, 0]);

    // Farther fragment is rejected
    fb.set_pixel(1, 1, 4.0, [0, 0, 255]);
    assert_eq!(fb.color[1 * 4 + 1], [0, 255, 0]);
}
```
Access internal buffer state directly via index math for verification.

**Boundary / Edge Case Testing:**
```rust
// From src/model/interface.rs
#[test]
fn test_empty_protein() {
    let protein = Protein { name: "empty".to_string(), chains: vec![] };
    let analysis = analyze_interface(&protein, 4.5);
    assert!(analysis.contacts.is_empty());
    assert_eq!(analysis.total_interface_residues, 0);
    let lines = analysis.summary(&protein);
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("no inter-chain contacts"));
}

#[test]
fn test_no_contacts_below_cutoff() {
    let protein = two_chain_protein();
    let analysis = analyze_interface(&protein, 2.0);
    assert!(analysis.contacts.is_empty());
}
```
Test both empty inputs and parameterized edge cases (e.g., cutoff too small).

**Integration Test with File Fixtures:**
```rust
// From src/model/secondary.rs
#[test]
fn test_parse_cif_ss_records_from_example_file() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/1ZVH.cif");
    let ranges = parse_cif_ss_records(path);

    let helix_count = ranges.iter()
        .filter(|r| r.ss_type == SecondaryStructure::Helix).count();
    let sheet_count = ranges.iter()
        .filter(|r| r.ss_type == SecondaryStructure::Sheet).count();

    assert_eq!(helix_count, 9, "Expected 9 helix ranges, got {}", helix_count);
    assert_eq!(sheet_count, 17, "Expected 17 sheet ranges, got {}", sheet_count);
}
```
Use `concat!(env!("CARGO_MANIFEST_DIR"), ...)` to locate fixture files relative to the crate root.

## Adding New Tests

**Where to add:**
- Add tests in the same file as the code being tested, inside the `#[cfg(test)] mod tests` block
- If the file has no test module yet, add one at the bottom:
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn test_your_function() {
          // ...
      }
  }
  ```

**Conventions to follow:**
- Prefix all test functions with `test_`
- Use descriptive names: `test_<what>_<condition>` (e.g., `test_zbuffer`, `test_empty_protein`)
- Include custom assertion messages for non-obvious failures
- Reuse helper functions within the test module for building test data
- Keep tests focused -- one logical assertion per test, though multiple `assert_*!` calls are fine when verifying multiple properties of a single result

---

*Testing analysis: 2026-03-06*
