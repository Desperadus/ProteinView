use ratatui::symbols::Marker;
use ratatui::widgets::canvas::{Canvas, Context, Line};

use crate::model::protein::{Protein, SecondaryStructure};
use crate::render::camera::Camera;
use crate::render::color::ColorScheme;

/// Draw a thick line by rendering `num_lines` parallel lines spread across
/// `thickness` units in the perpendicular direction.
fn draw_thick_line(
    ctx: &mut Context<'_>,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    color: ratatui::style::Color,
    thickness: f64,
    num_lines: usize,
) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.001 {
        return;
    }

    // Perpendicular direction
    let nx = -dy / len;
    let ny = dx / len;

    let half = num_lines / 2;
    for i in 0..num_lines {
        let offset = (i as f64 - half as f64) * (thickness / half.max(1) as f64);
        let ox = nx * offset;
        let oy = ny * offset;
        ctx.draw(&Line {
            x1: x1 + ox,
            y1: y1 + oy,
            x2: x2 + ox,
            y2: y2 + oy,
            color,
        });
    }
}

/// Draw an arrowhead by fanning out the line at the endpoint.
/// `base_thickness` is the normal band width; the arrow widens to
/// `arrow_thickness` at the tip.
fn draw_arrow_segment(
    ctx: &mut Context<'_>,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    color: ratatui::style::Color,
    base_thickness: f64,
    arrow_thickness: f64,
    num_lines: usize,
) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.001 {
        return;
    }

    let nx = -dy / len;
    let ny = dx / len;

    let half = num_lines / 2;
    for i in 0..num_lines {
        let frac = (i as f64 - half as f64) / half.max(1) as f64;
        // Start offset uses base thickness, end offset uses arrow thickness
        let start_off = frac * base_thickness;
        let end_off = frac * arrow_thickness;
        ctx.draw(&Line {
            x1: x1 + nx * start_off,
            y1: y1 + ny * start_off,
            x2: x2 + nx * end_off,
            y2: y2 + ny * end_off,
            color,
        });
    }
}

/// Render protein as a cartoon / ribbon view on a ratatui Canvas with
/// HalfBlock marker for coloured pixel output.
///
/// - Helices are drawn as wide ribbon strips (11 parallel lines, ~1.5 unit offset).
/// - Sheets are drawn as wide flat arrows (13 parallel lines, ~1.8 unit offset)
///   with a triangular arrowhead at the last few residues.
/// - Coils and turns are drawn as thin lines (3 parallel lines, ~0.3 unit offset).
pub fn render_cartoon<'a>(
    protein: &'a Protein,
    camera: &'a Camera,
    color_scheme: &'a ColorScheme,
    width: f64,
    height: f64,
) -> Canvas<'a, impl Fn(&mut Context<'_>) + 'a> {
    Canvas::default()
        .marker(Marker::HalfBlock)
        .x_bounds([-width / 2.0, width / 2.0])
        .y_bounds([-height / 2.0, height / 2.0])
        .paint(move |ctx| {
            let backbone = protein.backbone_atoms();
            if backbone.is_empty() {
                return;
            }

            // We need a small lookahead to detect the last residue of a sheet
            // run so we can draw the arrowhead. Collect projected points first.
            struct SegInfo {
                px: f64,
                py: f64,
                x: f64,
                y: f64,
                color: ratatui::style::Color,
                ss: SecondaryStructure,
                next_ss: Option<SecondaryStructure>,
            }

            // Build segment list
            let mut segments: Vec<SegInfo> = Vec::new();
            {
                let mut prev_pt: Option<(f64, f64, String)> = None;

                for (i, (atom, residue, chain)) in backbone.iter().enumerate() {
                    let proj = camera.project(atom.x, atom.y, atom.z);

                    let same_chain = chain.id == prev_pt.as_ref().map(|p| p.2.as_str()).unwrap_or("");

                    if same_chain {
                        if let Some((px, py, _)) = &prev_pt {
                            let color = color_scheme.residue_color(residue, chain);
                            // Peek at next residue's SS for arrowhead detection
                            let next_ss = backbone.get(i + 1).and_then(|(_, next_res, next_chain)| {
                                if next_chain.id == chain.id {
                                    Some(next_res.secondary_structure)
                                } else {
                                    None
                                }
                            });
                            segments.push(SegInfo {
                                px: *px,
                                py: *py,
                                x: proj.x,
                                y: proj.y,
                                color,
                                ss: residue.secondary_structure,
                                next_ss,
                            });
                        }
                    }

                    prev_pt = Some((proj.x, proj.y, chain.id.clone()));
                }
            }

            // Draw segments
            for seg in &segments {
                match seg.ss {
                    SecondaryStructure::Helix => {
                        // Wide ribbon: 11 parallel lines, 1.5 unit thickness
                        draw_thick_line(ctx, seg.px, seg.py, seg.x, seg.y, seg.color, 1.5, 11);
                    }
                    SecondaryStructure::Sheet => {
                        // Check if this is the last segment of a sheet run
                        // (next residue is not a sheet, or end of chain)
                        let is_last = match seg.next_ss {
                            Some(SecondaryStructure::Sheet) => false,
                            _ => true,
                        };

                        if is_last {
                            // Arrowhead: widen from 1.8 to 3.0 at the tip
                            draw_arrow_segment(
                                ctx, seg.px, seg.py, seg.x, seg.y, seg.color, 1.8, 3.0, 15,
                            );
                        } else {
                            // Normal sheet band
                            draw_thick_line(ctx, seg.px, seg.py, seg.x, seg.y, seg.color, 1.8, 13);
                        }
                    }
                    SecondaryStructure::Turn | SecondaryStructure::Coil => {
                        // Thin coil/turn: 3 parallel lines, 0.3 unit thickness
                        draw_thick_line(ctx, seg.px, seg.py, seg.x, seg.y, seg.color, 0.3, 3);
                    }
                }
            }
        })
}
