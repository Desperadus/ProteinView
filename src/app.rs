use crate::model::protein::Protein;
use crate::render::camera::Camera;
use crate::render::color::{ColorScheme, ColorSchemeType};

/// Visualization mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VizMode {
    Backbone,
    BallAndStick,
    Wireframe,
}

impl VizMode {
    pub fn next(&self) -> Self {
        match self {
            Self::Backbone => Self::BallAndStick,
            Self::BallAndStick => Self::Wireframe,
            Self::Wireframe => Self::Backbone,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Backbone => "Backbone",
            Self::BallAndStick => "Ball+Stick",
            Self::Wireframe => "Wireframe",
        }
    }
}

/// Main application state
pub struct App {
    pub protein: Protein,
    pub camera: Camera,
    pub color_scheme: ColorScheme,
    pub viz_mode: VizMode,
    pub current_chain: usize,
    pub hd_mode: bool,
    pub show_help: bool,
    pub should_quit: bool,
}

impl App {
    pub fn new(mut protein: Protein, hd_mode: bool, term_cols: u16, term_rows: u16) -> Self {
        protein.center();
        let total_residues = protein.residue_count();
        let radius = protein.bounding_radius().max(1.0);
        // Dynamic zoom based on actual terminal size
        // Viewport rows = total - 4 (header, status bar, help bar)
        let vp_rows = term_rows.saturating_sub(4) as f64;
        let vp_cols = term_cols as f64;
        let auto_zoom = if hd_mode {
            // HD: 1 pixel per col, 2 pixels per row
            0.6 * vp_cols.min(vp_rows * 2.0) / (2.0 * radius)
        } else {
            // Braille: 2 dots per col, 4 dots per row
            0.6 * (vp_cols * 2.0).min(vp_rows * 4.0) / (2.0 * radius)
        };
        let mut camera = Camera::default();
        camera.zoom = auto_zoom;
        Self {
            protein,
            camera,
            color_scheme: ColorScheme::new(ColorSchemeType::Structure, total_residues),
            viz_mode: VizMode::Backbone,
            current_chain: 0,
            hd_mode,
            show_help: false,
            should_quit: false,
        }
    }

    pub fn cycle_color(&mut self) {
        let next = self.color_scheme.scheme_type.next();
        self.color_scheme = ColorScheme::new(next, self.protein.residue_count());
    }

    pub fn cycle_viz_mode(&mut self) {
        self.viz_mode = self.viz_mode.next();
    }

    pub fn next_chain(&mut self) {
        if !self.protein.chains.is_empty() {
            self.current_chain = (self.current_chain + 1) % self.protein.chains.len();
        }
    }

    pub fn prev_chain(&mut self) {
        if !self.protein.chains.is_empty() {
            self.current_chain = if self.current_chain == 0 {
                self.protein.chains.len() - 1
            } else {
                self.current_chain - 1
            };
        }
    }

    pub fn tick(&mut self) {
        self.camera.tick();
    }
}
