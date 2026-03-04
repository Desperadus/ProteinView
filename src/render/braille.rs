use ratatui::symbols::Marker;
use ratatui::widgets::canvas::{Canvas, Context, Line};

use crate::model::protein::Protein;
use crate::render::camera::Camera;
use crate::render::color::ColorScheme;

/// Draw a thick line by rendering parallel offset lines along the perpendicular direction.
fn draw_thick_line(
    ctx: &mut Context<'_>,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    color: ratatui::style::Color,
    offsets: &[f64],
) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.001 {
        return;
    }

    // Perpendicular direction: (-dy, dx) normalized
    let nx = -dy / len;
    let ny = dx / len;

    for &off in offsets {
        let ox = nx * off;
        let oy = ny * off;
        ctx.draw(&Line {
            x1: x1 + ox,
            y1: y1 + oy,
            x2: x2 + ox,
            y2: y2 + oy,
            color,
        });
    }
}

/// Render protein on a ratatui Canvas with the Braille marker.
/// Lines are drawn with parallel offsets to create a thicker backbone trace.
pub fn render_protein<'a>(
    protein: &'a Protein,
    camera: &'a Camera,
    color_scheme: &'a ColorScheme,
    width: f64,
    height: f64,
) -> Canvas<'a, impl Fn(&mut Context<'_>) + 'a> {
    Canvas::default()
        .marker(Marker::Braille)
        .x_bounds([-width / 2.0, width / 2.0])
        .y_bounds([-height / 2.0, height / 2.0])
        .paint(move |ctx| {
            let backbone = protein.backbone_atoms();
            if backbone.is_empty() { return; }

            // Perpendicular offsets: centre line + 2 offsets on each side
            let offsets: [f64; 5] = [0.0, 0.3, -0.3, 0.6, -0.6];

            // Draw lines between consecutive C-alpha atoms in each chain
            let mut prev: Option<(f64, f64, &str)> = None;
            let mut prev_chain_id = "";

            for (atom, residue, chain) in &backbone {
                let proj = camera.project(atom.x, atom.y, atom.z);

                if chain.id == prev_chain_id {
                    if let Some((px, py, _)) = prev {
                        let color = color_scheme.residue_color(residue, chain);
                        draw_thick_line(ctx, px, py, proj.x, proj.y, color, &offsets);
                    }
                }

                prev = Some((proj.x, proj.y, &chain.id));
                prev_chain_id = &chain.id;
            }
        })
}
