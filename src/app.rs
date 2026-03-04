use crate::model::interface::{analyze_interface, InterfaceAnalysis};
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
    pub show_interface: bool,
    pub interface_analysis: InterfaceAnalysis,
    pub should_quit: bool,
}

impl App {
    pub fn new(mut protein: Protein, hd_mode: bool, term_cols: u16, term_rows: u16) -> Self {
        protein.center();
        let total_residues = protein.residue_count();
        let radius = protein.bounding_radius().max(1.0);
        // Dynamic zoom based on actual terminal size
        let vp_rows = term_rows.saturating_sub(4) as f64;
        let vp_cols = term_cols as f64;
        let auto_zoom = if hd_mode {
            0.6 * vp_cols.min(vp_rows * 2.0) / (2.0 * radius)
        } else {
            0.6 * (vp_cols * 2.0).min(vp_rows * 4.0) / (2.0 * radius)
        };
        let mut camera = Camera::default();
        camera.zoom = auto_zoom;

        // Pre-compute interface analysis (4.5A cutoff)
        let interface_analysis = analyze_interface(&protein, 4.5);

        Self {
            protein,
            camera,
            color_scheme: ColorScheme::new(ColorSchemeType::Structure, total_residues),
            viz_mode: VizMode::Backbone,
            current_chain: 0,
            hd_mode,
            show_help: false,
            show_interface: false,
            interface_analysis,
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

    fn rebuild_interface_colors(&mut self) {
        self.color_scheme = ColorScheme::new_interface(
            self.protein.residue_count(),
            self.current_chain,
            &self.interface_analysis,
            &self.protein,
        );
    }

    pub fn toggle_interface(&mut self) {
        self.show_interface = !self.show_interface;
        if self.show_interface {
            self.rebuild_interface_colors();
        } else {
            self.color_scheme = ColorScheme::new(
                ColorSchemeType::Structure,
                self.protein.residue_count(),
            );
        }
    }

    pub fn next_chain(&mut self) {
        if !self.protein.chains.is_empty() {
            self.current_chain = (self.current_chain + 1) % self.protein.chains.len();
            if self.show_interface {
                self.rebuild_interface_colors();
            }
        }
    }

    pub fn prev_chain(&mut self) {
        if !self.protein.chains.is_empty() {
            self.current_chain = if self.current_chain == 0 {
                self.protein.chains.len() - 1
            } else {
                self.current_chain - 1
            };
            if self.show_interface {
                self.rebuild_interface_colors();
            }
        }
    }

    pub fn chain_names(&self) -> Vec<String> {
        self.protein.chains.iter().map(|c| c.id.clone()).collect()
    }

    pub fn tick(&mut self) {
        self.camera.tick();
    }
}
