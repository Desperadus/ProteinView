use std::collections::HashSet;

use ratatui::style::Color;
use crate::model::interface::InterfaceAnalysis;
use crate::model::protein::{Atom, Chain, Residue, SecondaryStructure};

/// Available color schemes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorSchemeType {
    Structure,
    Chain,
    Element,
    BFactor,
    Rainbow,
    Interface,
}

impl ColorSchemeType {
    pub fn next(&self) -> Self {
        match self {
            Self::Structure => Self::Chain,
            Self::Chain => Self::Element,
            Self::Element => Self::BFactor,
            Self::BFactor => Self::Rainbow,
            Self::Rainbow => Self::Structure,
            // Interface is toggled separately, skip it in the cycle
            Self::Interface => Self::Structure,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Structure => "Structure",
            Self::Chain => "Chain",
            Self::Element => "Element",
            Self::BFactor => "B-Factor",
            Self::Rainbow => "Rainbow",
            Self::Interface => "Interface",
        }
    }
}

/// Color scheme for rendering
#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub scheme_type: ColorSchemeType,
    total_residues: usize,
    /// For Interface mode: chain ID of the "focus" (antibody) chain
    focus_chain_id: String,
    /// For Interface mode: set of (chain_id, seq_num) at the interface
    interface_residues_by_id: HashSet<(String, i32)>,
}

impl ColorScheme {
    pub fn new(scheme_type: ColorSchemeType, total_residues: usize) -> Self {
        Self {
            scheme_type,
            total_residues,
            focus_chain_id: String::new(),
            interface_residues_by_id: HashSet::new(),
        }
    }

    pub fn new_interface(
        total_residues: usize,
        focus_chain: usize,
        analysis: &InterfaceAnalysis,
        protein: &crate::model::protein::Protein,
    ) -> Self {
        let focus_chain_id = protein.chains.get(focus_chain)
            .map(|c| c.id.clone())
            .unwrap_or_default();
        Self {
            scheme_type: ColorSchemeType::Interface,
            total_residues,
            focus_chain_id,
            interface_residues_by_id: analysis.interface_residues_by_id_with_protein(protein),
        }
    }

    /// Get color for a residue based on current scheme
    pub fn residue_color(&self, residue: &Residue, chain: &Chain) -> Color {
        match self.scheme_type {
            ColorSchemeType::Structure => self.structure_color(residue),
            ColorSchemeType::Chain => self.chain_color(chain),
            ColorSchemeType::Element => Color::Rgb(144, 144, 144),
            ColorSchemeType::BFactor => self.bfactor_color(residue),
            ColorSchemeType::Rainbow => self.rainbow_color(residue),
            ColorSchemeType::Interface => self.interface_color(residue, chain),
        }
    }

    /// Get color for an individual atom, respecting the current scheme.
    pub fn atom_color(&self, atom: &Atom, residue: &Residue, chain: &Chain) -> Color {
        match self.scheme_type {
            ColorSchemeType::Element => Self::element_color(atom),
            _ => self.residue_color(residue, chain),
        }
    }

    /// Interface color scheme:
    /// - Focus chain (antibody): green tones
    ///   - Interface residues: bright green
    ///   - Non-interface: dim green
    /// - Other chains (antigen): orange tones
    ///   - Interface residues: bright orange
    ///   - Non-interface: dim gray-brown
    fn interface_color(&self, residue: &Residue, chain: &Chain) -> Color {
        let is_contact = self.interface_residues_by_id
            .contains(&(chain.id.clone(), residue.seq_num));
        let is_focus = chain.id == self.focus_chain_id;

        match (is_focus, is_contact) {
            (true, true) => Color::Rgb(0, 255, 100),    // Bright green — antibody interface
            (true, false) => Color::Rgb(40, 100, 60),   // Dim green — antibody non-interface
            (false, true) => Color::Rgb(255, 165, 0),   // Bright orange — antigen interface
            (false, false) => Color::Rgb(100, 80, 60),  // Dim brown — antigen non-interface
        }
    }

    /// CPK-style element coloring
    fn element_color(atom: &Atom) -> Color {
        match atom.element.trim() {
            "C" => Color::Rgb(144, 144, 144),
            "N" => Color::Rgb(48, 80, 248),
            "O" => Color::Rgb(255, 13, 13),
            "S" => Color::Rgb(255, 255, 48),
            "H" => Color::Rgb(255, 255, 255),
            "P" => Color::Rgb(255, 128, 0),
            "FE" | "Fe" => Color::Rgb(224, 102, 51),
            _ => Color::Rgb(200, 200, 200),
        }
    }

    fn structure_color(&self, residue: &Residue) -> Color {
        match residue.secondary_structure {
            SecondaryStructure::Helix => Color::Rgb(255, 0, 128),
            SecondaryStructure::Sheet => Color::Rgb(255, 200, 0),
            SecondaryStructure::Turn  => Color::Rgb(96, 128, 255),
            SecondaryStructure::Coil  => Color::Rgb(0, 204, 0),
        }
    }

    fn chain_color(&self, chain: &Chain) -> Color {
        let chain_colors = [
            Color::Rgb(0, 180, 255),
            Color::Rgb(255, 100, 0),
            Color::Rgb(0, 220, 100),
            Color::Rgb(255, 50, 150),
            Color::Rgb(180, 100, 255),
            Color::Rgb(255, 220, 0),
            Color::Rgb(0, 200, 200),
            Color::Rgb(255, 150, 150),
        ];
        let idx = chain.id.bytes().next().unwrap_or(b'A') as usize % chain_colors.len();
        chain_colors[idx]
    }

    fn bfactor_color(&self, residue: &Residue) -> Color {
        let avg_b: f64 = if residue.atoms.is_empty() {
            0.0
        } else {
            residue.atoms.iter().map(|a| a.b_factor).sum::<f64>() / residue.atoms.len() as f64
        };
        let t = ((avg_b - 5.0) / 75.0).clamp(0.0, 1.0);
        let r = (t * 255.0) as u8;
        let b = ((1.0 - t) * 255.0) as u8;
        Color::Rgb(r, 0, b)
    }

    fn rainbow_color(&self, residue: &Residue) -> Color {
        if self.total_residues == 0 { return Color::White; }
        let t = residue.seq_num as f64 / self.total_residues as f64;
        let hue = (1.0 - t) * 300.0;
        let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
        Color::Rgb(r, g, b)
    }
}

/// Convert HSV to RGB (h: 0-360, s: 0-1, v: 0-1)
fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match h as u32 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (((r + m) * 255.0) as u8, ((g + m) * 255.0) as u8, ((b + m) * 255.0) as u8)
}
