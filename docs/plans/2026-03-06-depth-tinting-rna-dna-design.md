# Design: 24-bit Depth-Tinted Shading + RNA/DNA Support

**Date:** 2026-03-06
**Branch:** feature/depth-tinting-rna-dna

## Scope

Two features on a single branch:
1. Depth-tinted shading in HD mode (post-pass fog-color blending)
2. RNA/DNA structure support with cartoon rendering

## Feature 1: Depth-Tinted Shading (HD mode only)

### Current Behavior
Depth fog multiplies brightness by `1.0 - depth_fog * t`, fading toward black.

### New Behavior
Post-pass blending: after the framebuffer is fully rasterized, iterate all pixels and lerp each pixel's color toward a cool blue-gray fog color `[40, 50, 70]` based on its z-buffer depth.

```
for each pixel:
    t = (depth[i] - z_near) / z_range
    color[i] = lerp(color[i], fog_color, t * fog_strength)
```

### Why post-pass instead of per-primitive
- Uniform: applies identically to triangles, lines, and circles
- No API changes to drawing primitives (`draw_line_3d`, `draw_circle_z`, etc.)
- Removes existing per-triangle fog code from `rasterize_triangle_depth`, simplifying it
- Cost: one extra pass over framebuffer pixels — negligible vs rasterization

### Changes
- `src/render/framebuffer.rs`: Add `apply_depth_tint(&mut self, fog_color, fog_strength)` method. Simplify `rasterize_triangle_depth` by removing its inline fog logic.
- `src/render/hd.rs`: Call `fb.apply_depth_tint()` after all geometry is rasterized, using z_min/z_max computed from the framebuffer's depth buffer.

## Feature 2: RNA/DNA Support

### Model Changes (`src/model/protein.rs`)
- Add `MoleculeType` enum (`Protein`, `RNA`, `DNA`) to `Chain`
- Rename `is_ca` to `is_backbone` on `Atom` — set true for `CA` (protein) and `C4'` (nucleic acid)
- This fixes `bounding_radius()` and `backbone_atoms()` for nucleic-acid-only structures

### Parser Changes (`src/parser/pdb.rs`)
- Set `is_backbone` true for both `CA` and `C4'` atoms
- After building chains, classify `MoleculeType` from residue names:
  - Standard amino acids (ALA, GLY, ...) → `Protein`
  - A, U, G, C, I → `RNA`
  - DA, DT, DG, DC, DI → `DNA`
  - Mixed/unknown → `Protein` (safe default)

### Rendering: Braille Mode
- **Backbone**: Use `is_backbone` instead of `is_ca` (drop-in fix)
- **Wireframe**: Add `O3'→P` inter-residue bonds for nucleic acid chains alongside `C→N`
- **Cartoon**: For nucleic acid chains, render as backbone trace with thin lines from backbone to base centroid (simple, matches braille fidelity constraints)

### Rendering: HD Mode
- **Backbone** (`src/render/hd.rs`): Use `is_backbone` instead of `is_ca`
- **Wireframe** (`src/render/hd.rs`): Add `O3'→P` phosphodiester bonds
- **Cartoon** (`src/render/ribbon.rs`): For nucleic acid chains:
  - Backbone: Triangle-mesh tube through `C4'` atoms (reuse existing coil tube geometry with Catmull-Rom spline, Lambert shading, depth tinting)
  - Base slabs: 3D shaded rectangular meshes from `C1'` toward base-ring centroid. Each slab = ~8 triangles (4 faces × 2 tris). Emitted into the same `Vec<RibbonTriangle>` and rasterized through the standard pipeline.
  - All nucleotide residues use `SecondaryStructure::Coil` (tube cross-section)

### Color Scheme (`src/render/color.rs`)
- **Structure mode**: Detect nucleotide residues and color by base type:
  - A → Red `[220, 60, 60]`
  - U/T → Blue `[60, 60, 220]`
  - G → Green `[60, 180, 60]`
  - C → Yellow `[220, 200, 40]`
- **Element mode**: Already handles P, N, O, C — works as-is
- **Other modes** (Chain, Rainbow, BFactor, Interface): Work unchanged

### Rendering Quality by Mode

| Component | Braille | HD |
|---|---|---|
| NA backbone | Thick lines through C4' | Triangle-mesh tube (Catmull-Rom spline) |
| Base indicators | Thin lines to base centroid | 3D shaded rectangular slabs (triangle mesh) |
| Depth tinting | N/A | Post-pass fog blending |
| Lighting | N/A | Lambert + half-Lambert wrap |

## Files Changed

| File | Change |
|---|---|
| `src/model/protein.rs` | Add `MoleculeType` to `Chain`, rename `is_ca` → `is_backbone` |
| `src/parser/pdb.rs` | Classify chain type, mark nucleic acid backbone atoms |
| `src/render/framebuffer.rs` | Add `apply_depth_tint()`, simplify `rasterize_triangle_depth` |
| `src/render/hd.rs` | Call depth tint post-pass, use `is_backbone` |
| `src/render/ribbon.rs` | Nucleic acid chain: tube backbone + base slab mesh |
| `src/render/braille.rs` | Use `is_backbone`, add NA inter-residue bonds |
| `src/render/color.rs` | Nucleotide base-type coloring in Structure mode |
| `src/model/secondary.rs` | Skip SS assignment for nucleic acid chains |
| `src/model/interface.rs` | Use `is_backbone` if referenced |

## Testing Strategy
- Unit tests for `MoleculeType` classification from residue names
- Unit tests for `apply_depth_tint` (verify color blending math)
- Unit tests for nucleic acid backbone atom detection
- Integration test: load a protein+RNA PDB and verify chain types
- Existing tests must pass after `is_ca` → `is_backbone` rename
